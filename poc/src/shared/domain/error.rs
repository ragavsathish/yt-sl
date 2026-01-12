use crate::shared::domain::Id;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;
use thiserror::Error;

/// Error category for user-friendly error classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCategory {
    /// Configuration-related errors
    Configuration,
    /// Network-related errors
    Network,
    /// File system I/O errors
    FileSystem,
    /// External dependency errors
    ExternalDependency,
    /// Processing errors
    Processing,
    /// Validation errors
    Validation,
    /// Memory-related errors
    Memory,
    /// Unknown or uncategorized errors
    Unknown,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Configuration => write!(f, "Configuration"),
            ErrorCategory::Network => write!(f, "Network"),
            ErrorCategory::FileSystem => write!(f, "File System"),
            ErrorCategory::ExternalDependency => write!(f, "External Dependency"),
            ErrorCategory::Processing => write!(f, "Processing"),
            ErrorCategory::Validation => write!(f, "Validation"),
            ErrorCategory::Memory => write!(f, "Memory"),
            ErrorCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Main error type for the extraction process.
///
/// This error type provides user-friendly messages with actionable guidance
/// as specified in US-ERR-01: Display User-Friendly Error Messages.
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

    // New error variants for user-friendly error messages (US-ERR-01)
    #[error("Output directory does not exist: {0}")]
    OutputDirectoryNotFound(String),

    #[error("Parent directory does not exist: {0}")]
    ParentDirectoryNotFound(String),

    #[error("Insufficient disk space: required {0}MB, available {1}MB")]
    InsufficientDiskSpace(u64, u64),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("External dependency check failed: {name} - {reason}")]
    DependencyCheckFailed { name: String, reason: String },

    #[error("External dependency version mismatch: {name} - required {required}, found {found}")]
    DependencyVersionMismatch {
        name: String,
        required: String,
        found: String,
    },

    #[error("Memory threshold exceeded: using {used}MB of {threshold}MB limit")]
    MemoryThresholdExceeded { used: u64, threshold: u64 },

    #[error("Video too long: {duration} seconds (maximum: {max} seconds)")]
    VideoTooLong { duration: u64, max: u64 },

    #[error("Video age-restricted: cannot download age-restricted content")]
    VideoAgeRestricted,

    #[error("Video region-locked: not available in your region")]
    VideoRegionLocked,

    #[error("Video deleted: the video has been removed by the uploader")]
    VideoDeleted,

    #[error("Video private: this video is not publicly available")]
    VideoPrivate,

    #[error("Session not found: {0}")]
    SessionNotFound(Id<Session>),

    #[error("Session recovery failed: {0}")]
    SessionRecoveryFailed(String),

    #[error("Corrupt frame at timestamp {timestamp}s - skipping")]
    CorruptFrame { timestamp: f64 },

    #[error("Too many corrupt frames: {count} frames skipped (maximum: {max})")]
    TooManyCorruptFrames { count: u32, max: u32 },

    #[error("OCR confidence too low for slide {slide_id}: {confidence} (threshold: {threshold})")]
    LowOcrConfidence {
        slide_id: Id<Slide>,
        confidence: f64,
        threshold: f64,
    },

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

impl ExtractionError {
    /// Returns the error category for this error.
    ///
    /// This helps classify errors for better user messaging and handling.
    pub fn category(&self) -> ErrorCategory {
        match self {
            ExtractionError::InvalidUrl(_) => ErrorCategory::Validation,
            ExtractionError::VideoUnavailable(_) => ErrorCategory::Network,
            ExtractionError::InvalidConfig(_) => ErrorCategory::Configuration,
            ExtractionError::DownloadFailed(_, _) => ErrorCategory::Network,
            ExtractionError::FrameExtractionFailed(_) => ErrorCategory::Processing,
            ExtractionError::HashComputationFailed(_) => ErrorCategory::Processing,
            ExtractionError::OcrFailed(_, _) => ErrorCategory::Processing,
            ExtractionError::MarkdownGenerationFailed(_) => ErrorCategory::Processing,
            ExtractionError::NoUniqueSlidesFound => ErrorCategory::Processing,
            ExtractionError::OutputDirectoryNotWritable(_) => ErrorCategory::FileSystem,
            ExtractionError::InsufficientMemory(_) => ErrorCategory::Memory,
            ExtractionError::NetworkTimeout(_) => ErrorCategory::Network,
            ExtractionError::ExternalDependencyUnavailable(_) => ErrorCategory::ExternalDependency,
            ExtractionError::OutputDirectoryNotFound(_) => ErrorCategory::FileSystem,
            ExtractionError::ParentDirectoryNotFound(_) => ErrorCategory::FileSystem,
            ExtractionError::InsufficientDiskSpace(_, _) => ErrorCategory::FileSystem,
            ExtractionError::PermissionDenied(_) => ErrorCategory::FileSystem,
            ExtractionError::DependencyCheckFailed { .. } => ErrorCategory::ExternalDependency,
            ExtractionError::DependencyVersionMismatch { .. } => ErrorCategory::ExternalDependency,
            ExtractionError::MemoryThresholdExceeded { .. } => ErrorCategory::Memory,
            ExtractionError::VideoTooLong { .. } => ErrorCategory::Validation,
            ExtractionError::VideoAgeRestricted => ErrorCategory::Validation,
            ExtractionError::VideoRegionLocked => ErrorCategory::Validation,
            ExtractionError::VideoDeleted => ErrorCategory::Validation,
            ExtractionError::VideoPrivate => ErrorCategory::Validation,
            ExtractionError::SessionNotFound(_) => ErrorCategory::Processing,
            ExtractionError::SessionRecoveryFailed(_) => ErrorCategory::Processing,
            ExtractionError::CorruptFrame { .. } => ErrorCategory::Processing,
            ExtractionError::TooManyCorruptFrames { .. } => ErrorCategory::Processing,
            ExtractionError::LowOcrConfidence { .. } => ErrorCategory::Processing,
            ExtractionError::TemplateError(_) => ErrorCategory::Configuration,
            ExtractionError::InternalError(_) => ErrorCategory::Unknown,
        }
    }

    /// Returns a user-friendly error message with actionable guidance.
    ///
    /// This provides more detailed information and suggestions for resolution
    /// as specified in US-ERR-01.
    pub fn user_message(&self) -> String {
        match self {
            ExtractionError::InvalidUrl(url) => {
                format!(
                    "The YouTube URL '{}' is invalid. Please ensure you provide a valid YouTube URL \
                    (e.g., https://www.youtube.com/watch?v=VIDEO_ID or https://youtu.be/VIDEO_ID).",
                    url
                )
            }

            ExtractionError::VideoUnavailable(video_id) => {
                format!(
                    "The video with ID '{}' is unavailable. This could be because the video has \
                    been deleted, is private, or has been made unavailable by the uploader.",
                    video_id
                )
            }

            ExtractionError::InvalidConfig(msg) => {
                format!(
                    "Invalid configuration: {}. Please check your configuration parameters and try again.",
                    msg
                )
            }

            ExtractionError::DownloadFailed(retries, reason) => {
                format!(
                    "Failed to download the video after {} retries. Reason: {}. \
                    Please check your internet connection and try again. If the problem persists, \
                    the video may be unavailable or region-locked.",
                    retries, reason
                )
            }

            ExtractionError::FrameExtractionFailed(reason) => {
                format!(
                    "Failed to extract frames from the video: {}. \
                    This may be due to a corrupted video file or an issue with FFmpeg. \
                    Please ensure FFmpeg is properly installed and try again.",
                    reason
                )
            }

            ExtractionError::HashComputationFailed(frame_id) => {
                format!(
                    "Failed to compute hash for frame {}: {}. \
                    This is an internal error. Please report this issue if it persists.",
                    frame_id, "Image processing error"
                )
            }

            ExtractionError::OcrFailed(slide_id, reason) => {
                format!(
                    "OCR failed for slide {}: {}. \
                    Please ensure Tesseract is properly installed and the required language data \
                    is available. The slide image will still be included in the output.",
                    slide_id, reason
                )
            }

            ExtractionError::MarkdownGenerationFailed(reason) => {
                format!(
                    "Failed to generate Markdown document: {}. \
                    Please check the output directory permissions and try again.",
                    reason
                )
            }

            ExtractionError::NoUniqueSlidesFound => {
                "No unique slides were found in the video. This could happen if: \
                1) The video doesn't contain slides, 2) The similarity threshold is too high, \
                or 3) The frame interval is too large. Try lowering the similarity threshold \
                or reducing the frame extraction interval."
                    .to_string()
            }

            ExtractionError::OutputDirectoryNotWritable(dir) => {
                format!(
                    "Cannot write to output directory '{}'. Please check that you have write \
                    permissions for this directory or choose a different output directory.",
                    dir
                )
            }

            ExtractionError::InsufficientMemory(required_mb) => {
                format!(
                    "Insufficient memory available. The operation requires approximately {}MB of memory. \
                    Suggestions: 1) Process a shorter video, 2) Increase the frame extraction interval, \
                    or 3) Close other applications to free up memory.",
                    required_mb
                )
            }

            ExtractionError::NetworkTimeout(duration) => {
                format!(
                    "Network timeout after {:?}. Please check your internet connection and try again. \
                    If you're on a slow connection, consider increasing the timeout duration.",
                    duration
                )
            }

            ExtractionError::ExternalDependencyUnavailable(dep) => {
                format!(
                    "External dependency '{}' is not available. Please ensure it is installed and \
                    accessible in your system PATH. See the documentation for installation instructions.",
                    dep
                )
            }

            ExtractionError::OutputDirectoryNotFound(dir) => {
                format!(
                    "Output directory '{}' does not exist. Please create this directory or provide \
                    a valid output path.",
                    dir
                )
            }

            ExtractionError::ParentDirectoryNotFound(dir) => {
                format!(
                    "Parent directory '{}' does not exist. Cannot create output directory. \
                    Please ensure the parent directory exists or provide a valid output path.",
                    dir
                )
            }

            ExtractionError::InsufficientDiskSpace(required, available) => {
                format!(
                    "Insufficient disk space. Required: {}MB, Available: {}MB. \
                    Please free up disk space or choose a different output directory.",
                    required, available
                )
            }

            ExtractionError::PermissionDenied(resource) => {
                format!(
                    "Permission denied: {}. Please ensure you have the necessary permissions to \
                    access this resource or try running with elevated privileges.",
                    resource
                )
            }

            ExtractionError::DependencyCheckFailed { name, reason } => {
                format!(
                    "External dependency check failed for '{}': {}. Please ensure the dependency \
                    is properly installed and configured. See the documentation for troubleshooting steps.",
                    name, reason
                )
            }

            ExtractionError::DependencyVersionMismatch {
                name,
                required,
                found,
            } => {
                format!(
                    "External dependency version mismatch for '{}': required {}, found {}. \
                    Please install the required version of the dependency.",
                    name, required, found
                )
            }

            ExtractionError::MemoryThresholdExceeded { used, threshold } => {
                format!(
                    "Memory usage exceeded threshold: using {}MB of {}MB limit. \
                    Suggestions: 1) Process a shorter video, 2) Increase the frame extraction interval, \
                    or 3) Increase the memory threshold in configuration.",
                    used, threshold
                )
            }

            ExtractionError::VideoTooLong { duration, max } => {
                format!(
                    "Video is too long: {} seconds (maximum: {} seconds). \
                    Please process a shorter video or adjust the maximum duration setting.",
                    duration, max
                )
            }

            ExtractionError::VideoAgeRestricted => {
                "The video is age-restricted and cannot be downloaded. Age-restricted content \
                is not supported. Please try a different video."
                    .to_string()
            }

            ExtractionError::VideoRegionLocked => {
                "The video is region-locked and not available in your current region. \
                Please try a different video or use a VPN if applicable."
                    .to_string()
            }

            ExtractionError::VideoDeleted => {
                "The video has been deleted by the uploader and is no longer available. \
                Please try a different video."
                    .to_string()
            }

            ExtractionError::VideoPrivate => {
                "The video is private and not publicly available. Please try a different video \
                or ensure you have access to the private content."
                    .to_string()
            }

            ExtractionError::SessionNotFound(session_id) => {
                format!(
                    "Session '{}' not found. The session may have expired or the ID is incorrect.",
                    session_id
                )
            }

            ExtractionError::SessionRecoveryFailed(reason) => {
                format!(
                    "Failed to recover session: {}. Please start a new extraction session.",
                    reason
                )
            }

            ExtractionError::CorruptFrame { timestamp } => {
                format!(
                    "Corrupt frame detected at {}s. This frame will be skipped. \
                    If many frames are corrupt, the video file may be damaged.",
                    timestamp
                )
            }

            ExtractionError::TooManyCorruptFrames { count, max } => {
                format!(
                    "Too many corrupt frames: {} frames skipped (maximum allowed: {}). \
                    The video file may be damaged. Please try a different video or re-download.",
                    count, max
                )
            }

            ExtractionError::LowOcrConfidence {
                slide_id,
                confidence,
                threshold,
            } => {
                format!(
                    "Low OCR confidence for slide {}: {:.2}% (threshold: {:.2}%). \
                    The extracted text may be inaccurate. This is indicated in the output with a warning.",
                    slide_id, confidence * 100.0, threshold * 100.0
                )
            }

            ExtractionError::TemplateError(reason) => {
                format!(
                    "Template error: {}. Please check your template syntax and ensure it's valid.",
                    reason
                )
            }

            ExtractionError::InternalError(msg) => {
                format!(
                    "An internal error occurred: {}. This is likely a bug. Please report this issue \
                    with details about what you were doing when it happened.",
                    msg
                )
            }
        }
    }

    /// Returns a short, one-line error message suitable for display.
    pub fn short_message(&self) -> String {
        self.to_string()
    }
}

pub type DomainResult<T> = Result<T, ExtractionError>;

#[derive(Debug, Clone)]
pub struct YouTubeVideo;

#[derive(Debug, Clone)]
pub struct VideoFrame;

#[derive(Debug, Clone)]
pub struct Slide;

#[derive(Debug, Clone)]
pub struct Session;

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
