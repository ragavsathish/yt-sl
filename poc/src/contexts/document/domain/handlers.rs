use crate::contexts::document::domain::commands::GenerateDocumentCommand;
use crate::contexts::document::domain::events::DocumentGenerated;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::fs;
use std::path::Path;
use tera::{Context, Tera};

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

- **Timestamp:** {{ slide.timestamp | round(precision=2) }}s

![Slide {{ slide.slide_index }}]({{ slide.image_path | split(pat="/") | last }})

#### Extracted Text

{% if slide.text | trim %}
{{ slide.text | trim }}
{% else %}
*No text detected.*
{% endif %}

---
{% endfor %}
"#;

    tera.add_raw_template("default", default_template)
        .map_err(|e| {
            ExtractionError::TemplateError(format!("Failed to load default template: {}", e))
        })?;

    let mut context = Context::new();
    context.insert("title", &command.title);
    context.insert("url", &command.url);
    context.insert("duration", &command.duration);
    context.insert("slides", &command.slides);
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

    Ok(DocumentGenerated {
        video_id: command.video_id,
        file_path: command.output_path.clone(),
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
