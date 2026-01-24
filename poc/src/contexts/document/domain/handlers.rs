use crate::contexts::document::domain::commands::GenerateDocumentCommand;
use crate::contexts::document::domain::events::DocumentGenerated;
use crate::contexts::document::infrastructure::PdfGenerator;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::fs;
use std::path::Path;
use tera::{Context, Tera};
use tracing::info;

/// Handles the generate document command using Tera templates.
pub fn handle_generate_document(
    command: GenerateDocumentCommand,
) -> DomainResult<DocumentGenerated> {
    if command.slides.is_empty() {
        return Err(ExtractionError::InternalError(
            "Cannot generate document with no slides".to_string(),
        ));
    }

    let mut tera = Tera::default();

    // Default template
    let default_template = r#"
# {{ title }}

## Video Information

- **URL:** {{ url }}
- **Duration:** {{ duration }} seconds
- **Extracted Slides:** {{ slides | length }}

{% if transcription %}
## Transcription

<details>
<summary>Click to expand transcription</summary>

{{ transcription }}

</details>
{% endif %}

{% if include_timeline_diagram %}
## Timeline

```mermaid
graph LR
{% for slide in slides %}
    S{{ slide.slide_index }}["Slide {{ slide.slide_index }} ({{ slide.timestamp | round }}s)"]
    {% if not loop.first %}
    S{{ loop.index0 }} --> S{{ slide.slide_index }}
    {% endif %}
{% endfor %}
```
{% endif %}

## Slides Detail

{% for slide in slides %}
### Slide {{ slide.slide_index }}

{% if slide.requires_human_review %}
> ⚠️ **Note:** This frame was identified by the AI as potentially not being a slide (e.g., speaker view). Please review.
{% endif %}

- **Timestamp:** {{ slide.timestamp | round(precision=2) }}s

![Slide {{ slide.slide_index }}](./slides/{{ slide.image_path | split(pat="/") | last }})

#### Extracted Text

{% if slide.text | trim %}
{{ slide.text | trim }}
{% else %}
*No text detected.*
{% endif %}

{% if slide.transcription %}
#### Transcription
{{ slide.transcription | trim }}
{% endif %}

---
{% endfor %}
"#;

    tera.add_raw_template("default", default_template)
        .map_err(|e| {
            ExtractionError::TemplateError(format!("Failed to load default template: {}", e))
        })?;

    // 1. Generate standard Markdown (User facing) - Includes ALL slides (even reviewed ones)
    let mut context = Context::new();
    context.insert("title", &command.title);
    context.insert("url", &command.url);
    context.insert("duration", &command.duration);
    context.insert("slides", &command.slides);
    context.insert("transcription", &command.transcription);
    context.insert(
        "include_timeline_diagram",
        &command.include_timeline_diagram,
    );

    let rendered = tera
        .render("default", &context)
        .map_err(|e| ExtractionError::TemplateError(format!("Failed to render template: {}", e)))?;

    // Write to file
    let output_path = Path::new(&command.output_path);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).map_err(|e| ExtractionError::FileSystemError(e.to_string()))?;
    }

    fs::write(output_path, rendered)
        .map_err(|e| ExtractionError::FileSystemError(e.to_string()))?;

    let mut pdf_path_result = None;

    // 2. Generate PDF if requested (using sanitized Markdown AND filtered slides)
    if command.generate_pdf {
        // Sanitize data to prevent Typst/Pandoc errors (specifically @ symbols causing citation errors)
        // We do this separately so the user's Markdown file remains clean (no backslashes)
        let sanitized_title = command.title.replace("@", "\\@");
        let sanitized_transcription = command
            .transcription
            .as_ref()
            .map(|t| t.replace("@", "\\@"));

        // Filter out non-slides (duplicates/bad frames) AND sanitize text
        let sanitized_slides: Vec<_> = command
            .slides
            .iter()
            .filter(|s| !s.requires_human_review) // Remove duplicates/non-slides for PDF
            .map(|s| {
                let mut new_s = s.clone();
                new_s.text = new_s.text.replace("@", "\\@");
                new_s.transcription = new_s.transcription.as_ref().map(|t| t.replace("@", "\\@"));
                new_s
            })
            .collect();

        let mut pdf_context = Context::new();
        pdf_context.insert("title", &sanitized_title);
        pdf_context.insert("url", &command.url);
        pdf_context.insert("duration", &command.duration);
        pdf_context.insert("slides", &sanitized_slides);
        pdf_context.insert("transcription", &sanitized_transcription);
        pdf_context.insert(
            "include_timeline_diagram",
            &command.include_timeline_diagram,
        );

        let pdf_rendered = tera.render("default", &pdf_context).map_err(|e| {
            ExtractionError::TemplateError(format!("Failed to render PDF template: {}", e))
        })?;

        // Write to temporary file for PDF generation
        let temp_md_path = output_path.with_extension("pdf_temp.md");
        fs::write(&temp_md_path, pdf_rendered).map_err(|e| {
            ExtractionError::FileSystemError(format!("Failed to write temp markdown: {}", e))
        })?;

        let temp_md_path_str = temp_md_path.to_str().ok_or_else(|| {
            ExtractionError::InternalError("Failed to convert temp path to string".to_string())
        })?;

        let pdf_path = output_path.with_extension("pdf");
        let pdf_path_str = pdf_path.to_str().ok_or_else(|| {
            ExtractionError::InternalError("Failed to convert PDF path to string".to_string())
        })?;

        info!("PDF generation requested. Output path: {}", pdf_path_str);

        let generation_result = PdfGenerator::generate_pdf(
            temp_md_path_str,
            pdf_path_str,
            command.pdf_template.as_deref(),
        );

        // Cleanup temporary file
        let _ = fs::remove_file(&temp_md_path);

        generation_result?;
        pdf_path_result = Some(pdf_path_str.to_string());
    }

    Ok(DocumentGenerated {
        video_id: command.video_id,
        file_path: command.output_path.clone(),
        pdf_path: pdf_path_result,
        slide_count: command.slides.len() as u32,
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
                    transcription: Some("Hello World spoken".to_string()),
                    requires_human_review: false,
                },
                SlideData {
                    slide_index: 2,
                    timestamp: 15.0,
                    image_path: "slide_0002.jpg".to_string(),
                    text: "Page 2 Content".to_string(),
                    transcription: Some("Page 2 spoken".to_string()),
                    requires_human_review: true,
                },
            ],
            transcription: Some("Sample transcription".to_string()),
            output_path: output_path.to_str().unwrap().to_string(),
            include_timeline_diagram: true,
            generate_pdf: false,
            pdf_template: None,
        };

        let result = handle_generate_document(command).unwrap();
        assert_eq!(result.slide_count, 2);
        assert!(output_path.exists());
        assert!(result.pdf_path.is_none());

        let content = fs::read_to_string(output_path).unwrap();
        assert!(content.contains("# Test Video"));
        assert!(content.contains("graph LR"));
        assert!(content.contains("Hello World"));
        assert!(content.contains("Hello World spoken"));
        assert!(content.contains("Page 2 spoken"));
    }
}
