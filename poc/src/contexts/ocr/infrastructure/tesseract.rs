use crate::contexts::ocr::domain::commands::ExtractTextCommand;
use crate::contexts::ocr::domain::state::OcrResult;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::fs;
use std::process::Command;
use tempfile::NamedTempFile;

/// Infrastructure component for running Tesseract OCR.
pub struct TesseractEngine;

impl TesseractEngine {
    /// Executes OCR on an image and returns the result.
    pub fn extract_text(command: &ExtractTextCommand) -> DomainResult<OcrResult> {
        // We'll use a temporary file for Tesseract output to avoid clutter
        // Tesseract adds the extension itself if we provide a base name
        let output_temp = NamedTempFile::new().map_err(|e| {
            ExtractionError::FileSystemError(format!("Failed to create temp file: {}", e))
        })?;
        let output_base = output_temp.path().to_str().unwrap();

        let mut tesseract_cmd = Command::new("tesseract");
        tesseract_cmd.arg(&command.image_path);
        tesseract_cmd.arg(output_base);

        // Add languages
        if !command.languages.is_empty() {
            tesseract_cmd.arg("-l");
            tesseract_cmd.arg(command.languages.join("+"));
        }

        // Request TSV output as well to get confidence scores
        tesseract_cmd.arg("tsv");

        let output = tesseract_cmd.output().map_err(|e| {
            ExtractionError::ExternalDependencyUnavailable(format!(
                "Tesseract failed to start: {}",
                e
            ))
        })?;

        if !output.status.success() {
            return Err(ExtractionError::OcrFailed(
                command.slide_id.clone(),
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Tesseract created <output_base>.tsv
        let tsv_path = format!("{}.tsv", output_base);
        let tsv_content = fs::read_to_string(&tsv_path).map_err(|e| {
            ExtractionError::FileSystemError(format!("Failed to read Tesseract TSV output: {}", e))
        })?;

        // Cleanup the TSV file
        let _ = fs::remove_file(tsv_path);

        let (text, confidence) = Self::parse_tsv(&tsv_content);

        Ok(OcrResult {
            slide_id: command.slide_id.clone(),
            text,
            confidence,
            languages: command.languages.clone(),
        })
    }

    /// Parses Tesseract TSV output to extract text and calculate average confidence.
    pub fn parse_tsv(tsv_content: &str) -> (String, f64) {
        let mut full_text = Vec::new();
        let mut total_confidence = 0.0;
        let mut word_count = 0;

        // Skip header
        for line in tsv_content.lines().skip(1) {
            let cols: Vec<&str> = line.split('\t').collect();
            if cols.len() >= 12 {
                let conf_str = cols[10];
                let text_str = cols[11];

                if let Ok(conf) = conf_str.parse::<f64>() {
                    // Tesseract uses -1 for non-word levels
                    if conf >= 0.0 {
                        let trimmed_text = text_str.trim();
                        if !trimmed_text.is_empty() {
                            full_text.push(trimmed_text);
                            total_confidence += conf;
                            word_count += 1;
                        }
                    }
                }
            }
        }

        let avg_confidence = if word_count > 0 {
            (total_confidence / word_count as f64) / 100.0
        } else {
            0.0
        };

        (full_text.join(" "), avg_confidence)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tsv_success() {
        let tsv_data =
            "level	page_num	block_num	par_num	line_num	word_num	left	top	width	height	conf	text
1	1	0	0	0	0	0	0	1920	1080	-1
2	1	1	0	0	0	100	100	200	50	-1
3	1	1	1	0	0	100	100	200	50	-1
4	1	1	1	1	0	100	100	200	50	-1
5	1	1	1	1	1	100	100	100	50	90	Hello
5	1	1	1	1	2	210	100	90	50	80	World";

        let (text, confidence) = TesseractEngine::parse_tsv(tsv_data);
        assert_eq!(text, "Hello World");
        assert_eq!(confidence, 0.85); // (90+80)/2 / 100
    }

    #[test]
    fn test_parse_tsv_empty() {
        let tsv_data =
            "level	page_num	block_num	par_num	line_num	word_num	left	top	width	height	conf	text";
        let (text, confidence) = TesseractEngine::parse_tsv(tsv_data);
        assert_eq!(text, "");
        assert_eq!(confidence, 0.0);
    }
}
