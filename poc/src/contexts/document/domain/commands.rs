use crate::shared::domain::{Id, YouTubeVideo};
use serde::{Deserialize, Serialize};

/// Data for a single slide in the document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideData {
    pub slide_index: u32,
    pub timestamp: f64,
    pub image_path: String,
    pub text: String,
}

/// Command to generate the final Markdown document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateDocumentCommand {
    pub video_id: Id<YouTubeVideo>,
    pub title: String,
    pub url: String,
    pub duration: u64,
    pub slides: Vec<SlideData>,
    pub output_path: String,
    pub include_timeline_diagram: bool,
}

impl Default for GenerateDocumentCommand {
    fn default() -> Self {
        Self {
            video_id: Id::new(),
            title: "Untitled Video".to_string(),
            url: String::new(),
            duration: 0,
            slides: Vec::new(),
            output_path: "output.md".to_string(),
            include_timeline_diagram: true,
        }
    }
}
