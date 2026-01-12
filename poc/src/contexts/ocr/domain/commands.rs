use crate::shared::domain::{Id, Slide};
use serde::{Deserialize, Serialize};

/// Command to extract text from a slide image using OCR.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractTextCommand {
    /// The ID of the slide being processed
    pub slide_id: Id<Slide>,
    /// Path to the slide image
    pub image_path: String,
    /// Languages to use for OCR (e.g., ["eng", "deu"])
    pub languages: Vec<String>,
    /// Minimum confidence threshold (0.0 to 1.0)
    pub confidence_threshold: f64,
}

impl Default for ExtractTextCommand {
    fn default() -> Self {
        Self {
            slide_id: Id::new(),
            image_path: String::new(),
            languages: vec!["eng".to_string()],
            confidence_threshold: 0.6,
        }
    }
}
