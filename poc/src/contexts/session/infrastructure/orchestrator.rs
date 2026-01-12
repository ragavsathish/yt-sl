use crate::cli::CliProgressReporter;
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
use crate::contexts::session::domain::state::{SessionState, SessionStatus};
use crate::contexts::video::domain::commands::DownloadVideoCommand;
use crate::contexts::video::infrastructure::{AvailabilityChecker, UrlValidator, VideoDownloader};
use crate::shared::domain::{DomainResult, Id};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Central orchestrator for the video slide extraction pipeline.
pub struct SessionOrchestrator;

impl SessionOrchestrator {
    /// Executes the full extraction pipeline from URL to Markdown document.
    /// Supports session recovery if a session state file exists.
    pub async fn run_session(
        command: StartExtractionSessionCommand,
        progress: Option<Arc<Mutex<CliProgressReporter>>>,
    ) -> DomainResult<DocumentGenerated> {
        let session_id = command.session_id;
        let session_dir = format!("{}/{}", command.output_dir, session_id);
        let frames_dir = format!("{}/frames", session_dir);
        let slides_dir = format!("{}/slides", session_dir);
        let doc_path = format!("{}/report.md", session_dir);
        let state_path = format!("{}/session.json", session_dir);

        // Ensure session directory exists
        fs::create_dir_all(&session_dir).map_err(|e| {
            crate::shared::domain::ExtractionError::FileSystemError(format!(
                "Failed to create session directory: {}",
                e
            ))
        })?;

        // Try to load existing state
        let mut state = if Path::new(&state_path).exists() {
            info!("Existing session found. Attempting recovery...");
            let content = fs::read_to_string(&state_path).map_err(|e| {
                crate::shared::domain::ExtractionError::FileSystemError(format!(
                    "Failed to read session state: {}",
                    e
                ))
            })?;
            serde_json::from_str::<SessionState>(&content).unwrap_or_else(|e| {
                warn!("Failed to parse session state: {}. Starting fresh.", e);
                SessionState::new(session_id.clone())
            })
        } else {
            SessionState::new(session_id.clone())
        };

        // 1. Validate URL & Fetch Metadata
        if state.status == SessionStatus::Starting {
            if let Some(p) = &progress {
                p.lock().await.set_stage("Validating URL...");
            }
            info!("Step 1: Validating URL and fetching metadata...");
            let validator = UrlValidator::new();
            let (url, video_id) = validator.validate_and_extract(&command.youtube_url)?;

            let checker = AvailabilityChecker::new();
            let metadata = checker.check_availability(&video_id, &url).await?;
            info!("Video found: '{}' ({}s)", metadata.title, metadata.duration);

            state.video_metadata = Some(metadata);
            state.status = SessionStatus::MetadataFetched;
            Self::save_state(&state_path, &state)?;
        }

        let metadata = state.video_metadata.as_ref().unwrap().clone();

        // 2. Download Video
        if state.status == SessionStatus::MetadataFetched {
            if let Some(p) = &progress {
                p.lock().await.set_stage("Downloading video...");
            }
            info!("Step 2: Downloading video...");
            let url = command.youtube_url.clone();
            let downloader = VideoDownloader::new();
            let download_cmd = DownloadVideoCommand {
                video_id: Id::new(),
            };
            let download_event = downloader
                .download_video(download_cmd, &url, &session_dir)
                .await?;

            state.video_path = Some(download_event.path);
            state.status = SessionStatus::VideoDownloaded;
            Self::save_state(&state_path, &state)?;
        }
        info!("Video available at: {}", state.video_path.as_ref().unwrap());

        // 3. Extract Frames
        if state.status == SessionStatus::VideoDownloaded {
            if let Some(p) = &progress {
                p.lock().await.set_stage("Extracting frames...");
            }
            info!(
                "Step 3: Extracting frames at {}s intervals...",
                command.frame_interval_secs
            );
            let mut extractor = FrameExtractor::new();
            let extract_cmd = ExtractFramesCommand {
                video_id: Id::new(),
                video_path: state.video_path.as_ref().unwrap().clone(),
                output_dir: frames_dir.clone(),
                interval_secs: command.frame_interval_secs,
                output_format: FrameFormat::Jpeg,
                jpeg_quality: Some(85),
            };
            let (_frames_extracted, _frames) = extractor
                .extract_frames(extract_cmd, metadata.duration)
                .await?;

            state.frames_dir = Some(frames_dir.clone());
            state.status = SessionStatus::FramesExtracted;
            Self::save_state(&state_path, &state)?;
        }
        info!(
            "Frames extracted to: {}",
            state.frames_dir.as_ref().unwrap()
        );

        // 4. Compute Hashes & Identify Unique Slides
        if state.status == SessionStatus::FramesExtracted {
            if let Some(p) = &progress {
                p.lock().await.set_stage("Deduplicating slides...");
            }
            info!("Step 4: Computing hashes and identifying unique slides...");

            let mut frames_with_hashes = Vec::new();

            let entries = fs::read_dir(state.frames_dir.as_ref().unwrap()).map_err(|e| {
                crate::shared::domain::ExtractionError::FileSystemError(format!(
                    "Failed to read frames directory: {}",
                    e
                ))
            })?;

            let hasher = PerceptualHasher::new();
            let mut entries_vec: Vec<_> = entries.filter_map(Result::ok).collect();
            entries_vec.sort_by_key(|e| e.path());

            if let Some(p) = &progress {
                p.lock().await.start_progress(entries_vec.len() as u64);
            }

            for (i, entry) in entries_vec.into_iter().enumerate() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("jpg") {
                    let frame_id = Id::new();
                    let hash_cmd = ComputeHashCommand {
                        frame_id: frame_id.clone(),
                        frame_path: path.to_str().unwrap().to_string(),
                        algorithm: HashAlgorithm::Average,
                    };
                    let hash_event = hasher.compute_hash(hash_cmd)?;

                    frames_with_hashes.push(FrameDedupMetadata {
                        frame_id,
                        frame_number: (i + 1) as u32,
                        timestamp: (i as u64 * command.frame_interval_secs) as f64,
                        hash: hash_event.hash,
                        frame_path: path.to_str().unwrap().to_string(),
                    });
                }
                if let Some(p) = &progress {
                    p.lock().await.update_progress((i + 1) as u64);
                }
            }

            let dedup_cmd = IdentifyUniqueSlidesCommand {
                video_id: Id::new(),
                frames: frames_with_hashes.clone(),
                slides_dir: slides_dir.clone(),
                similarity_threshold: command.similarity_threshold,
                selection_strategy: SelectionStrategy::Middle,
            };
            let (_dedup_summary, slide_preserved_events) =
                handle_identify_unique_slides(dedup_cmd)?;
            info!("Found {} unique slides.", slide_preserved_events.len());

            // Preserve Slide Images
            SlideSelector::preserve_slides(&slide_preserved_events, &frames_with_hashes)?;

            state.slides_dir = Some(slides_dir.clone());
            state.status = SessionStatus::UniqueSlidesIdentified;
            Self::save_state(&state_path, &state)?;
        }

        // 5. OCR & Generate Document
        if state.status == SessionStatus::UniqueSlidesIdentified {
            if let Some(p) = &progress {
                p.lock().await.set_stage("Running OCR on slides...");
            }
            info!("Step 5: Performing OCR and generating report...");

            let mut slides_data = Vec::new();
            let entries = fs::read_dir(state.slides_dir.as_ref().unwrap()).map_err(|e| {
                crate::shared::domain::ExtractionError::FileSystemError(format!(
                    "Failed to read slides directory: {}",
                    e
                ))
            })?;

            let mut entries_vec: Vec<_> = entries.filter_map(Result::ok).collect();
            entries_vec.sort_by_key(|e| e.path());

            if let Some(p) = &progress {
                p.lock().await.start_progress(entries_vec.len() as u64);
            }

            for (i, entry) in entries_vec.into_iter().enumerate() {
                let path = entry.path();
                let ocr_cmd = ExtractTextCommand {
                    slide_id: Id::new(),
                    image_path: path.to_str().unwrap().to_string(),
                    languages: command.languages.clone(),
                    confidence_threshold: command.confidence_threshold,
                };

                let (ocr_result, _) = handle_extract_text(ocr_cmd)?;

                slides_data.push(SlideData {
                    slide_index: (i + 1) as u32,
                    timestamp: 0.0, // Should ideally recover from frame metadata
                    image_path: path.to_str().unwrap().to_string(),
                    text: ocr_result.text,
                });
                if let Some(p) = &progress {
                    p.lock().await.update_progress((i + 1) as u64);
                }
            }

            if let Some(p) = &progress {
                p.lock().await.set_stage("Generating report...");
            }
            let doc_cmd = GenerateDocumentCommand {
                video_id: Id::new(),
                title: metadata.title,
                url: command.youtube_url,
                duration: metadata.duration,
                slides: slides_data,
                output_path: doc_path.clone(),
                include_timeline_diagram: true,
            };

            let doc_event = handle_generate_document(doc_cmd)?;

            state.report_path = Some(doc_event.file_path);
            state.status = SessionStatus::Completed;
            Self::save_state(&state_path, &state)?;
        }

        // 6. Cleanup
        if let Some(p) = &progress {
            p.lock().await.set_stage("Cleaning up...");
        }
        info!("Step 6: Cleaning up temporary resources...");
        if let Some(video_path) = &state.video_path {
            let _ = fs::remove_file(video_path);
        }
        if let Some(frames_dir) = &state.frames_dir {
            let _ = fs::remove_dir_all(frames_dir);
        }

        if let Some(p) = &progress {
            p.lock().await.finish_all();
        }

        Ok(DocumentGenerated {
            video_id: Id::new(),
            file_path: state.report_path.unwrap(),
            slide_count: 0, // Could be accurately tracked if needed
        })
    }

    fn save_state(path: &str, state: &SessionState) -> DomainResult<()> {
        let content = serde_json::to_string_pretty(state).map_err(|e| {
            crate::shared::domain::ExtractionError::InternalError(format!(
                "Failed to serialize session state: {}",
                e
            ))
        })?;
        fs::write(path, content).map_err(|e| {
            crate::shared::domain::ExtractionError::FileSystemError(format!(
                "Failed to write session state: {}",
                e
            ))
        })?;
        Ok(())
    }
}
