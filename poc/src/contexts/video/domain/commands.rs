use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Command to download a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadVideoCommand {
    /// The video ID to download
    pub video_id: Id<YouTubeVideo>,
}

/// Command to validate a YouTube URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUrlCommand {
    /// The URL to validate
    pub url: String,
}

/// Command to verify video availability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyAvailabilityCommand {
    /// The video ID to verify
    pub video_id: Id<YouTubeVideo>,
    /// The full YouTube URL
    pub url: String,
}

/// Command to handle network timeout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleTimeoutCommand {
    /// The video ID being processed
    pub video_id: Id<YouTubeVideo>,
    /// The operation that timed out
    pub operation: String,
    /// The timeout duration
    pub timeout_secs: u64,
    /// The current retry attempt
    pub retry_attempt: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_video_command() {
        let uuid = uuid::Uuid::new_v4();
        let cmd = DownloadVideoCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
        };
        assert_eq!(cmd.video_id.as_uuid(), uuid);
    }

    #[test]
    fn test_validate_url_command() {
        let cmd = ValidateUrlCommand {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        };
        assert_eq!(cmd.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_verify_availability_command() {
        let uuid = uuid::Uuid::new_v4();
        let cmd = VerifyAvailabilityCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        };
        assert_eq!(cmd.video_id.as_uuid(), uuid);
        assert_eq!(cmd.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_handle_timeout_command() {
        let uuid = uuid::Uuid::new_v4();
        let cmd = HandleTimeoutCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            operation: "download".to_string(),
            timeout_secs: 60,
            retry_attempt: 1,
        };
        assert_eq!(cmd.video_id.as_uuid(), uuid);
        assert_eq!(cmd.operation, "download");
        assert_eq!(cmd.timeout_secs, 60);
        assert_eq!(cmd.retry_attempt, 1);
    }
}
