use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoUrlValidated {
    pub url: String,
    pub video_id: Id<YouTubeVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoDownloaded {
    pub video_id: Id<YouTubeVideo>,
    pub path: String,
    pub duration_sec: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_video_url_validated() {
        let uuid = Uuid::new_v4();
        let event = VideoUrlValidated {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
            video_id: Id::from_uuid(uuid),
        };
        assert_eq!(event.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }

    #[test]
    fn test_video_downloaded() {
        let uuid = Uuid::new_v4();
        let event = VideoDownloaded {
            video_id: Id::from_uuid(uuid),
            path: "/tmp/video.mp4".to_string(),
            duration_sec: 180,
        };
        assert_eq!(event.path, "/tmp/video.mp4");
        assert_eq!(event.duration_sec, 180);
    }
}
