use crate::contexts::ocr::domain::commands::ExtractTextCommand;
use crate::contexts::ocr::domain::events::{LowConfidenceDetected, TextExtracted};
use crate::contexts::ocr::infrastructure::tesseract::TesseractEngine;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::path::Path;

/// Handles the extract text command.
pub fn handle_extract_text(
    command: ExtractTextCommand,
) -> DomainResult<(TextExtracted, Option<LowConfidenceDetected>)> {
    if !Path::new(&command.image_path).exists() {
        return Err(ExtractionError::FileSystemError(format!(
            "Slide image not found: {}",
            command.image_path
        )));
    }

    let result = TesseractEngine::extract_text(&command)?;

    let mut low_confidence_event = None;
    if result.confidence < command.confidence_threshold {
        low_confidence_event = Some(LowConfidenceDetected {
            slide_id: command.slide_id.clone(),
            confidence: result.confidence,
            threshold: command.confidence_threshold,
        });
    }

    let success_event = TextExtracted {
        slide_id: command.slide_id,
        text: result.text,
        confidence: result.confidence,
    };

    Ok((success_event, low_confidence_event))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain::{Id, Slide};

    #[test]
    fn test_handle_extract_text_missing_file() {
        let command = ExtractTextCommand {
            slide_id: Id::<Slide>::new(),
            image_path: "/nonexistent/image.jpg".to_string(),
            languages: vec!["eng".to_string()],
            confidence_threshold: 0.6,
        };

        let result = handle_extract_text(command);
        assert!(result.is_err());
        if let Err(ExtractionError::FileSystemError(msg)) = result {
            assert!(msg.contains("not found"));
        } else {
            panic!("Expected FileSystemError");
        }
    }
}
