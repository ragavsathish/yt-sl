use crate::shared::domain::Id;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ExtractionError {
    #[error("Invalid YouTube URL: {0}")]
    InvalidUrl(String),

    #[error("Video ID '{0}' is unavailable")]
    VideoUnavailable(Id<YouTubeVideo>),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Download failed after {0} retries: {1}")]
    DownloadFailed(u8, String),

    #[error("Frame extraction failed: {0}")]
    FrameExtractionFailed(String),

    #[error("Hash computation failed for frame {0}")]
    HashComputationFailed(Id<VideoFrame>),

    #[error("OCR failed for slide {0}: {1}")]
    OcrFailed(Id<Slide>, String),

    #[error("Markdown generation failed: {0}")]
    MarkdownGenerationFailed(String),

    #[error("No unique slides found")]
    NoUniqueSlidesFound,

    #[error("Output directory not writable: {0:?}")]
    OutputDirectoryNotWritable(String),

    #[error("Insufficient memory: required {0}MB")]
    InsufficientMemory(u64),

    #[error("Network timeout after {0:?}")]
    NetworkTimeout(Duration),

    #[error("External dependency unavailable: {0}")]
    ExternalDependencyUnavailable(String),
}

pub type DomainResult<T> = Result<T, ExtractionError>;

#[derive(Debug, Clone)]
pub struct YouTubeVideo;

#[derive(Debug, Clone)]
pub struct VideoFrame;

#[derive(Debug, Clone)]
pub struct Slide;

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn test_invalid_url_error() {
        let err = ExtractionError::InvalidUrl("https://example.com".to_string());
        assert_eq!(err.to_string(), "Invalid YouTube URL: https://example.com");
    }

    #[test]
    fn test_video_unavailable_error() {
        let uuid = Uuid::new_v4();
        let video_id = Id::<YouTubeVideo>::from_uuid(uuid);
        let err = ExtractionError::VideoUnavailable(video_id);
        assert!(err.to_string().contains("is unavailable"));
    }

    #[test]
    fn test_download_failed_error() {
        let err = ExtractionError::DownloadFailed(3, "network error".to_string());
        assert_eq!(
            err.to_string(),
            "Download failed after 3 retries: network error"
        );
    }

    #[test]
    fn test_network_timeout_error() {
        let err = ExtractionError::NetworkTimeout(Duration::from_secs(30));
        assert_eq!(err.to_string(), "Network timeout after 30s");
    }
}
