use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Event emitted when a YouTube URL has been validated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoUrlValidated {
    /// The validated URL
    pub url: String,
    /// The extracted video ID
    pub video_id: Id<YouTubeVideo>,
}

/// Event emitted when a video's availability has been verified.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoAvailabilityVerified {
    /// The video ID that was verified
    pub video_id: Id<YouTubeVideo>,
    /// The video title
    pub title: String,
    /// The video duration in seconds
    pub duration: u64,
    /// The video width
    pub width: u32,
    /// The video height
    pub height: u32,
    /// The uploader/channel name
    pub uploader: String,
    /// The upload date
    pub upload_date: String,
    /// Age limit for the video (0 if none)
    pub age_limit: u8,
}

/// Event emitted when a video has been downloaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDownloaded {
    /// The video ID that was downloaded
    pub video_id: Id<YouTubeVideo>,
    /// Path to the downloaded video file
    pub path: String,
    /// Video duration in seconds
    pub duration_sec: u64,
    /// Video width
    pub width: u32,
    /// Video height
    pub height: u32,
    /// File size in bytes
    pub file_size: u64,
}

/// Event emitted when a network timeout occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTimeoutOccurred {
    /// The video ID being processed
    pub video_id: Id<YouTubeVideo>,
    /// The operation that timed out
    pub operation: String,
    /// The timeout duration in seconds
    pub timeout_secs: u64,
    /// The current retry attempt
    pub retry_attempt: u8,
    /// Maximum retry attempts
    pub max_retries: u8,
}

/// Event emitted when a download retry is initiated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRetryInitiated {
    /// The video ID being downloaded
    pub video_id: Id<YouTubeVideo>,
    /// The retry attempt number
    pub attempt: u8,
    /// The backoff duration in seconds
    pub backoff_secs: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_url_validated() {
        let uuid = uuid::Uuid::new_v4();
        let event = VideoUrlValidated {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            video_id: Id::from_uuid(uuid),
        };
        assert_eq!(event.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        assert_eq!(event.video_id.as_uuid(), uuid);
    }

    #[test]
    fn test_video_availability_verified() {
        let uuid = uuid::Uuid::new_v4();
        let event = VideoAvailabilityVerified {
            video_id: Id::from_uuid(uuid),
            title: "Test Video".to_string(),
            duration: 180,
            width: 1920,
            height: 1080,
            uploader: "Test Channel".to_string(),
            upload_date: "20240101".to_string(),
            age_limit: 0,
        };
        assert_eq!(event.title, "Test Video");
        assert_eq!(event.duration, 180);
        assert_eq!(event.width, 1920);
        assert_eq!(event.height, 1080);
        assert_eq!(event.uploader, "Test Channel");
        assert_eq!(event.upload_date, "20240101");
        assert_eq!(event.age_limit, 0);
    }

    #[test]
    fn test_video_downloaded() {
        let uuid = uuid::Uuid::new_v4();
        let event = VideoDownloaded {
            video_id: Id::from_uuid(uuid),
            path: "/tmp/video.mp4".to_string(),
            duration_sec: 180,
            width: 1920,
            height: 1080,
            file_size: 10 * 1024 * 1024,
        };
        assert_eq!(event.path, "/tmp/video.mp4");
        assert_eq!(event.duration_sec, 180);
        assert_eq!(event.width, 1920);
        assert_eq!(event.height, 1080);
        assert_eq!(event.file_size, 10 * 1024 * 1024);
    }

    #[test]
    fn test_network_timeout_occurred() {
        let uuid = uuid::Uuid::new_v4();
        let event = NetworkTimeoutOccurred {
            video_id: Id::from_uuid(uuid),
            operation: "download".to_string(),
            timeout_secs: 60,
            retry_attempt: 1,
            max_retries: 3,
        };
        assert_eq!(event.operation, "download");
        assert_eq!(event.timeout_secs, 60);
        assert_eq!(event.retry_attempt, 1);
        assert_eq!(event.max_retries, 3);
    }

    #[test]
    fn test_download_retry_initiated() {
        let uuid = uuid::Uuid::new_v4();
        let event = DownloadRetryInitiated {
            video_id: Id::from_uuid(uuid),
            attempt: 2,
            backoff_secs: 2,
        };
        assert_eq!(event.attempt, 2);
        assert_eq!(event.backoff_secs, 2);
    }
}
