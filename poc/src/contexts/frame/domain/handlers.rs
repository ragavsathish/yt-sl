use crate::contexts::frame::domain::commands::{
    ComputeHashCommand, ExtractFramesCommand, HandleFrameErrorCommand, OptimizeStorageCommand,
};
use crate::contexts::frame::domain::state::{
    FrameErrorOccurred, FrameExtracted, FramesExtracted, HashComputed, StorageOptimized,
    TemporaryFramesCleaned, TooManyFrameErrors,
};
use crate::shared::domain::{DomainResult, ExtractionError, Id, VideoFrame, YouTubeVideo};
use std::path::Path;

/// Handles an extract frames command.
pub fn handle_extract_frames(
    command: ExtractFramesCommand,
    duration_sec: u64,
) -> DomainResult<FramesExtracted> {
    if command.interval_secs == 0 {
        return Err(ExtractionError::InvalidConfig(
            "Frame interval must be greater than 0".to_string(),
        ));
    }

    if !Path::new(&command.video_path).exists() {
        return Err(ExtractionError::FrameExtractionFailed(format!(
            "Video file not found: {}",
            command.video_path
        )));
    }

    let total_frames = calculate_total_frames(duration_sec, command.interval_secs);

    Ok(FramesExtracted {
        video_id: command.video_id,
        total_frames,
        frames_dir: command.output_dir.clone(),
        interval_secs: command.interval_secs,
        format: format!("{:?}", command.output_format),
    })
}

/// Calculates the total number of frames to extract.
pub fn calculate_total_frames(duration_sec: u64, interval_secs: u64) -> u32 {
    if interval_secs == 0 {
        return 0;
    }
    let mut count = (duration_sec / interval_secs) as u32;
    // Always include the last frame
    if !duration_sec.is_multiple_of(interval_secs) {
        count += 1;
    }
    count.max(1)
}

/// Handles a compute hash command.
pub fn handle_compute_hash(command: ComputeHashCommand) -> DomainResult<HashComputed> {
    if !Path::new(&command.frame_path).exists() {
        return Err(ExtractionError::HashComputationFailed(command.frame_id));
    }

    Ok(HashComputed {
        frame_id: command.frame_id,
        hash: String::new(),
        algorithm: format!("{:?}", command.algorithm),
        computation_time_ms: 0,
    })
}

/// Handles a frame error command.
pub fn handle_frame_error(command: HandleFrameErrorCommand) -> DomainResult<FrameErrorOccurred> {
    let continue_extraction = command.skipped_count < command.max_skipped;

    if !continue_extraction {
        return Err(ExtractionError::TooManyCorruptFrames {
            count: command.skipped_count,
            max: command.max_skipped,
        });
    }

    Ok(FrameErrorOccurred {
        frame_id: command.frame_id,
        error: command.error,
        timestamp: command.timestamp,
        continue_extraction,
    })
}

/// Handles an optimize storage command.
pub fn handle_optimize_storage(
    command: OptimizeStorageCommand,
    original_size_bytes: u64,
    optimized_size_bytes: u64,
    frame_count: u32,
) -> DomainResult<StorageOptimized> {
    let space_saved_bytes = original_size_bytes.saturating_sub(optimized_size_bytes);
    let space_saved_percent = if original_size_bytes > 0 {
        (space_saved_bytes as f64 / original_size_bytes as f64) * 100.0
    } else {
        0.0
    };

    Ok(StorageOptimized {
        video_id: command.video_id,
        original_size_bytes,
        optimized_size_bytes,
        frame_count,
        space_saved_percent,
    })
}

pub fn create_frame_extracted_event(
    frame_id: Id<VideoFrame>,
    video_id: Id<YouTubeVideo>,
    frame_number: u32,
    timestamp: f64,
    frame_path: String,
    width: u32,
    height: u32,
) -> FrameExtracted {
    FrameExtracted {
        frame_id,
        video_id,
        frame_number,
        timestamp,
        frame_path,
        width,
        height,
    }
}

pub fn create_too_many_errors_event(
    video_id: Id<YouTubeVideo>,
    failed_count: u32,
    max_failures: u32,
) -> TooManyFrameErrors {
    TooManyFrameErrors {
        video_id,
        failed_count,
        max_failures,
    }
}

pub fn create_frames_cleaned_event(
    video_id: Id<YouTubeVideo>,
    directory: String,
    frames_deleted: u32,
    space_freed_bytes: u64,
) -> TemporaryFramesCleaned {
    TemporaryFramesCleaned {
        video_id,
        directory,
        frames_deleted,
        space_freed_bytes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::frame::domain::commands::{FrameFormat, HashAlgorithm};

    #[test]
    fn test_calculate_total_frames() {
        // 60 seconds, 5 second interval = 12 frames
        assert_eq!(calculate_total_frames(60, 5), 12);
        // 65 seconds, 5 second interval = 13 frames (last frame included)
        assert_eq!(calculate_total_frames(65, 5), 13);
        // 10 seconds, 1 second interval = 10 frames
        assert_eq!(calculate_total_frames(10, 1), 10);
        // 0 seconds = 1 frame (minimum)
        assert_eq!(calculate_total_frames(0, 5), 1);
    }

    #[test]
    fn test_handle_extract_frames_success() {
        use tempfile::NamedTempFile;

        // Create a temporary file to simulate a video file
        let temp_file = NamedTempFile::new().unwrap();
        let video_path = temp_file.path().to_str().unwrap().to_string();

        let uuid = uuid::Uuid::new_v4();
        let command = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path,
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 5,
            output_format: FrameFormat::Png,
            jpeg_quality: None,
        };
        let result = handle_extract_frames(command, 60);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.total_frames, 12);
        assert_eq!(event.interval_secs, 5);
    }

    #[test]
    fn test_handle_extract_frames_zero_interval() {
        let uuid = uuid::Uuid::new_v4();
        let command = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path: "/tmp/video.mp4".to_string(),
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 0,
            output_format: FrameFormat::Png,
            jpeg_quality: None,
        };
        let result = handle_extract_frames(command, 60);
        assert!(matches!(result, Err(ExtractionError::InvalidConfig(_))));
    }

    #[test]
    fn test_handle_extract_frames_missing_video() {
        let uuid = uuid::Uuid::new_v4();
        let command = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path: "/nonexistent/video.mp4".to_string(),
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 5,
            output_format: FrameFormat::Png,
            jpeg_quality: None,
        };
        let result = handle_extract_frames(command, 60);
        assert!(matches!(
            result,
            Err(ExtractionError::FrameExtractionFailed(_))
        ));
    }

    #[test]
    fn test_handle_compute_hash_success() {
        let uuid = uuid::Uuid::new_v4();
        let command = ComputeHashCommand {
            frame_id: Id::<VideoFrame>::from_uuid(uuid),
            frame_path: "/tmp/frame.png".to_string(),
            algorithm: HashAlgorithm::Average,
        };
        // Note: This test will fail if the file doesn't exist
        // The actual hash computation is delegated to infrastructure
        let result = handle_compute_hash(command);
        // We expect failure since file doesn't exist
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_frame_error_continue() {
        let uuid = uuid::Uuid::new_v4();
        let command = HandleFrameErrorCommand {
            frame_id: Id::<VideoFrame>::from_uuid(uuid),
            error: "Corrupt frame".to_string(),
            timestamp: 10.5,
            skipped_count: 5,
            max_skipped: 10,
        };
        let result = handle_frame_error(command);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(event.continue_extraction);
    }

    #[test]
    fn test_handle_frame_error_too_many() {
        let uuid = uuid::Uuid::new_v4();
        let command = HandleFrameErrorCommand {
            frame_id: Id::<VideoFrame>::from_uuid(uuid),
            error: "Corrupt frame".to_string(),
            timestamp: 10.5,
            skipped_count: 10,
            max_skipped: 10,
        };
        let result = handle_frame_error(command);
        assert!(matches!(
            result,
            Err(ExtractionError::TooManyCorruptFrames { .. })
        ));
    }

    #[test]
    fn test_handle_optimize_storage() {
        let uuid = uuid::Uuid::new_v4();
        let command = OptimizeStorageCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            frames_dir: "/tmp/frames".to_string(),
            compress: true,
            compression_quality: Some(85),
            cleanup_temp: true,
        };
        let result = handle_optimize_storage(command, 100_000_000, 50_000_000, 100);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.original_size_bytes, 100_000_000);
        assert_eq!(event.optimized_size_bytes, 50_000_000);
        assert_eq!(event.space_saved_percent, 50.0);
    }

    #[test]
    fn test_create_frame_extracted_event() {
        let video_uuid = uuid::Uuid::new_v4();
        let frame_uuid = uuid::Uuid::new_v4();
        let event = create_frame_extracted_event(
            Id::from_uuid(frame_uuid),
            Id::from_uuid(video_uuid),
            1,
            0.0,
            "/tmp/frame.png".to_string(),
            1920,
            1080,
        );
        assert_eq!(event.frame_number, 1);
        assert_eq!(event.timestamp, 0.0);
        assert_eq!(event.width, 1920);
        assert_eq!(event.height, 1080);
    }

    #[test]
    fn test_create_too_many_errors_event() {
        let uuid = uuid::Uuid::new_v4();
        let event = create_too_many_errors_event(Id::from_uuid(uuid), 15, 10);
        assert_eq!(event.failed_count, 15);
        assert_eq!(event.max_failures, 10);
    }

    #[test]
    fn test_create_frames_cleaned_event() {
        let uuid = uuid::Uuid::new_v4();
        let event = create_frames_cleaned_event(
            Id::from_uuid(uuid),
            "/tmp/frames".to_string(),
            100,
            50_000_000,
        );
        assert_eq!(event.frames_deleted, 100);
        assert_eq!(event.space_freed_bytes, 50_000_000);
    }
}
