use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Event emitted when the Markdown document has been generated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentGenerated {
    pub video_id: Id<YouTubeVideo>,
    pub file_path: String,
    pub slide_count: u32,
}
