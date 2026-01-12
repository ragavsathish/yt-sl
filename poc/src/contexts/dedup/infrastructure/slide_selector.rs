use crate::contexts::dedup::domain::events::SlidePreserved;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::fs;
use std::path::Path;

/// Infrastructure component for preserving unique slide images.
pub struct SlideSelector;

impl SlideSelector {
    /// Preserves unique slide images by copying/moving them to the slides directory.
    pub fn preserve_slides(
        events: &[SlidePreserved],
        frames: &[crate::contexts::dedup::domain::commands::FrameDedupMetadata],
    ) -> DomainResult<()> {
        for event in events {
            // Find the frame metadata for this event
            let frame_metadata = frames
                .iter()
                .find(|f| f.frame_id == event.frame_id)
                .ok_or_else(|| {
                    ExtractionError::InternalError(format!(
                        "Frame ID {} not found in metadata",
                        event.frame_id
                    ))
                })?;

            let source_path = Path::new(&frame_metadata.frame_path);
            let target_path = Path::new(&event.slide_path);

            // Ensure parent directory exists
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    ExtractionError::FileSystemError(format!(
                        "Failed to create slides directory: {}",
                        e
                    ))
                })?;
            }

            // Copy frame to slide path
            fs::copy(source_path, target_path).map_err(|e| {
                ExtractionError::FileSystemError(format!(
                    "Failed to copy frame {} to slide {}: {}",
                    source_path.display(),
                    target_path.display(),
                    e
                ))
            })?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::dedup::domain::commands::FrameDedupMetadata;
    use crate::shared::domain::{Id, VideoFrame, YouTubeVideo};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_preserve_slides_success() {
        let temp_dir = TempDir::new().unwrap();
        let frames_dir = temp_dir.path().join("frames");
        let slides_dir = temp_dir.path().join("slides");
        fs::create_dir(&frames_dir).unwrap();

        let frame_id = Id::<VideoFrame>::new();
        let frame_path = frames_dir.join("frame_0001.png");
        let mut file = fs::File::create(&frame_path).unwrap();
        file.write_all(b"fake image data").unwrap();

        let frames = vec![FrameDedupMetadata {
            frame_id: frame_id.clone(),
            frame_number: 1,
            timestamp: 0.0,
            hash: "ffff".to_string(),
            frame_path: frame_path.to_str().unwrap().to_string(),
        }];

        let events = vec![SlidePreserved {
            video_id: Id::<YouTubeVideo>::new(),
            frame_id,
            slide_index: 1,
            slide_path: slides_dir
                .join("slide_0001.jpg")
                .to_str()
                .unwrap()
                .to_string(),
        }];

        let result = SlideSelector::preserve_slides(&events, &frames);
        assert!(result.is_ok());
        assert!(slides_dir.join("slide_0001.jpg").exists());
    }
}
