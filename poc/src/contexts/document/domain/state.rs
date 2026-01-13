// Document state entities
use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentModel {
    pub video_id: Id<YouTubeVideo>,
    pub title: String,
}
