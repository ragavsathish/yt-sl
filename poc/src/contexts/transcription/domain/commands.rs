use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Command to extract audio from a video file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractAudioCommand {
    /// The video ID being processed
    pub video_id: Id<YouTubeVideo>,
    /// Path to the source video file
    pub video_path: String,
    /// Directory where the audio file should be saved
    pub output_dir: String,
}

/// Command to transcribe an audio file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeAudioCommand {
    /// The video ID being processed
    pub video_id: Id<YouTubeVideo>,
    /// Path to the audio file to transcribe
    pub audio_path: String,
}
