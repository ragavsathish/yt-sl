// Placeholder for events
use crate::shared::domain::{Id, VideoFrame, YouTubeVideo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniqueSlidesIdentified {
    pub video_id: Id<YouTubeVideo>,
    pub slide_count: u32,
    pub slides_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlidePreserved {
    pub video_id: Id<YouTubeVideo>,
    pub frame_id: Id<VideoFrame>,
    pub slide_index: u32,
    pub slide_path: String,
}
