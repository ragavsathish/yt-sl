use crate::shared::domain::{Id, Slide};
use serde::{Deserialize, Serialize};

/// Represents the result of an OCR operation on a slide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrResult {
    /// The ID of the slide
    pub slide_id: Id<Slide>,
    /// The extracted text
    pub text: String,
    /// Average confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Languages used for OCR
    pub languages: Vec<String>,
}
