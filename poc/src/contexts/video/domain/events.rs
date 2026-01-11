use crate::contexts::video::domain::state::VideoUrlValidated;
use crate::shared::domain::{DomainResult, ExtractionError, Id, YouTubeVideo};

pub fn validate_video_url(url: &str) -> DomainResult<VideoUrlValidated> {
    if url.is_empty() {
        return Err(ExtractionError::InvalidUrl("URL is empty".to_string()));
    }

    if !is_valid_youtube_url(url) {
        return Err(ExtractionError::InvalidUrl(url.to_string()));
    }

    let video_id = Id::<YouTubeVideo>::new();
    Ok(VideoUrlValidated {
        url: url.to_string(),
        video_id,
    })
}

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
    } else {
        String::new()
    }
}

pub fn is_valid_youtube_url(url: &str) -> bool {
    url.starts_with("https://www.youtube.com/") || url.starts_with("https://youtu.be/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_youtube_url() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = validate_video_url(url);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_youtu_be_short_url() {
        let url = "https://youtu.be/dQw4w9WgXcQ";
        let result = validate_video_url(url);
        assert!(result.is_ok());
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
    fn test_extract_video_id_from_url() {
        let url = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
        let result = extract_video_id(url);
        assert_eq!(result, "dQw4w9WgXcQ");
    }
}
