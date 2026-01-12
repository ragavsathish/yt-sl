use crate::contexts::video::infrastructure::VideoMetadata;
use crate::shared::domain::{Id, Session};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    Starting,
    MetadataFetched,
    VideoDownloaded,
    FramesExtracted,
    UniqueSlidesIdentified,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideState {
    pub slide_index: u32,
    pub timestamp: f64,
    pub image_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: Id<Session>,
    pub status: SessionStatus,
    pub video_metadata: Option<VideoMetadata>,
    pub video_path: Option<String>,
    pub frames_dir: Option<String>,
    pub slides_dir: Option<String>,
    pub report_path: Option<String>,
    pub slides: Vec<SlideState>,
}

impl SessionState {
    pub fn new(session_id: Id<Session>) -> Self {
        Self {
            session_id,
            status: SessionStatus::Starting,
            video_metadata: None,
            video_path: None,
            frames_dir: None,
            slides_dir: None,
            report_path: None,
            slides: Vec::new(),
        }
    }
}
