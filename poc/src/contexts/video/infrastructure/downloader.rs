use crate::shared::domain::{Id, YouTubeVideo};
use async_trait::async_trait;

#[async_trait]
pub trait VideoDownloader: Send + Sync {
    async fn download(
        &self,
        video_id: &Id<YouTubeVideo>,
    ) -> Result<(String, u64), Box<dyn std::error::Error>>;
}

pub struct MockVideoDownloader;

#[async_trait]
impl VideoDownloader for MockVideoDownloader {
    async fn download(
        &self,
        _video_id: &Id<YouTubeVideo>,
    ) -> Result<(String, u64), Box<dyn std::error::Error>> {
        Ok(("/tmp/mock_video.mp4".to_string(), 180))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_downloader() {
        let downloader = MockVideoDownloader;
        let video_id = Id::new();
        let result = downloader.download(&video_id).await;
        assert!(result.is_ok());
        let (path, duration) = result.unwrap();
        assert_eq!(path, "/tmp/mock_video.mp4");
        assert_eq!(duration, 180);
    }
}
