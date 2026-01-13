// Handlers for deduplication
use crate::contexts::dedup::domain::commands::{IdentifyUniqueSlidesCommand, SelectionStrategy};
use crate::contexts::dedup::domain::events::{SlidePreserved, UniqueSlidesIdentified};
use crate::contexts::dedup::infrastructure::comparer::HashComparer;
use crate::shared::domain::{DomainResult, ExtractionError};

/// Handles the identify unique slides command.
pub fn handle_identify_unique_slides(
    command: IdentifyUniqueSlidesCommand,
) -> DomainResult<(UniqueSlidesIdentified, Vec<SlidePreserved>)> {
    if command.frames.is_empty() {
        return Err(ExtractionError::NoUniqueSlidesFound);
    }

    let mut groups: Vec<Vec<_>> = Vec::new();
    let mut current_group = Vec::new();

    for frame in command.frames {
        if current_group.is_empty() {
            current_group.push(frame);
        } else {
            // Compare with the first frame of the current group
            let similarity =
                HashComparer::calculate_similarity(&current_group[0].hash, &frame.hash);
            if similarity >= command.similarity_threshold {
                current_group.push(frame);
            } else {
                groups.push(std::mem::replace(&mut current_group, vec![frame]));
            }
        }
    }
    if !current_group.is_empty() {
        groups.push(current_group);
    }

    let mut slide_preserved_events = Vec::new();
    for (index, group) in groups.into_iter().enumerate() {
        let representative = match command.selection_strategy {
            SelectionStrategy::First => &group[0],
            SelectionStrategy::Middle => &group[group.len() / 2],
            SelectionStrategy::Last => &group[group.len() - 1],
        };

        slide_preserved_events.push(SlidePreserved {
            video_id: command.video_id.clone(),
            frame_id: representative.frame_id.clone(),
            slide_index: (index + 1) as u32,
            slide_path: format!("{}/slide_{:04}.jpg", command.slides_dir, index + 1), // Final path will be handled by infrastructure
        });
    }

    let summary = UniqueSlidesIdentified {
        video_id: command.video_id,
        slide_count: slide_preserved_events.len() as u32,
        slides_dir: command.slides_dir,
    };

    Ok((summary, slide_preserved_events))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::dedup::domain::commands::FrameDedupMetadata;
    use crate::shared::domain::{Id, VideoFrame, YouTubeVideo};

    #[test]
    fn test_handle_identify_unique_slides_success() {
        let video_id = Id::<YouTubeVideo>::new();
        let frames = vec![
            FrameDedupMetadata {
                frame_id: Id::<VideoFrame>::new(),
                frame_number: 1,
                timestamp: 0.0,
                hash: "ffff".to_string(),
                frame_path: "path1".to_string(),
            },
            FrameDedupMetadata {
                frame_id: Id::<VideoFrame>::new(),
                frame_number: 2,
                timestamp: 5.0,
                hash: "ffff".to_string(), // Similar
                frame_path: "path2".to_string(),
            },
            FrameDedupMetadata {
                frame_id: Id::<VideoFrame>::new(),
                frame_number: 3,
                timestamp: 10.0,
                hash: "0000".to_string(), // Different
                frame_path: "path3".to_string(),
            },
        ];

        let command = IdentifyUniqueSlidesCommand {
            video_id: video_id.clone(),
            frames,
            slides_dir: "/tmp/slides".to_string(),
            similarity_threshold: 0.95,
            selection_strategy: SelectionStrategy::First,
        };

        let result = handle_identify_unique_slides(command).unwrap();
        assert_eq!(result.0.slide_count, 2);
        assert_eq!(result.1.len(), 2);
        assert_eq!(result.1[0].slide_index, 1);
        assert_eq!(result.1[1].slide_index, 2);
    }

    #[test]
    fn test_handle_identify_unique_slides_empty() {
        let video_id = Id::<YouTubeVideo>::new();
        let command = IdentifyUniqueSlidesCommand {
            video_id: video_id.clone(),
            frames: vec![],
            slides_dir: "/tmp/slides".to_string(),
            similarity_threshold: 0.95,
            selection_strategy: SelectionStrategy::First,
        };

        let result = handle_identify_unique_slides(command);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_identify_unique_slides_single_frame() {
        let video_id = Id::<YouTubeVideo>::new();
        let frames = vec![FrameDedupMetadata {
            frame_id: Id::<VideoFrame>::new(),
            frame_number: 1,
            timestamp: 0.0,
            hash: "ffff".to_string(),
            frame_path: "path1".to_string(),
        }];

        let command = IdentifyUniqueSlidesCommand {
            video_id: video_id.clone(),
            frames,
            slides_dir: "/tmp/slides".to_string(),
            similarity_threshold: 0.95,
            selection_strategy: SelectionStrategy::First,
        };

        let result = handle_identify_unique_slides(command).unwrap();
        assert_eq!(result.0.slide_count, 1);
        assert_eq!(result.1.len(), 1);
    }
}
