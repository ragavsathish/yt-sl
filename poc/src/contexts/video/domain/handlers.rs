use crate::contexts::video::domain::commands::{
    DownloadVideoCommand, HandleTimeoutCommand, ValidateUrlCommand, VerifyAvailabilityCommand,
};
use crate::contexts::video::domain::events::validate_video_url;
use crate::contexts::video::domain::state::{
    DownloadRetryInitiated, NetworkTimeoutOccurred, VideoAvailabilityVerified, VideoDownloaded,
};
use crate::shared::domain::{DomainResult, ExtractionError, Id, YouTubeVideo};
use std::time::Duration;

/// Handles a download video command.
///
/// This function provides video download functionality as specified in US-VIDEO-02:
/// Download Video.
///
/// # Arguments
///
/// * `command` - The download command
/// * `output_path` - The output path for the downloaded video
/// * `duration_sec` - The video duration in seconds
///
/// # Returns
///
/// A VideoDownloaded event
///
/// # Errors
///
/// Returns an error if the download fails
pub fn handle_download_video(
    command: DownloadVideoCommand,
    output_path: &str,
    duration_sec: u64,
) -> DomainResult<VideoDownloaded> {
    if output_path.is_empty() {
        return Err(ExtractionError::DownloadFailed(
            0,
            "Output path is empty".to_string(),
        ));
    }

    let path = format!("{}/{}.mp4", output_path, command.video_id);
    Ok(VideoDownloaded {
        video_id: command.video_id,
        path,
        duration_sec,
        width: 0,
        height: 0,
        file_size: 0,
    })
}

/// Handles a validate URL command.
///
/// This function provides URL validation functionality as specified in US-VIDEO-01:
/// Validate YouTube URL.
///
/// # Arguments
///
/// * `command` - The validate URL command
///
/// # Returns
///
/// A VideoUrlValidated event
///
/// # Errors
///
/// Returns an error if the URL is invalid
pub fn handle_validate_url(
    command: ValidateUrlCommand,
) -> DomainResult<crate::contexts::video::domain::state::VideoUrlValidated> {
    let validated = validate_video_url(&command.url)?;
    Ok(crate::contexts::video::domain::state::VideoUrlValidated {
        url: validated.url,
        video_id: validated.video_id,
    })
}

/// Handles a verify availability command.
///
/// This function provides video availability checking functionality as specified in US-VIDEO-03:
/// Verify Video Availability.
///
/// # Arguments
///
/// * `command` - The verify availability command
/// * `metadata` - The video metadata from availability check
///
/// # Returns
///
/// A VideoAvailabilityVerified event
///
/// # Errors
///
/// Returns an error if the video is unavailable
pub fn handle_verify_availability(
    command: VerifyAvailabilityCommand,
    metadata: crate::contexts::video::infrastructure::VideoMetadata,
) -> DomainResult<VideoAvailabilityVerified> {
    Ok(VideoAvailabilityVerified {
        video_id: command.video_id,
        title: metadata.title,
        duration: metadata.duration,
        width: metadata.width,
        height: metadata.height,
        uploader: metadata.uploader,
        upload_date: metadata.upload_date,
        age_restricted: metadata.age_restricted,
    })
}

/// Handles a network timeout command.
///
/// This function provides network timeout handling functionality as specified in US-VIDEO-04:
/// Handle Network Timeouts.
///
/// # Arguments
///
/// * `command` - The handle timeout command
/// * `max_retries` - Maximum number of retry attempts
///
/// # Returns
///
/// A NetworkTimeoutOccurred event
///
/// # Errors
///
/// Returns an error if all retries are exhausted
pub fn handle_timeout(
    command: HandleTimeoutCommand,
    max_retries: u8,
) -> DomainResult<NetworkTimeoutOccurred> {
    if command.retry_attempt >= max_retries {
        return Err(ExtractionError::NetworkTimeout(Duration::from_secs(
            command.timeout_secs,
        )));
    }

    Ok(NetworkTimeoutOccurred {
        video_id: command.video_id,
        operation: command.operation,
        timeout_secs: command.timeout_secs,
        retry_attempt: command.retry_attempt,
        max_retries,
    })
}

/// Calculates the backoff duration for a retry attempt.
///
/// This function implements exponential backoff as specified in US-VIDEO-04:
/// Handle Network Timeouts.
///
/// # Arguments
///
/// * `attempt` - The current retry attempt (0-indexed)
/// * `initial_backoff` - The initial backoff duration
/// * `max_backoff` - The maximum backoff duration
///
/// # Returns
///
/// The backoff duration for the retry attempt
pub fn calculate_backoff(
    attempt: u8,
    initial_backoff: Duration,
    max_backoff: Duration,
) -> Duration {
    let base = initial_backoff.as_secs_f64();
    let multiplier = 2_f64.powi(attempt as i32);
    let backoff_secs = base * multiplier;

    let max_secs = max_backoff.as_secs_f64();
    let backoff_secs = backoff_secs.min(max_secs);

    // Add jitter to avoid thundering herd
    let jitter = backoff_secs * 0.1;
    let backoff_secs = backoff_secs + (rand::random::<f64>() * jitter);

    Duration::from_secs_f64(backoff_secs)
}

/// Creates a download retry initiated event.
///
/// # Arguments
///
/// * `video_id` - The video ID being downloaded
/// * `attempt` - The retry attempt number
/// * `backoff_secs` - The backoff duration in seconds
///
/// # Returns
///
/// A DownloadRetryInitiated event
pub fn create_retry_event(
    video_id: Id<YouTubeVideo>,
    attempt: u8,
    backoff_secs: u64,
) -> DownloadRetryInitiated {
    DownloadRetryInitiated {
        video_id,
        attempt,
        backoff_secs,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handle_download_video_success() {
        let command = DownloadVideoCommand {
            video_id: Id::new(),
        };
        let result = handle_download_video(command, "/tmp", 180);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert!(event.path.contains("/tmp/"));
        assert_eq!(event.duration_sec, 180);
    }

    #[test]
    fn test_handle_download_video_empty_path() {
        let command = DownloadVideoCommand {
            video_id: Id::new(),
        };
        let result = handle_download_video(command, "", 0);
        assert!(matches!(result, Err(ExtractionError::DownloadFailed(_, _))));
    }

    #[test]
    fn test_handle_validate_url_success() {
        let command = ValidateUrlCommand {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        };
        let result = handle_validate_url(command);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(event.video_id, expected_id);
    }

    #[test]
    fn test_handle_validate_url_invalid() {
        let command = ValidateUrlCommand {
            url: "https://example.com/video".to_string(),
        };
        let result = handle_validate_url(command);
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_handle_verify_availability() {
        let uuid = uuid::Uuid::new_v4();
        let command = VerifyAvailabilityCommand {
            video_id: Id::from_uuid(uuid),
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        };
        let metadata = crate::contexts::video::infrastructure::VideoMetadata {
            title: "Test Video".to_string(),
            duration: 180,
            width: 1920,
            height: 1080,
            uploader: "Test Channel".to_string(),
            upload_date: "20240101".to_string(),
            view_count: Some(1000),
            age_restricted: false,
        };
        let result = handle_verify_availability(command, metadata);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.title, "Test Video");
        assert_eq!(event.duration, 180);
        assert_eq!(event.width, 1920);
        assert_eq!(event.height, 1080);
    }

    #[test]
    fn test_handle_timeout_within_retries() {
        let uuid = uuid::Uuid::new_v4();
        let command = HandleTimeoutCommand {
            video_id: Id::from_uuid(uuid),
            operation: "download".to_string(),
            timeout_secs: 60,
            retry_attempt: 1,
        };
        let result = handle_timeout(command, 3);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.operation, "download");
        assert_eq!(event.timeout_secs, 60);
        assert_eq!(event.retry_attempt, 1);
        assert_eq!(event.max_retries, 3);
    }

    #[test]
    fn test_handle_timeout_exhausted() {
        let uuid = uuid::Uuid::new_v4();
        let command = HandleTimeoutCommand {
            video_id: Id::from_uuid(uuid),
            operation: "download".to_string(),
            timeout_secs: 60,
            retry_attempt: 3,
        };
        let result = handle_timeout(command, 3);
        assert!(matches!(result, Err(ExtractionError::NetworkTimeout(_))));
    }

    #[test]
    fn test_calculate_backoff() {
        let initial = Duration::from_secs(1);
        let max = Duration::from_secs(30);

        // First retry: ~1 second
        let backoff = calculate_backoff(0, initial, max);
        assert!(backoff >= Duration::from_secs(1));
        assert!(backoff < Duration::from_secs(2));

        // Second retry: ~2 seconds
        let backoff = calculate_backoff(1, initial, max);
        assert!(backoff >= Duration::from_secs(2));
        assert!(backoff < Duration::from_secs(3));

        // Third retry: ~4 seconds
        let backoff = calculate_backoff(2, initial, max);
        assert!(backoff >= Duration::from_secs(4));
        assert!(backoff < Duration::from_secs(5));
    }

    #[test]
    fn test_calculate_backoff_max() {
        let initial = Duration::from_secs(10);
        let max = Duration::from_secs(20);

        // Should be capped at max_backoff
        let backoff = calculate_backoff(10, initial, max);
        assert!(backoff >= Duration::from_secs(20));
        assert!(backoff < Duration::from_secs(25));
    }

    #[test]
    fn test_create_retry_event() {
        let uuid = uuid::Uuid::new_v4();
        let event = create_retry_event(Id::from_uuid(uuid), 2, 4);
        assert_eq!(event.attempt, 2);
        assert_eq!(event.backoff_secs, 4);
    }
}
