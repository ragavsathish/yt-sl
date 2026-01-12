use crate::contexts::document::domain::commands::GenerateDocumentCommand;
use crate::contexts::document::domain::events::DocumentGenerated;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::fs;
use std::path::Path;

/// Handles the generate document command.
pub fn handle_generate_document(
    command: GenerateDocumentCommand,
) -> DomainResult<DocumentGenerated> {
    if command.slides.is_empty() {
        return Err(ExtractionError::InternalError(
            "Cannot generate document with no slides".to_string(),
        ));
    }

    let mut markdown = String::new();
    let slide_count = command.slides.len() as u32;

    // Title and Metadata
    markdown.push_str(&format!("# {}\n\n", command.title));
    markdown.push_str("## Video Information\n\n");
    markdown.push_str(&format!("- **URL:** {}\n", command.url));
    markdown.push_str(&format!("- **Duration:** {} seconds\n", command.duration));
    markdown.push_str(&format!("- **Extracted Slides:** {}\n\n", slide_count));

    // Timeline Diagram (Mermaid)
    if command.include_timeline_diagram {
        markdown.push_str("## Timeline\n\n");
        markdown.push_str("```mermaid\ngraph LR\n");
        for slide in &command.slides {
            let label = format!("Slide {} ({:.0}s)", slide.slide_index, slide.timestamp);
            markdown.push_str(&format!("    S{}[\"{}\"]\n", slide.slide_index, label));
            if slide.slide_index > 1 {
                markdown.push_str(&format!(
                    "    S{} --> S{}\n",
                    slide.slide_index - 1,
                    slide.slide_index
                ));
            }
        }
        markdown.push_str("```\n\n");
    }

    // Slides
    markdown.push_str("## Slides Detail\n\n");
    for slide in command.slides {
        markdown.push_str(&format!("### Slide {}\n\n", slide.slide_index));
        markdown.push_str(&format!("- **Timestamp:** {:.2}s\n\n", slide.timestamp));

        // Relative image path for the markdown file
        let img_path = Path::new(&slide.image_path);
        let img_name = img_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image.jpg");
        markdown.push_str(&format!("![Slide {}]({})\n\n", slide.slide_index, img_name));

        markdown.push_str("#### Extracted Text\n\n");
        if slide.text.trim().is_empty() {
            markdown.push_str("*No text detected.*\n\n");
        } else {
            markdown.push_str(&format!("{}\n\n", slide.text.trim()));
        }
        markdown.push_str("---\n\n");
    }

    // Write to file
    let output_path = Path::new(&command.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| ExtractionError::FileSystemError(e.to_string()))?;
    }

    fs::write(output_path, markdown)
        .map_err(|e| ExtractionError::FileSystemError(e.to_string()))?;

    Ok(DocumentGenerated {
        video_id: command.video_id,
        file_path: command.output_path.clone(),
        slide_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::document::domain::commands::SlideData;
    use crate::shared::domain::{Id, YouTubeVideo};
    use tempfile::TempDir;

    #[test]
    fn test_handle_generate_document_success() {
        let temp_dir = TempDir::new().unwrap();
        let output_path = temp_dir.path().join("output.md");

        let command = GenerateDocumentCommand {
            video_id: Id::<YouTubeVideo>::new(),
            title: "Test Video".to_string(),
            url: "https://youtube.com/watch?v=123".to_string(),
            duration: 60,
            slides: vec![
                SlideData {
                    slide_index: 1,
                    timestamp: 5.0,
                    image_path: "slide_0001.jpg".to_string(),
                    text: "Hello World".to_string(),
                },
                SlideData {
                    slide_index: 2,
                    timestamp: 15.0,
                    image_path: "slide_0002.jpg".to_string(),
                    text: "Page 2 Content".to_string(),
                },
            ],
            output_path: output_path.to_str().unwrap().to_string(),
            include_timeline_diagram: true,
        };

        let result = handle_generate_document(command).unwrap();
        assert_eq!(result.slide_count, 2);
        assert!(output_path.exists());

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("# Test Video"));
        assert!(content.contains("graph LR"));
        assert!(content.contains("Hello World"));
    }
}
