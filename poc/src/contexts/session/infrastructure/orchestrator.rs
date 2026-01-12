use crate::contexts::dedup::domain::commands::{
    FrameDedupMetadata, IdentifyUniqueSlidesCommand, SelectionStrategy,
};
use crate::contexts::dedup::domain::handlers::handle_identify_unique_slides;
use crate::contexts::dedup::infrastructure::SlideSelector;
use crate::contexts::document::domain::commands::{GenerateDocumentCommand, SlideData};
use crate::contexts::document::domain::handlers::handle_generate_document;
use crate::contexts::frame::domain::commands::{
    ComputeHashCommand, ExtractFramesCommand, FrameFormat, HashAlgorithm,
};
use crate::contexts::frame::infrastructure::{FrameExtractor, PerceptualHasher};
use crate::contexts::ocr::domain::commands::ExtractTextCommand;
use crate::contexts::ocr::domain::handlers::handle_extract_text;
use crate::contexts::session::domain::commands::StartExtractionSessionCommand;
use crate::contexts::session::domain::events::DocumentGenerated;
use crate::contexts::video::domain::commands::DownloadVideoCommand;
use crate::contexts::video::infrastructure::{AvailabilityChecker, UrlValidator, VideoDownloader};
use crate::shared::domain::{DomainResult, Id};

/// Central orchestrator for the video slide extraction pipeline.
pub struct SessionOrchestrator;

impl SessionOrchestrator {
    /// Executes the full extraction pipeline from URL to Markdown document.
    pub async fn run_session(
        command: StartExtractionSessionCommand,
    ) -> DomainResult<DocumentGenerated> {
        let session_id = command.session_id;
        let session_dir = format!("{}/{}", command.output_dir, session_id);
        let frames_dir = format!("{}/frames", session_dir);
        let slides_dir = format!("{}/slides", session_dir);
        let doc_path = format!("{}/report.md", session_dir);

        // 1. Validate URL
        let validator = UrlValidator::new();
        let (url, video_id) = validator.validate_and_extract(&command.youtube_url)?;

        // 2. Check Availability and Get Metadata
        let checker = AvailabilityChecker::new();
        let metadata = checker.check_availability(&video_id, &url).await?;

        // 3. Download Video
        let downloader = VideoDownloader::new();
        let download_cmd = DownloadVideoCommand {
            video_id: video_id.clone(),
        };
        let download_event = downloader
            .download_video(download_cmd, &url, &session_dir)
            .await?;

        // 4. Extract Frames
        let mut extractor = FrameExtractor::new();
        let extract_cmd = ExtractFramesCommand {
            video_id: video_id.clone(),
            video_path: download_event.path.clone(),
            output_dir: frames_dir.clone(),
            interval_secs: command.frame_interval_secs,
            output_format: FrameFormat::Jpeg,
            jpeg_quality: Some(85),
        };
        let (_frames_extracted, frames) = extractor
            .extract_frames(extract_cmd, metadata.duration)
            .await?;

        // 5. Compute Hashes
        let hasher = PerceptualHasher::new();
        let mut frames_with_hashes = Vec::new();
        for frame in frames {
            let hash_cmd = ComputeHashCommand {
                frame_id: frame.frame_id.clone(),
                frame_path: frame.frame_path.clone(),
                algorithm: HashAlgorithm::Average,
            };
            let hash_event = hasher.compute_hash(hash_cmd)?;

            frames_with_hashes.push(FrameDedupMetadata {
                frame_id: frame.frame_id,
                frame_number: frame.frame_number,
                timestamp: frame.timestamp,
                hash: hash_event.hash,
                frame_path: frame.frame_path,
            });
        }

        // 6. Identify Unique Slides
        let dedup_cmd = IdentifyUniqueSlidesCommand {
            video_id: video_id.clone(),
            frames: frames_with_hashes.clone(),
            slides_dir: slides_dir.clone(),
            similarity_threshold: command.similarity_threshold,
            selection_strategy: SelectionStrategy::Middle,
        };
        let (_dedup_summary, slide_preserved_events) = handle_identify_unique_slides(dedup_cmd)?;

        // 7. Preserve Slide Images
        SlideSelector::preserve_slides(&slide_preserved_events, &frames_with_hashes)?;

        // 8. OCR each slide
        let mut slides_data = Vec::new();
        for event in slide_preserved_events {
            // Find timestamp from original frames
            let timestamp = frames_with_hashes
                .iter()
                .find(|f| f.frame_id == event.frame_id)
                .map(|f| f.timestamp)
                .unwrap_or(0.0);

            let ocr_cmd = ExtractTextCommand {
                slide_id: Id::new(),
                image_path: event.slide_path.clone(),
                languages: command.languages.clone(),
                confidence_threshold: command.confidence_threshold,
            };

            let (ocr_result, _) = handle_extract_text(ocr_cmd)?;

            slides_data.push(SlideData {
                slide_index: event.slide_index,
                timestamp,
                image_path: event.slide_path,
                text: ocr_result.text,
            });
        }

        // 9. Generate Document
        let doc_cmd = GenerateDocumentCommand {
            video_id: video_id.clone(),
            title: metadata.title,
            url: command.youtube_url,
            duration: metadata.duration,
            slides: slides_data,
            output_path: doc_path,
            include_timeline_diagram: true,
        };

        let doc_event = handle_generate_document(doc_cmd)?;

        Ok(DocumentGenerated {
            video_id,
            file_path: doc_event.file_path,
            slide_count: doc_event.slide_count,
        })
    }
}
