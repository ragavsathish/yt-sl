//! Video availability checker infrastructure.
//!
//! This module provides video availability checking functionality as specified in US-VIDEO-03:
//! Verify Video Availability.
//!
//! Features:
//! - Checks if video is accessible and available
//! - Verifies video is not private or deleted
//! - Checks video duration (warns if too short/long)
//! - Uses yt-dlp to verify availability
//! - Handles network errors gracefully

use crate::shared::domain::{DomainResult, ExtractionError, Id, YouTubeVideo};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Video metadata retrieved from YouTube.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct VideoMetadata {
    /// Video title
    pub title: String,
    /// Video duration in seconds
    pub duration: u64,
    /// Video width (resolution)
    pub width: u32,
    /// Video height (resolution)
    pub height: u32,
    /// Video uploader/channel name
    pub uploader: String,
    /// Video upload date
    pub upload_date: String,
    /// View count
    pub view_count: Option<u64>,
    /// Whether the video is age-restricted
    #[serde(default)]
    pub age_restricted: bool,
}

/// Video availability status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AvailabilityStatus {
    /// Video is available and can be downloaded
    Available,
    /// Video is private
    Private,
    /// Video is deleted
    Deleted,
    /// Video is age-restricted
    AgeRestricted,
    /// Video is region-locked
    RegionLocked,
    /// Video is unavailable for other reasons
    Unavailable(String),
}

/// Configuration for the availability checker.
#[derive(Debug, Clone)]
pub struct AvailabilityCheckerConfig {
    /// Timeout for availability check
    pub timeout: Duration,
    /// Maximum video duration in seconds (0 = no limit)
    pub max_duration: u64,
    /// Minimum video duration in seconds (0 = no limit)
    pub min_duration: u64,
}

impl Default for AvailabilityCheckerConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(5),
            max_duration: 4 * 60 * 60, // 4 hours
            min_duration: 0,
        }
    }
}

impl AvailabilityCheckerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn max_duration(mut self, max_duration: u64) -> Self {
        self.max_duration = max_duration;
        self
    }

    pub fn min_duration(mut self, min_duration: u64) -> Self {
        self.min_duration = min_duration;
        self
    }
}

/// Video availability checker.
///
/// Uses yt-dlp to verify if a video is available and retrieve metadata.
#[derive(Debug, Clone)]
pub struct AvailabilityChecker {
    config: AvailabilityCheckerConfig,
}

impl AvailabilityChecker {
    pub fn new() -> Self {
        Self {
            config: AvailabilityCheckerConfig::default(),
        }
    }

    pub fn with_config(config: AvailabilityCheckerConfig) -> Self {
        Self { config }
    }

    /// Checks if a video is available and retrieves its metadata.
    pub async fn check_availability(
        &self,
        _video_id: &Id<YouTubeVideo>,
        url: &str,
    ) -> DomainResult<VideoMetadata> {
        let metadata = self.fetch_metadata(url).await?;

        if self.config.max_duration > 0 && metadata.duration > self.config.max_duration {
            return Err(ExtractionError::VideoTooLong {
                duration: metadata.duration,
                max: self.config.max_duration,
            });
        }

        if self.config.min_duration > 0 && metadata.duration < self.config.min_duration {
            return Err(ExtractionError::InvalidUrl(format!(
                "Video is too short: {} seconds (minimum: {} seconds)",
                metadata.duration, self.config.min_duration
            )));
        }

        if metadata.age_restricted {
            return Err(ExtractionError::VideoAgeRestricted);
        }

        Ok(metadata)
    }

    pub async fn check_status(
        &self,
        _video_id: &Id<YouTubeVideo>,
        url: &str,
    ) -> DomainResult<AvailabilityStatus> {
        match self.fetch_metadata(url).await {
            Ok(metadata) => {
                if metadata.age_restricted {
                    Ok(AvailabilityStatus::AgeRestricted)
                } else {
                    Ok(AvailabilityStatus::Available)
                }
            }
            Err(e) => {
                let error_msg = e.to_string().to_lowercase();

                if error_msg.contains("private") {
                    Ok(AvailabilityStatus::Private)
                } else if error_msg.contains("deleted")
                    || error_msg.contains("not found")
                    || error_msg.contains("unavailable")
                {
                    Ok(AvailabilityStatus::Deleted)
                } else if error_msg.contains("age") || error_msg.contains("sign in") {
                    Ok(AvailabilityStatus::AgeRestricted)
                } else if error_msg.contains("region") || error_msg.contains("country") {
                    Ok(AvailabilityStatus::RegionLocked)
                } else {
                    Ok(AvailabilityStatus::Unavailable(error_msg))
                }
            }
        }
    }

    async fn fetch_metadata(&self, url: &str) -> DomainResult<VideoMetadata> {
        let json_output = timeout(
            self.config.timeout,
            Command::new("yt-dlp")
                .args(["--dump-json", "--no-playlist", "--no-warnings", url])
                .output(),
        )
        .await
        .map_err(|_| ExtractionError::NetworkTimeout(self.config.timeout))?
        .map_err(|e| {
            ExtractionError::ExternalDependencyUnavailable(format!(
                "yt-dlp execution failed: {}",
                e
            ))
        })?;

        if !json_output.status.success() {
            let stderr = String::from_utf8_lossy(&json_output.stderr);
            let stderr_lower = stderr.to_lowercase();

            if stderr_lower.contains("private video") {
                return Err(ExtractionError::VideoPrivate);
            } else if stderr_lower.contains("deleted")
                || stderr_lower.contains("unavailable")
                || stderr_lower.contains("not found")
            {
                return Err(ExtractionError::VideoDeleted);
            } else if stderr_lower.contains("age")
                || stderr_lower.contains("sign in")
                || stderr_lower.contains("age-gate")
            {
                return Err(ExtractionError::VideoAgeRestricted);
            } else if stderr_lower.contains("region")
                || stderr_lower.contains("geo")
                || stderr_lower.contains("country")
            {
                return Err(ExtractionError::VideoRegionLocked);
            }

            return Err(ExtractionError::VideoUnavailable(Id::<YouTubeVideo>::new()));
        }

        let stdout = String::from_utf8_lossy(&json_output.stdout);

        serde_json::from_str(&stdout).map_err(|e| {
            ExtractionError::InternalError(format!("Failed to parse yt-dlp JSON output: {}", e))
        })
    }

    pub async fn check_ytdlp_available(&self) -> DomainResult<()> {
        let result = timeout(
            Duration::from_secs(2),
            Command::new("yt-dlp").args(["--version"]).output(),
        )
        .await
        .map_err(|_| ExtractionError::NetworkTimeout(Duration::from_secs(2)))?
        .map_err(|e| {
            ExtractionError::ExternalDependencyUnavailable(format!(
                "yt-dlp execution failed: {}",
                e
            ))
        })?;

        if !result.status.success() {
            return Err(ExtractionError::ExternalDependencyUnavailable(
                "yt-dlp is not installed or not working".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for AvailabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Mock availability checker for testing.
#[derive(Debug, Clone)]
pub struct MockAvailabilityChecker {
    pub available: bool,
    pub metadata: Option<VideoMetadata>,
    pub error: Option<ExtractionError>,
}

impl MockAvailabilityChecker {
    pub fn new() -> Self {
        Self {
            available: true,
            metadata: Some(VideoMetadata {
                title: "Test Video".to_string(),
                duration: 180,
                width: 1920,
                height: 1080,
                uploader: "Test Channel".to_string(),
                upload_date: "20240101".to_string(),
                view_count: Some(1000),
                age_restricted: false,
            }),
            error: None,
        }
    }

    pub fn with_available(mut self, available: bool) -> Self {
        self.available = available;
        self
    }

    pub fn with_metadata(mut self, metadata: VideoMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    pub fn with_error(mut self, error: ExtractionError) -> Self {
        self.error = Some(error);
        self
    }
}

impl Default for MockAvailabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_availability_checker_config_default() {
        let config = AvailabilityCheckerConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.max_duration, 4 * 60 * 60);
        assert_eq!(config.min_duration, 0);
    }

    #[test]
    fn test_availability_checker_config_builder() {
        let config = AvailabilityCheckerConfig::new()
            .timeout(Duration::from_secs(10))
            .max_duration(3600)
            .min_duration(10);

        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_duration, 3600);
        assert_eq!(config.min_duration, 10);
    }

    #[test]
    fn test_availability_checker_new() {
        let checker = AvailabilityChecker::new();
        assert_eq!(checker.config.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_availability_checker_with_config() {
        let config = AvailabilityCheckerConfig::new().timeout(Duration::from_secs(10));
        let checker = AvailabilityChecker::with_config(config);
        assert_eq!(checker.config.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_mock_availability_checker_default() {
        let checker = MockAvailabilityChecker::new();
        assert!(checker.available);
        assert!(checker.metadata.is_some());
        assert!(checker.error.is_none());
    }

    #[test]
    fn test_mock_availability_checker_builder() {
        let metadata = VideoMetadata {
            title: "Custom Video".to_string(),
            duration: 300,
            width: 1280,
            height: 720,
            uploader: "Custom Channel".to_string(),
            upload_date: "20240102".to_string(),
            view_count: Some(2000),
            age_restricted: false,
        };

        let checker = MockAvailabilityChecker::new()
            .with_available(false)
            .with_metadata(metadata.clone())
            .with_error(ExtractionError::VideoPrivate);

        assert!(!checker.available);
        assert_eq!(checker.metadata.unwrap().title, "Custom Video");
        assert!(matches!(checker.error, Some(ExtractionError::VideoPrivate)));
    }

    #[test]
    fn test_video_metadata() {
        let metadata = VideoMetadata {
            title: "Test Video".to_string(),
            duration: 180,
            width: 1920,
            height: 1080,
            uploader: "Test Channel".to_string(),
            upload_date: "20240101".to_string(),
            view_count: Some(1000),
            age_restricted: false,
        };

        assert_eq!(metadata.title, "Test Video");
        assert_eq!(metadata.duration, 180);
        assert_eq!(metadata.width, 1920);
        assert_eq!(metadata.height, 1080);
        assert_eq!(metadata.uploader, "Test Channel");
        assert_eq!(metadata.upload_date, "20240101");
        assert_eq!(metadata.view_count, Some(1000));
        assert!(!metadata.age_restricted);
    }

    #[test]
    fn test_availability_status_variants() {
        assert_eq!(AvailabilityStatus::Available, AvailabilityStatus::Available);
        assert_eq!(AvailabilityStatus::Private, AvailabilityStatus::Private);
        assert_eq!(AvailabilityStatus::Deleted, AvailabilityStatus::Deleted);
        assert_eq!(
            AvailabilityStatus::AgeRestricted,
            AvailabilityStatus::AgeRestricted
        );
        assert_eq!(
            AvailabilityStatus::RegionLocked,
            AvailabilityStatus::RegionLocked
        );
        assert_ne!(
            AvailabilityStatus::Available,
            AvailabilityStatus::Unavailable("test".to_string())
        );
    }
}
