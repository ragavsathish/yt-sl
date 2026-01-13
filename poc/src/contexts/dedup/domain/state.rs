// Placeholder for state
use crate::shared::domain::{Id, VideoFrame, YouTubeVideo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub id: Id<VideoFrame>,
    pub video_id: Id<YouTubeVideo>,
    pub frame_number: u32,
    pub timestamp: f64,
    pub hash: String,
}
