use crate::contexts::video::domain::state::VideoUrlValidated;
use crate::contexts::video::domain::VideoDownloaded;
use crate::shared::domain::Id;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum VideoEvent {
    UrlValidated(VideoUrlValidated),
    Downloaded(VideoDownloaded),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_video_event_url_validated() {
        let uuid = Uuid::new_v4();
        let event = VideoEvent::UrlValidated(VideoUrlValidated {
            url: "https://www.youtube.com/watch?v=abc".to_string(),
            video_id: Id::from_uuid(uuid),
        });
        assert!(matches!(event, VideoEvent::UrlValidated(_)));
    }
}
