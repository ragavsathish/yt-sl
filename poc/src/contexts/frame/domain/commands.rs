use crate::shared::domain::{Id, VideoFrame, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Command to extract frames from a video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractFramesCommand {
    /// The video ID to extract frames from
    pub video_id: Id<YouTubeVideo>,
    /// Path to the video file
    pub video_path: String,
    /// Output directory for frames
    pub output_dir: String,
    /// Frame extraction interval in seconds
    pub interval_secs: u64,
    /// Output format (JPEG, PNG)
    pub output_format: FrameFormat,
    /// JPEG quality (1-100) for JPEG format
    pub jpeg_quality: Option<u8>,
}

/// Command to compute perceptual hash for a frame.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeHashCommand {
    /// The frame ID to compute hash for
    pub frame_id: Id<VideoFrame>,
    /// Path to the frame image file
    pub frame_path: String,
    /// Hash algorithm to use
    pub algorithm: HashAlgorithm,
}

/// Command to handle frame extraction errors.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandleFrameErrorCommand {
    /// The frame ID that failed
    pub frame_id: Id<VideoFrame>,
    /// The error that occurred
    pub error: String,
    /// Frame timestamp in seconds
    pub timestamp: f64,
    /// Current number of skipped frames
    pub skipped_count: u32,
    /// Maximum allowed skipped frames
    pub max_skipped: u32,
}

/// Command to optimize frame storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizeStorageCommand {
    /// The video ID for frames to optimize
    pub video_id: Id<YouTubeVideo>,
    /// Directory containing frames
    pub frames_dir: String,
    /// Whether to compress frames
    pub compress: bool,
    /// JPEG quality for compression (1-100)
    pub compression_quality: Option<u8>,
    /// Whether to delete temporary frames
    pub cleanup_temp: bool,
}

/// Frame output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameFormat {
    /// JPEG format (lossy compression, smaller files)
    Jpeg,
    /// PNG format (lossless compression, larger files)
    Png,
}

/// Perceptual hash algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashAlgorithm {
    /// Average hash algorithm
    Average,
    /// Difference hash algorithm
    Difference,
    /// Perceptual hash algorithm
    Perceptual,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_frames_command() {
        let uuid = uuid::Uuid::new_v4();
        let cmd = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path: "/tmp/video.mp4".to_string(),
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 5,
            output_format: FrameFormat::Png,
            jpeg_quality: None,
        };
        assert_eq!(cmd.video_id.as_uuid(), uuid);
        assert_eq!(cmd.interval_secs, 5);
        assert_eq!(cmd.output_format, FrameFormat::Png);
    }

    #[test]
    fn test_compute_hash_command() {
        let uuid = uuid::Uuid::new_v4();
        let cmd = ComputeHashCommand {
            frame_id: Id::<VideoFrame>::from_uuid(uuid),
            frame_path: "/tmp/frame.png".to_string(),
            algorithm: HashAlgorithm::Average,
        };
        assert_eq!(cmd.frame_id.as_uuid(), uuid);
        assert_eq!(cmd.algorithm, HashAlgorithm::Average);
    }

    #[test]
    fn test_handle_frame_error_command() {
        let uuid = uuid::Uuid::new_v4();
        let cmd = HandleFrameErrorCommand {
            frame_id: Id::<VideoFrame>::from_uuid(uuid),
            error: "Corrupt frame".to_string(),
            timestamp: 10.5,
            skipped_count: 1,
            max_skipped: 10,
        };
        assert_eq!(cmd.frame_id.as_uuid(), uuid);
        assert_eq!(cmd.timestamp, 10.5);
        assert_eq!(cmd.skipped_count, 1);
    }

    #[test]
    fn test_optimize_storage_command() {
        let uuid = uuid::Uuid::new_v4();
        let cmd = OptimizeStorageCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            frames_dir: "/tmp/frames".to_string(),
            compress: true,
            compression_quality: Some(85),
            cleanup_temp: true,
        };
        assert_eq!(cmd.video_id.as_uuid(), uuid);
        assert!(cmd.compress);
        assert_eq!(cmd.compression_quality, Some(85));
    }

    #[test]
    fn test_frame_format() {
        assert_eq!(FrameFormat::Jpeg, FrameFormat::Jpeg);
        assert_eq!(FrameFormat::Png, FrameFormat::Png);
    }

    #[test]
    fn test_hash_algorithm() {
        assert_eq!(HashAlgorithm::Average, HashAlgorithm::Average);
        assert_eq!(HashAlgorithm::Difference, HashAlgorithm::Difference);
        assert_eq!(HashAlgorithm::Perceptual, HashAlgorithm::Perceptual);
    }
}
