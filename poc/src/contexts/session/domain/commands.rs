use crate::shared::domain::{Id, Session};
use serde::{Deserialize, Serialize};

/// Command to start a full video slide extraction session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartExtractionSessionCommand {
    pub session_id: Id<Session>,
    pub youtube_url: String,
    pub output_dir: String,
    pub frame_interval_secs: u64,
    pub similarity_threshold: f64,
    pub confidence_threshold: f64,
    pub languages: Vec<String>,
}

impl Default for StartExtractionSessionCommand {
    fn default() -> Self {
        Self {
            session_id: Id::new(),
            youtube_url: String::new(),
            output_dir: "output".to_string(),
            frame_interval_secs: 5,
            similarity_threshold: 0.95,
            confidence_threshold: 0.6,
            languages: vec!["eng".to_string()],
        }
    }
}
