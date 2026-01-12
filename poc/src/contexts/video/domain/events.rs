use crate::contexts::video::domain::state::VideoUrlValidated;
use crate::shared::domain::{DomainResult, ExtractionError, Id, YouTubeVideo};
use regex::Regex;
use std::sync::OnceLock;

/// Validates a YouTube URL and extracts the video ID.
///
/// This function provides URL validation functionality as specified in US-VIDEO-01:
/// Validate YouTube URL.
///
/// # Arguments
///
/// * `url` - The URL string to validate
///
/// # Returns
///
/// A VideoUrlValidated event containing the URL and video ID
///
/// # Errors
///
/// Returns an error if the URL is invalid or not a YouTube URL
pub fn validate_video_url(url: &str) -> DomainResult<VideoUrlValidated> {
    if url.is_empty() {
        return Err(ExtractionError::InvalidUrl("URL is empty".to_string()));
    }

    if !is_valid_youtube_url(url) {
        return Err(ExtractionError::InvalidUrl(format!(
            "Not a valid YouTube URL: {}",
            url
        )));
    }

    let video_id_str = extract_video_id(url);

    if video_id_str.is_empty() {
        return Err(ExtractionError::InvalidUrl(
            "Could not extract video ID from URL".to_string(),
        ));
    }

    // Validate video ID format
    validate_video_id_format(&video_id_str)?;

    let video_id: Id<YouTubeVideo> = video_id_str.parse().map_err(|_| {
        ExtractionError::InvalidUrl(format!("Invalid video ID format: {}", video_id_str))
    })?;

    Ok(VideoUrlValidated {
        url: url.to_string(),
        video_id,
    })
}

/// Extracts the video ID from a YouTube URL.
///
/// # Arguments
///
/// * `url` - The YouTube URL
///
/// # Returns
///
/// The extracted video ID string
pub fn extract_video_id(url: &str) -> String {
    if url.contains("v=") {
        url.split("v=")
            .nth(1)
            .unwrap_or("")
            .split('&')
            .next()
            .unwrap_or("")
            .to_string()
    } else if url.contains("youtu.be/") {
        url.split("youtu.be/")
            .nth(1)
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_string()
    } else if url.contains("/embed/") {
        url.split("/embed/")
            .nth(1)
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_string()
    } else if url.contains("/shorts/") {
        url.split("/shorts/")
            .nth(1)
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_string()
    } else if url.contains("/v/") {
        url.split("/v/")
            .nth(1)
            .unwrap_or("")
            .split('?')
            .next()
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    }
}

/// Checks if a URL is a valid YouTube URL.
///
/// # Arguments
///
/// * `url` - The URL string to check
///
/// # Returns
///
/// true if the URL is a valid YouTube URL, false otherwise
pub fn is_valid_youtube_url(url: &str) -> bool {
    url.starts_with("https://www.youtube.com/")
        || url.starts_with("https://youtube.com/")
        || url.starts_with("https://m.youtube.com/")
        || url.starts_with("https://music.youtube.com/")
        || url.starts_with("https://youtu.be/")
}

/// Validates the format of a video ID.
///
/// # Arguments
///
/// * `video_id` - The video ID string to validate
///
/// # Returns
///
/// Ok(()) if valid, error otherwise
fn validate_video_id_format(video_id: &str) -> DomainResult<()> {
    if video_id.is_empty() {
        return Err(ExtractionError::InvalidUrl("Video ID is empty".to_string()));
    }

    if video_id.len() < 10 || video_id.len() > 12 {
        return Err(ExtractionError::InvalidUrl(format!(
            "Video ID has invalid length: {} (expected 10-12 characters)",
            video_id.len()
        )));
    }

    // YouTube video IDs are typically alphanumeric with some special characters
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

/// Lazy-initialized regex for video ID validation.
static VIDEO_ID_REGEX: OnceLock<Regex> = OnceLock::new();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_youtube_url() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = validate_video_url(url);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.url, url);
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(event.video_id, expected_id);
    }

    #[test]
    fn test_validate_youtu_be_short_url() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        let result = validate_video_url(url);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.url, url);
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(event.video_id, expected_id);
    }

    #[test]
    fn test_validate_embed_url() {
        let url = "https://www.youtube.com/embed/dQw4w9WgXcQ";
        let result = validate_video_url(url);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.url, url);
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(event.video_id, expected_id);
    }

    #[test]
    fn test_validate_shorts_url() {
        let url = "https://www.youtube.com/shorts/dQw4w9WgXcQ";
        let result = validate_video_url(url);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.url, url);
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(event.video_id, expected_id);
    }

    #[test]
    fn test_validate_v_url() {
        let url = "https://www.youtube.com/v/dQw4w9WgXcQ";
        let result = validate_video_url(url);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.url, url);
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(event.video_id, expected_id);
    }

    #[test]
    fn test_validate_with_additional_params() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s";
        let result = validate_video_url(url);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.url, url);
        // Verify that the video ID can be parsed from the same string
        let expected_id: Id<YouTubeVideo> = "dQw4w9WgXcQ".parse().unwrap();
        assert_eq!(event.video_id, expected_id);
    }

    #[test]
    fn test_validate_invalid_url() {
        let url = "https://example.com/video";
        let result = validate_video_url(url);
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_empty_url() {
        let url = "";
        let result = validate_video_url(url);
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_validate_invalid_video_id_format() {
        let url = "https://www.youtube.com/watch?v=invalid@id";
        let result = validate_video_url(url);
        assert!(matches!(result, Err(ExtractionError::InvalidUrl(_))));
    }

    #[test]
    fn test_extract_video_id_from_watch_url() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert_eq!(result, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_from_short_url() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert_eq!(result, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_from_embed_url() {
        let url = "https://www.youtube.com/embed/dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert_eq!(result, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_from_shorts_url() {
        let url = "https://www.youtube.com/shorts/dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert_eq!(result, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_from_v_url() {
        let url = "https://www.youtube.com/v/dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert_eq!(result, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_extract_video_id_with_params() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ&t=10s";
        let result = extract_video_id(url);
        assert_eq!(result, "dQw4w9WgXcQ");
    }

    #[test]
    fn test_is_valid_youtube_url() {
        assert!(is_valid_youtube_url("https://www.youtube.com/watch?v=test"));
        assert!(is_valid_youtube_url("https://youtube.com/watch?v=test"));
        assert!(is_valid_youtube_url("https://m.youtube.com/watch?v=test"));
        assert!(is_valid_youtube_url(
            "https://music.youtube.com/watch?v=test"
        ));
        assert!(is_valid_youtube_url("https://youtu.be/test"));
        assert!(!is_valid_youtube_url("https://example.com/video"));
        assert!(!is_valid_youtube_url("http://www.youtube.com/watch?v=test")); // http not https
    }

    #[test]
    fn test_validate_video_id_format_valid() {
        assert!(validate_video_id_format("dQw4w9WgXcQ").is_ok());
        assert!(validate_video_id_format("abcdefghijk").is_ok());
        assert!(validate_video_id_format("ABC123-xyz_1").is_ok());
    }

    #[test]
    fn test_validate_video_id_format_invalid() {
        assert!(validate_video_id_format("").is_err());
        assert!(validate_video_id_format("short").is_err());
        assert!(validate_video_id_format("this_is_way_too_long_id").is_err());
        assert!(validate_video_id_format("invalid@chars!").is_err());
    }
}
