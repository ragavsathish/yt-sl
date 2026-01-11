use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadVideoCommand {
    pub video_id: Id<YouTubeVideo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUrlCommand {
    pub url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_download_video_command() {
        let uuid = Uuid::new_v4();
        let cmd = DownloadVideoCommand {
            video_id: Id::<YouTubeVideo>::from_uuid(uuid),
        };
        assert_eq!(cmd.video_id.as_uuid(), uuid);
    }

    #[test]
    fn test_validate_url_command() {
        let cmd = ValidateUrlCommand {
            url: "https://www.youtube.com/watch?v=dQw4w9WgXcQ".to_string(),
        };
        assert_eq!(cmd.url, "https://www.youtube.com/watch?v=dQw4w9WgXcQ");
    }
}
