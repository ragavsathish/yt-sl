use crate::shared::domain::{Id, VideoFrame, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Event emitted when frames have been extracted from a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FramesExtracted {
    /// The video ID frames were extracted from
    pub video_id: Id<YouTubeVideo>,
    /// Total number of frames extracted
    pub total_frames: u32,
    /// Directory where frames are stored
    pub frames_dir: String,
    /// Frame extraction interval in seconds
    pub interval_secs: u64,
    /// Format of extracted frames
    pub format: String,
}

/// Event emitted when a single frame has been extracted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameExtracted {
    /// The frame ID
    pub frame_id: Id<VideoFrame>,
    /// The video ID this frame belongs to
    pub video_id: Id<YouTubeVideo>,
    /// Frame number (sequential)
    pub frame_number: u32,
    /// Frame timestamp in seconds from video start
    pub timestamp: f64,
    /// Path to the frame image file
    pub frame_path: String,
    /// Frame width in pixels
    pub width: u32,
    /// Frame height in pixels
    pub height: u32,
}

/// Event emitted when perceptual hash has been computed for a frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashComputed {
    /// The frame ID hash was computed for
    pub frame_id: Id<VideoFrame>,
    /// The perceptual hash value
    pub hash: String,
    /// Hash algorithm used
    pub algorithm: String,
    /// Hash computation time in milliseconds
    pub computation_time_ms: u64,
}

/// Event emitted when a frame extraction error occurred.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameErrorOccurred {
    /// The frame ID that failed
    pub frame_id: Id<VideoFrame>,
    /// The error message
    pub error: String,
    /// Frame timestamp in seconds
    pub timestamp: f64,
    /// Whether extraction should continue
    pub continue_extraction: bool,
}

/// Event emitted when too many frames have failed extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TooManyFrameErrors {
    /// The video ID being processed
    pub video_id: Id<YouTubeVideo>,
    /// Number of frames that failed
    pub failed_count: u32,
    /// Maximum allowed failures
    pub max_failures: u32,
}

/// Event emitted when frame storage has been optimized.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptimized {
    /// The video ID frames were optimized for
    pub video_id: Id<YouTubeVideo>,
    /// Original total size in bytes
    pub original_size_bytes: u64,
    /// Optimized total size in bytes
    pub optimized_size_bytes: u64,
    /// Number of frames processed
    pub frame_count: u32,
    /// Percentage of space saved
    pub space_saved_percent: f64,
}

/// Event emitted when temporary frames have been cleaned up.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporaryFramesCleaned {
    /// The video ID frames were cleaned for
    pub video_id: Id<YouTubeVideo>,
    /// Directory that was cleaned
    pub directory: String,
    /// Number of frames deleted
    pub frames_deleted: u32,
    /// Space freed in bytes
    pub space_freed_bytes: u64,
}

/// Progress information for frame extraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameExtractionProgress {
    /// Current frame number being processed
    pub current_frame: u32,
    /// Total frames to extract
    pub total_frames: u32,
    /// Current timestamp in seconds
    pub current_timestamp: f64,
    /// Estimated remaining time in seconds
    pub estimated_remaining_secs: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frames_extracted() {
        let uuid = uuid::Uuid::new_v4();
        let event = FramesExtracted {
            video_id: Id::from_uuid(uuid),
            total_frames: 100,
            frames_dir: "/tmp/frames".to_string(),
            interval_secs: 5,
            format: "png".to_string(),
        };
        assert_eq!(event.total_frames, 100);
        assert_eq!(event.interval_secs, 5);
        assert_eq!(event.format, "png");
    }

    #[test]
    fn test_frame_extracted() {
        let video_uuid = uuid::Uuid::new_v4();
        let frame_uuid = uuid::Uuid::new_v4();
        let event = FrameExtracted {
            frame_id: Id::from_uuid(frame_uuid),
            video_id: Id::from_uuid(video_uuid),
            frame_number: 1,
            timestamp: 0.0,
            frame_path: "/tmp/frames/frame_0001.png".to_string(),
            width: 1920,
            height: 1080,
        };
        assert_eq!(event.frame_number, 1);
        assert_eq!(event.timestamp, 0.0);
        assert_eq!(event.width, 1920);
        assert_eq!(event.height, 1080);
    }

    #[test]
    fn test_hash_computed() {
        let frame_uuid = uuid::Uuid::new_v4();
        let event = HashComputed {
            frame_id: Id::from_uuid(frame_uuid),
            hash: "a1b2c3d4e5f6".to_string(),
            algorithm: "average".to_string(),
            computation_time_ms: 50,
        };
        assert_eq!(event.hash, "a1b2c3d4e5f6");
        assert_eq!(event.algorithm, "average");
        assert_eq!(event.computation_time_ms, 50);
    }

    #[test]
    fn test_frame_error_occurred() {
        let frame_uuid = uuid::Uuid::new_v4();
        let event = FrameErrorOccurred {
            frame_id: Id::from_uuid(frame_uuid),
            error: "Corrupt frame".to_string(),
            timestamp: 10.5,
            continue_extraction: true,
        };
        assert_eq!(event.error, "Corrupt frame");
        assert_eq!(event.timestamp, 10.5);
        assert!(event.continue_extraction);
    }

    #[test]
    fn test_too_many_frame_errors() {
        let uuid = uuid::Uuid::new_v4();
        let event = TooManyFrameErrors {
            video_id: Id::from_uuid(uuid),
            failed_count: 15,
            max_failures: 10,
        };
        assert_eq!(event.failed_count, 15);
        assert_eq!(event.max_failures, 10);
    }

    #[test]
    fn test_storage_optimized() {
        let uuid = uuid::Uuid::new_v4();
        let event = StorageOptimized {
            video_id: Id::from_uuid(uuid),
            original_size_bytes: 100_000_000,
            optimized_size_bytes: 50_000_000,
            frame_count: 100,
            space_saved_percent: 50.0,
        };
        assert_eq!(event.original_size_bytes, 100_000_000);
        assert_eq!(event.optimized_size_bytes, 50_000_000);
        assert_eq!(event.space_saved_percent, 50.0);
    }

    #[test]
    fn test_temporary_frames_cleaned() {
        let uuid = uuid::Uuid::new_v4();
        let event = TemporaryFramesCleaned {
            video_id: Id::from_uuid(uuid),
            directory: "/tmp/frames".to_string(),
            frames_deleted: 100,
            space_freed_bytes: 50_000_000,
        };
        assert_eq!(event.frames_deleted, 100);
        assert_eq!(event.space_freed_bytes, 50_000_000);
    }

    #[test]
    fn test_frame_extraction_progress() {
        let progress = FrameExtractionProgress {
            current_frame: 50,
            total_frames: 100,
            current_timestamp: 250.0,
            estimated_remaining_secs: Some(250),
        };
        assert_eq!(progress.current_frame, 50);
        assert_eq!(progress.total_frames, 100);
        assert_eq!(progress.current_timestamp, 250.0);
        assert_eq!(progress.estimated_remaining_secs, Some(250));
    }
}
