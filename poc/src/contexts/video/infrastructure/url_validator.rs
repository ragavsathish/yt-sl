//! URL validation infrastructure for YouTube URLs.
//!
//! This module provides URL validation functionality as specified in US-VIDEO-01:
//! Validate YouTube URL.
//!
//! Features:
//! - Validates YouTube URL format (youtube.com, youtu.be)
//! - Extracts video ID from URL
//! - Checks for valid video ID format
//! - Handles various YouTube URL formats (watch, embed, short)
//! - Provides clear error messages for invalid URLs

use crate::shared::domain::{DomainResult, ExtractionError, Id, YouTubeVideo};
use regex::Regex;
use std::sync::OnceLock;
use url::Url;

/// YouTube URL validator.
///
/// Validates YouTube URLs and extracts video IDs.
#[derive(Debug, Clone)]
pub struct UrlValidator;

impl UrlValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validates a YouTube URL and extracts the video ID.
    pub fn validate_and_extract(&self, url_str: &str) -> DomainResult<(String, Id<YouTubeVideo>)> {
        if url_str.is_empty() {
            return Err(ExtractionError::InvalidUrl("URL is empty".to_string()));
        }

        let parsed_url = Url::parse(url_str)
            .map_err(|e| ExtractionError::InvalidUrl(format!("Invalid URL format: {}", e)))?;

        if !self.is_youtube_url(&parsed_url) {
            return Err(ExtractionError::InvalidUrl(format!(
                "Not a valid YouTube URL: {}",
                url_str
            )));
        }

        let video_id_str = self.extract_video_id(&parsed_url)?;

        self.validate_video_id_format(&video_id_str)?;

        let video_id: Id<YouTubeVideo> = video_id_str.parse().map_err(|_| {
            ExtractionError::InvalidUrl(format!("Invalid video ID format: {}", video_id_str))
        })?;

        Ok((url_str.to_string(), video_id))
    }

    fn is_youtube_url(&self, url: &Url) -> bool {
        let host = url.host_str().unwrap_or("");
        host == "www.youtube.com"
            || host == "youtube.com"
            || host == "youtu.be"
            || host == "m.youtube.com"
            || host == "music.youtube.com"
    }

    fn extract_video_id(&self, url: &Url) -> DomainResult<String> {
        let host = url.host_str().unwrap_or("");

        if host == "youtu.be" {
            // Short URL format: https://youtu.be/VIDEO_ID
            let path = url.path();
            let video_id = path.trim_start_matches('/');
            if video_id.is_empty() {
                return Err(ExtractionError::InvalidUrl(
                    "No video ID found in URL".to_string(),
                ));
            }
            Ok(video_id.split('?').next().unwrap_or(video_id).to_string())
        } else {
            // Standard YouTube URL formats
            // Watch URL: https://www.youtube.com/watch?v=VIDEO_ID
            // Embed URL: https://www.youtube.com/embed/VIDEO_ID
            // Short URL: https://www.youtube.com/shorts/VIDEO_ID
            // V URL: https://www.youtube.com/v/VIDEO_ID

            if let Some(v) = url.query_pairs().find(|(k, _)| k == "v") {
                return Ok(v.1.to_string());
            }

            let path = url.path();
            let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

            if segments.len() >= 2
                && (segments[0] == "embed" || segments[0] == "shorts" || segments[0] == "v")
            {
                return Ok(segments[1].to_string());
            }

            Err(ExtractionError::InvalidUrl(
                "Could not extract video ID from URL".to_string(),
            ))
        }
    }

    fn validate_video_id_format(&self, video_id: &str) -> DomainResult<()> {
        if video_id.is_empty() {
            return Err(ExtractionError::InvalidUrl("Video ID is empty".to_string()));
        }

        if video_id.len() < 10 || video_id.len() > 12 {
            return Err(ExtractionError::InvalidUrl(format!(
                "Video ID has invalid length: {} (expected 10-12 characters)",
                video_id.len()
            )));
        }

        let video_id_regex =
            VIDEO_ID_REGEX.get_or_init(|| Regex::new(r"^[a-zA-Z0-9_-]{10,12}$").unwrap());

        if !video_id_regex.is_match(video_id) {
            return Err(ExtractionError::InvalidUrl(format!(
                "Video ID has invalid format: {}",
                video_id
            )));
        }

        Ok(())
    }
}

impl Default for UrlValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Lazy-initialized regex for video ID validation.
static VIDEO_ID_REGEX: OnceLock<Regex> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_standard_watch_url() {
        let validator = UrlValidator::new();
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = validator.validate_and_extract(url);
        assert!(result.is_ok());
        let (validated_url, video_id) = result.unwrap();
        assert_eq!(validated_url, url);
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(video_id, expected_id);
    }

    #[test]
    fn test_validate_short_url() {
        let validator = UrlValidator::new();
        let url = "https://youtu.be/dQw4w9WgXcQ";
        let result = validator.validate_and_extract(url);
        assert!(result.is_ok());
        let (_, video_id) = result.unwrap();
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(video_id, expected_id);
    }

    #[test]
    fn test_validate_embed_url() {
        let validator = UrlValidator::new();
        let url = "https://www.youtube.com/embed/dQw4w9WgXcQ";
        let result = validator.validate_and_extract(url);
        assert!(result.is_ok());
        let (_, video_id) = result.unwrap();
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(video_id, expected_id);
    }

    #[test]
    fn test_validate_shorts_url() {
        let validator = UrlValidator::new();
        let url = "https://www.youtube.com/shorts/dQw4w9WgXcQ";
        let result = validator.validate_and_extract(url);
        assert!(result.is_ok());
        let (_, video_id) = result.unwrap();
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(video_id, expected_id);
    }

    #[test]
    fn test_validate_v_url() {
        let validator = UrlValidator::new();
        let url = "https://www.youtube.com/v/dQw4w9WgXcQ";
        let result = validator.validate_and_extract(url);
        assert!(result.is_ok());
        let (_, video_id) = result.unwrap();
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(video_id, expected_id);
    }

    #[test]
    fn test_validate_with_additional_params() {
        let validator = UrlValidator::new();
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s";
        let result = validator.validate_and_extract(url);
        assert!(result.is_ok());
        let (_, video_id) = result.unwrap();
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(video_id, expected_id);
    }

    #[test]
    fn test_validate_mobile_url() {
        let validator = UrlValidator::new();
        let url = "https://m.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = validator.validate_and_extract(url);
        assert!(result.is_ok());
        let (_, video_id) = result.unwrap();
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(video_id, expected_id);
    }

    #[test]
    fn test_validate_empty_url() {
        let validator = UrlValidator::new();
        let result = validator.validate_and_extract("");
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_non_youtube_url() {
        let validator = UrlValidator::new();
        let result = validator.validate_and_extract("https://example.com/video");
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_url_without_video_id() {
        let validator = UrlValidator::new();
        let result = validator.validate_and_extract("https://www.youtube.com/watch");
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_short_url_without_video_id() {
        let validator = UrlValidator::new();
        let result = validator.validate_and_extract("https://youtu.be/");
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_invalid_video_id_format() {
        let validator = UrlValidator::new();
        let result = validator.validate_and_extract("https://www.youtube.com/watch?v=invalid@id");
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_is_youtube_url() {
        let validator = UrlValidator::new();

        let url = Url::parse("https://www.youtube.com/watch?v=test").unwrap();
        assert!(validator.is_youtube_url(&url));

        let url = Url::parse("https://youtu.be/test").unwrap();
        assert!(validator.is_youtube_url(&url));

        let url = Url::parse("https://m.youtube.com/watch?v=test").unwrap();
        assert!(validator.is_youtube_url(&url));

        let url = Url::parse("https://example.com/video").unwrap();
        assert!(!validator.is_youtube_url(&url));
    }

    #[test]
    fn test_extract_video_id_from_watch_url() {
        let validator = UrlValidator::new();
        let url = Url::parse("https://www.youtube.com/watch?v=dQw4w9WgXcQ").unwrap();
        let result = validator.extract_video_id(&url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_from_short_url() {
        let validator = UrlValidator::new();
        let url = Url::parse("https://youtu.be/dQw4w9WgXcQ").unwrap();
        let result = validator.extract_video_id(&url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_from_embed_url() {
        let validator = UrlValidator::new();
        let url = Url::parse("https://www.youtube.com/embed/dQw4w9WgXcQ").unwrap();
        let result = validator.extract_video_id(&url);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "dQw4w9WgXcQ");
    }

    #[test]
    fn test_validate_video_id_format_valid() {
        let validator = UrlValidator::new();
        assert!(validator.validate_video_id_format("dQw4w9WgXcQ").is_ok());
        assert!(validator.validate_video_id_format("abcdefghijk").is_ok());
        assert!(validator.validate_video_id_format("ABC123-xyz_1").is_ok());
    }

    #[test]
    fn test_validate_video_id_format_invalid() {
        let validator = UrlValidator::new();
        assert!(validator.validate_video_id_format("").is_err());
        assert!(validator.validate_video_id_format("short").is_err());
        assert!(validator
            .validate_video_id_format("this_is_way_too_long_id")
            .is_err());
        assert!(validator
            .validate_video_id_format("invalid@chars!")
            .is_err());
    }
}
