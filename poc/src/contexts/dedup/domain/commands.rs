use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Command to identify unique slides from a set of frames.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyUniqueSlidesCommand {
    /// The ID of the video being processed
    pub video_id: Id<YouTubeVideo>,
    /// Directory where extracted frames are stored
    pub frames_dir: String,
    /// Directory where unique slides should be preserved
    pub slides_dir: String,
    /// Similarity threshold (0.0 to 1.0)
    /// Frames with similarity above this are considered the same slide
    pub similarity_threshold: f64,
    /// Whether to use the first or middle frame of a group as representative
    pub selection_strategy: SelectionStrategy,
}

/// Strategy for selecting a representative frame from a group of similar frames.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SelectionStrategy {
    /// Select the first frame in the sequence
    First,
    /// Select the middle frame in the sequence (often more stable)
    Middle,
    /// Select the last frame in the sequence
    Last,
}

impl Default for IdentifyUniqueSlidesCommand {
    fn default() -> Self {
        Self {
            video_id: Id::new(),
            frames_dir: String::new(),
            slides_dir: String::new(),
            similarity_threshold: 0.95,
            selection_strategy: SelectionStrategy::Middle,
        }
    }
}
