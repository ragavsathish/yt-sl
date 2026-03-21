use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// OCR statistics for the summary report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcrStats {
    pub success_rate: f64,
    pub avg_confidence: f64,
    pub low_confidence_count: u32,
    pub failure_count: u32,
}

/// Event emitted when the full extraction process is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentGenerated {
    pub video_id: Id<YouTubeVideo>,
    pub file_path: String,
    pub pdf_path: Option<String>,
    pub cleaned_file_path: Option<String>,
    pub cleaned_pdf_path: Option<String>,
    pub slide_count: u32,
    pub total_frames: u32,
    pub review_count: u32,
    pub review_slides: Vec<String>,
    pub ocr_stats: Option<OcrStats>,
}
