use crate::contexts::transcription::domain::state::TranscriptionResult;
use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Event emitted when audio is successfully extracted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioExtracted {
    pub video_id: Id<YouTubeVideo>,
    pub audio_path: String,
}

/// Event emitted when audio is successfully transcribed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextTranscribed {
    pub video_id: Id<YouTubeVideo>,
    pub result: TranscriptionResult,
}
