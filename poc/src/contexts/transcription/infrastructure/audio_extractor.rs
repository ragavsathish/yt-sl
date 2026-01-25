use crate::contexts::transcription::domain::commands::ExtractAudioCommand;
use crate::contexts::transcription::domain::ports::AudioExtractorPort;
use crate::shared::domain::{DomainResult, ExtractionError};
use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

/// FFmpeg-based audio extractor.
pub struct AudioExtractor;

impl AudioExtractor {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AudioExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AudioExtractorPort for AudioExtractor {
    /// Extracts audio from a video file as 16kHz mono WAV.
    async fn extract_audio(&self, command: ExtractAudioCommand) -> DomainResult<String> {
        let output_path =
            PathBuf::from(&command.output_dir).join(format!("{}.wav", command.youtube_video_id));
        let output_path_str = output_path.to_str().ok_or_else(|| {
            ExtractionError::AudioExtractionFailed("Invalid output path".to_string())
        })?;

        if output_path.exists() {
            info!(
                "Audio file already exists at {}. Skipping extraction.",
                output_path_str
            );
            return Ok(output_path_str.to_string());
        }

        let output = Command::new("ffmpeg")
            .arg("-i")
            .arg(&command.video_path)
            .arg("-vn")
            .arg("-acodec")
            .arg("pcm_s16le")
            .arg("-ar")
            .arg("16000")
            .arg("-ac")
            .arg("1")
            .arg("-y")
            .arg(output_path_str)
            .output()
            .map_err(|e| {
                ExtractionError::AudioExtractionFailed(format!("Failed to execute FFmpeg: {}", e))
            })?;

        if !output.status.success() {
            return Err(ExtractionError::AudioExtractionFailed(format!(
                "FFmpeg failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(output_path_str.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain::{Id, YouTubeVideo};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_extract_audio_fail_nonexistent_video() {
        let extractor = AudioExtractor::new();
        let temp = tempdir().unwrap();
        let command = ExtractAudioCommand {
            video_id: Id::<YouTubeVideo>::new(),
            youtube_video_id: "dQw4w9WgXcQ".to_string(),
            video_path: "nonexistent.mp4".to_string(),
            output_dir: temp.path().to_str().unwrap().to_string(),
        };

        let result = extractor.extract_audio(command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extract_audio_skip_if_exists() {
        let extractor = AudioExtractor::new();
        let temp = tempdir().unwrap();
        let output_dir = temp.path().to_str().unwrap().to_string();
        let youtube_id = "dQw4w9WgXcQ";

        // Create a dummy audio file at the expected location
        let expected_path = temp.path().join(format!("{}.wav", youtube_id));
        std::fs::write(&expected_path, "dummy content").unwrap();

        let command = ExtractAudioCommand {
            video_id: Id::<YouTubeVideo>::new(),
            youtube_video_id: youtube_id.to_string(),
            video_path: "nonexistent_video.mp4".to_string(), // Should not be used
            output_dir,
        };

        // This should succeed because it detects the existing file and skips ffmpeg
        let result = extractor.extract_audio(command).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), expected_path.to_str().unwrap());
    }
}
