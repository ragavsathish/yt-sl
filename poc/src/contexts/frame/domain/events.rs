use crate::contexts::frame::domain::commands::{ExtractFramesCommand, HashAlgorithm};
use crate::contexts::frame::domain::state::FrameExtracted;
use crate::shared::domain::{DomainResult, ExtractionError, Id, VideoFrame, YouTubeVideo};
use std::path::Path;

/// Validates frame extraction parameters.
///
/// This function provides validation for frame extraction as specified in US-FRAME-01:
/// Extract Frames at Intervals.
///
/// # Arguments
///
/// * `command` - The extract frames command
///
/// # Returns
///
/// Ok(()) if parameters are valid, error otherwise
pub fn validate_extraction_params(command: &ExtractFramesCommand) -> DomainResult<()> {
    if command.interval_secs < 1 {
        return Err(ExtractionError::InvalidConfig(
            "Frame interval must be at least 1 second".to_string(),
        ));
    }

    if command.interval_secs > 60 {
        return Err(ExtractionError::InvalidConfig(
            "Frame interval must be at most 60 seconds".to_string(),
        ));
    }

    if let Some(quality) = command.jpeg_quality {
        if !(1..=100).contains(&quality) {
            return Err(ExtractionError::InvalidConfig(
                "JPEG quality must be between 1 and 100".to_string(),
            ));
        }
    }

    Ok(())
}

/// Generates frame filename with session ID and frame number.
///
/// This function provides frame naming as specified in US-FRAME-04:
/// Optimize Frame Storage.
///
/// # Arguments
///
/// * `session_id` - The session ID
/// * `frame_number` - The frame number
/// * `format` - The output format
///
/// # Returns
///
/// A formatted frame filename
pub fn generate_frame_filename(
    session_id: Id<YouTubeVideo>,
    frame_number: u32,
    format: &str,
) -> String {
    format!("{}_frame_{:04}.{}", session_id, frame_number, format)
}

/// Generates frame output path.
///
/// # Arguments
///
/// * `output_dir` - The output directory
/// * `session_id` - The session ID
/// * `frame_number` - The frame number
/// * `format` - The output format
///
/// # Returns
///
/// A formatted frame output path
pub fn generate_frame_path(
    output_dir: &str,
    session_id: &Id<YouTubeVideo>,
    frame_number: u32,
    format: &str,
) -> String {
    let filename = generate_frame_filename(session_id.clone(), frame_number, format);
    Path::new(output_dir)
        .join(filename)
        .to_string_lossy()
        .to_string()
}

/// Creates a FrameExtracted event from frame extraction data.
///
/// This function provides event creation as specified in US-FRAME-01:
/// Extract Frames at Intervals.
///
/// # Arguments
///
/// * `frame_id` - The frame ID
/// * `video_id` - The video ID
/// * `frame_number` - The frame number
/// * `timestamp` - The frame timestamp in seconds
/// * `frame_path` - The path to the frame file
/// * `width` - The frame width in pixels
/// * `height` - The frame height in pixels
///
/// # Returns
///
/// A FrameExtracted event
pub fn create_extracted_event(
    frame_id: Id<VideoFrame>,
    video_id: &Id<YouTubeVideo>,
    frame_number: u32,
    timestamp: f64,
    frame_path: String,
    width: u32,
    height: u32,
) -> FrameExtracted {
    FrameExtracted {
        frame_id,
        video_id: video_id.clone(),
        frame_number,
        timestamp,
        frame_path,
        width,
        height,
    }
}

/// Validates hash computation parameters.
///
/// This function provides validation for hash computation as specified in US-FRAME-02:
/// Compute Perceptual Hash.
///
/// # Arguments
///
/// * `algorithm` - The hash algorithm to use
/// * `hash_size` - The hash size in pixels (width and height)
///
/// # Returns
///
/// Ok(()) if parameters are valid, error otherwise
pub fn validate_hash_params(algorithm: HashAlgorithm, hash_size: u32) -> DomainResult<()> {
    if hash_size < 8 {
        return Err(ExtractionError::InvalidConfig(
            "Hash size must be at least 8x8 pixels".to_string(),
        ));
    }

    if hash_size > 64 {
        return Err(ExtractionError::InvalidConfig(
            "Hash size must be at most 64x64 pixels".to_string(),
        ));
    }

    // Additional validation for specific algorithms
    match algorithm {
        HashAlgorithm::Average | HashAlgorithm::Difference | HashAlgorithm::Perceptual => {
            // These algorithms work with any hash size within the range
            Ok(())
        }
    }
}

/// Calculates expected number of frames based on video duration and interval.
///
/// This function provides frame count estimation as specified in US-FRAME-01:
/// Extract Frames at Intervals.
///
/// # Arguments
///
/// * `duration_secs` - The video duration in seconds
/// * `interval_secs` - The frame extraction interval in seconds
///
/// # Returns
///
/// The expected number of frames
pub fn calculate_expected_frames(duration_secs: u64, interval_secs: u64) -> u32 {
    if interval_secs == 0 {
        return 0;
    }
    // Add 1 to include the first frame at timestamp 0
    ((duration_secs / interval_secs) + 1) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::frame::domain::commands::FrameFormat;

    #[test]
    fn test_validate_extraction_params_valid() {
        let uuid = uuid::Uuid::new_v4();
        let command = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path: "/tmp/video.mp4".to_string(),
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 5,
            output_format: FrameFormat::Png,
            jpeg_quality: None,
        };
        assert!(validate_extraction_params(&command).is_ok());
    }

    #[test]
    fn test_validate_extraction_params_interval_too_small() {
        let uuid = uuid::Uuid::new_v4();
        let command = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path: "/tmp/video.mp4".to_string(),
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 0,
            output_format: FrameFormat::Png,
            jpeg_quality: None,
        };
        assert!(matches!(
            validate_extraction_params(&command),
            Err(ExtractionError::InvalidConfig(_))
        ));
    }

    #[test]
    fn test_validate_extraction_params_interval_too_large() {
        let uuid = uuid::Uuid::new_v4();
        let command = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path: "/tmp/video.mp4".to_string(),
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 61,
            output_format: FrameFormat::Png,
            jpeg_quality: None,
        };
        assert!(matches!(
            validate_extraction_params(&command),
            Err(ExtractionError::InvalidConfig(_))
        ));
    }

    #[test]
    fn test_validate_extraction_params_jpeg_quality_invalid() {
        let uuid = uuid::Uuid::new_v4();
        let command = ExtractFramesCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
            video_path: "/tmp/video.mp4".to_string(),
            output_dir: "/tmp/frames".to_string(),
            interval_secs: 5,
            output_format: FrameFormat::Jpeg,
            jpeg_quality: Some(150),
        };
        assert!(matches!(
            validate_extraction_params(&command),
            Err(ExtractionError::InvalidConfig(_))
        ));
    }

    #[test]
    fn test_generate_frame_filename() {
        let uuid = uuid::Uuid::new_v4();
        let session_id = Id::<YouTubeVideo>::from_uuid(uuid);
        let filename = generate_frame_filename(session_id.clone(), 1, "png");
        assert!(filename.contains(&session_id.to_string()));
        assert!(filename.contains("frame_0001"));
        assert!(filename.ends_with(".png"));
    }

    #[test]
    fn test_generate_frame_path() {
        let uuid = uuid::Uuid::new_v4();
        let session_id = Id::<YouTubeVideo>::from_uuid(uuid);
        let path = generate_frame_path("/tmp/frames", &session_id, 1, "png");
        assert!(path.starts_with("/tmp/frames"));
        assert!(path.contains(&session_id.to_string()));
        assert!(path.contains("frame_0001"));
        assert!(path.ends_with(".png"));
    }

    #[test]
    fn test_calculate_expected_frames() {
        assert_eq!(calculate_expected_frames(60, 5), 13);
        assert_eq!(calculate_expected_frames(65, 5), 14);
        assert_eq!(calculate_expected_frames(10, 1), 11);
    }
}
