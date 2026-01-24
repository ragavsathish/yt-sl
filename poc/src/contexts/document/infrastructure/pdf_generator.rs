use crate::shared::domain::{DomainResult, ExtractionError};
use crate::shared::infrastructure::dependencies::{Dependency, DependencyChecker};
use std::fs;
use std::path::Path;
use std::process::Command;
use tracing::{info, warn};

/// Default Typst template for professional slide-looking PDFs.
const DEFAULT_TYPST_TEMPLATE: &str = r##"
#set page(
  paper: "presentation-16-9",
  margin: (x: 1cm, y: 1cm),
  fill: rgb("#ffffff"),
)

#set text(
  font: "libertinus serif",
  size: 18pt,
  fill: rgb("#202124"),
)

// Title Slide
#show heading.where(level: 1): it => {
  set align(center + horizon)
  set text(size: 40pt, weight: "bold", fill: rgb("#1a73e8"))
  pagebreak(weak: true)
  v(1fr)
  it.body
  v(1fr)
  pagebreak(weak: true)
}

// Section Headers
#show heading.where(level: 2): it => {
  set text(size: 28pt, weight: "bold", fill: rgb("#5f6368"))
  v(1em)
  it
  v(0.5em)
}

// Slide Headers (each starts a new page)
#show heading.where(level: 3): it => {
  pagebreak(weak: true)
  v(0.2em)
  block(
    fill: rgb("#f1f3f4"),
    width: 100%,
    inset: 15pt,
    radius: 8pt,
    text(size: 24pt, weight: "bold", fill: rgb("#1967d2"))[#it.body]
  )
  v(0.5em)
}

// Image Styling
#show image: it => {
  set align(center)
  block(
    stroke: 1pt + rgb("#dadce0"),
    radius: 4pt,
    clip: true,
    it
  )
}

// Mermaid diagrams / Code blocks
#show raw: it => {
  block(
    fill: rgb("#f8f9fa"),
    width: 100%,
    inset: 10pt,
    radius: 4pt,
    text(size: 14pt, font: "monospace")[#it]
  )
}

// Pandoc compatibility
#let horizontalrule = line(length: 100%, stroke: 0.5pt + rgb("#dadce0"))
#let tightlist(it) = it

$body$
"##;

/// Generator for PDF documents using Pandoc and Typst.
pub struct PdfGenerator;

impl PdfGenerator {
    /// Generates a PDF from a Markdown file.
    pub fn generate_pdf(
        markdown_path: &str,
        output_path: &str,
        template_path: Option<&str>,
    ) -> DomainResult<()> {
        info!("Generating PDF from {} to {}", markdown_path, output_path);

        let markdown_path_obj = Path::new(markdown_path);
        if !markdown_path_obj.exists() {
            return Err(ExtractionError::FileSystemError(format!(
                "Markdown file not found: {}",
                markdown_path
            )));
        }

        // Validate dependencies
        let checker = DependencyChecker::new();
        if !checker.check(&Dependency::Pandoc).is_ok() {
            return Err(ExtractionError::ExternalDependencyUnavailable(
                "Pandoc is required for PDF generation. Please install it.".to_string(),
            ));
        }
        if !checker.check(&Dependency::Typst).is_ok() {
            return Err(ExtractionError::ExternalDependencyUnavailable(
                "Typst is required for PDF generation. Please install it.".to_string(),
            ));
        }

        let mut cmd = Command::new("pandoc");
        cmd.arg(markdown_path)
            .arg("-o")
            .arg(output_path)
            .arg("--pdf-engine=typst");

        // Handle templates
        let mut _temp_template_path = None;
        if let Some(template) = template_path {
            info!("Using custom Typst template: {}", template);
            cmd.arg("--template").arg(template);
        } else {
            info!("Using default professional slide template");
            let temp_file = markdown_path_obj.parent().unwrap().join("template.typ");
            fs::write(&temp_file, DEFAULT_TYPST_TEMPLATE)
                .map_err(|e| ExtractionError::FileSystemError(e.to_string()))?;
            cmd.arg("--template").arg(&temp_file);
            _temp_template_path = Some(temp_file);
        }

        // Ensure images are correctly embedded by setting the resource path
        if let Some(parent) = markdown_path_obj.parent() {
            cmd.arg("--resource-path").arg(parent);
        }

        let result = cmd.output().map_err(|e| {
            ExtractionError::InternalError(format!("Failed to execute pandoc: {}", e))
        });

        // Cleanup temporary template if we created one
        if let Some(path) = _temp_template_path {
            let _ = fs::remove_file(path);
        }

        let output = result?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            warn!("Pandoc failed: {}", stderr);
            return Err(ExtractionError::InternalError(format!(
                "Pandoc failed to generate PDF: {}",
                stderr
            )));
        }

        info!("PDF generated successfully at {}", output_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_pdf_fails_missing_markdown() {
        let result = PdfGenerator::generate_pdf("non_existent.md", "output.pdf", None);
        assert!(result.is_err());
        match result {
            Err(ExtractionError::FileSystemError(msg)) => assert!(msg.contains("not found")),
            _ => panic!("Expected FileSystemError, got {:?}", result),
        }
    }

    #[test]
    fn test_generate_pdf_success() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let markdown_path = temp_dir.path().join("test.md");
        let pdf_path = temp_dir.path().join("test.pdf");

        std::fs::write(&markdown_path, "# Test\n\nHello PDF!").unwrap();

        let result = PdfGenerator::generate_pdf(
            markdown_path.to_str().unwrap(),
            pdf_path.to_str().unwrap(),
            None,
        );

        // This test will only pass if pandoc and typst are installed
        if result.is_ok() {
            assert!(pdf_path.exists());
        } else {
            // If they are not installed, it should fail with ExternalDependencyUnavailable
            match result {
                Err(ExtractionError::ExternalDependencyUnavailable(_)) => {}
                _ => panic!(
                    "Expected success or ExternalDependencyUnavailable, got {:?}",
                    result
                ),
            }
        }
    }
}
