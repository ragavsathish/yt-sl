use crate::shared::domain::{Id, Session};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionStatus {
    Starting,
    Downloading,
    ExtractingFrames,
    Deduplicating,
    RunningOCR,
    GeneratingDocument,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: Id<Session>,
    pub status: SessionStatus,
}
