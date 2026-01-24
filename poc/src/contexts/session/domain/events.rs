use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Event emitted when the full extraction process is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentGenerated {
    pub video_id: Id<YouTubeVideo>,
    pub file_path: String,
    pub cleaned_file_path: Option<String>,
    pub slide_count: u32,
    pub review_count: u32,
    pub review_slides: Vec<String>,
}
