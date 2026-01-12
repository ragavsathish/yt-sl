use crate::shared::domain::{Id, Slide};
use serde::{Deserialize, Serialize};

/// Event emitted when text has been successfully extracted from a slide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtracted {
    pub slide_id: Id<Slide>,
    pub text: String,
    pub confidence: f64,
}

/// Event emitted when OCR failed for a slide.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrFailed {
    pub slide_id: Id<Slide>,
    pub error: String,
}

/// Event emitted when OCR confidence is below the threshold.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowConfidenceDetected {
    pub slide_id: Id<Slide>,
    pub confidence: f64,
    pub threshold: f64,
}
