use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Event emitted when the full extraction process is complete.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentGenerated {
    pub video_id: Id<YouTubeVideo>,
    pub file_path: String,
    pub slide_count: u32,
}
