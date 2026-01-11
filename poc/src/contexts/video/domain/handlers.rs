use crate::contexts::video::domain::commands::DownloadVideoCommand;
use crate::contexts::video::domain::state::VideoDownloaded;
use crate::shared::domain::{DomainResult, ExtractionError};

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

    let path = format!("{}/video.mp4", output_path);
    Ok(VideoDownloaded {
        video_id: command.video_id,
        path,
        duration_sec,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain::Id;

    #[test]
    fn test_handle_download_video_success() {
        let command = DownloadVideoCommand {
            video_id: Id::new(),
        };
        let result = handle_download_video(command, "/tmp", 180);
        assert!(result.is_ok());
        let event = result.unwrap();
        assert_eq!(event.path, "/tmp/video.mp4");
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
}
