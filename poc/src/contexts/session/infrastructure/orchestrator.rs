use crate::cli::CliProgressReporter;
use crate::contexts::dedup::domain::commands::{
    FrameDedupMetadata, IdentifyUniqueSlidesCommand, SelectionStrategy,
};
use crate::contexts::dedup::domain::handlers::handle_identify_unique_slides;
use crate::contexts::dedup::infrastructure::{LlmVerifier, SlideSelector};
use crate::contexts::document::domain::commands::{GenerateDocumentCommand, SlideData};
use crate::contexts::document::domain::handlers::handle_generate_document;
use crate::contexts::frame::domain::commands::{
    ComputeHashCommand, ExtractFramesCommand, FrameFormat, HashAlgorithm,
};
use crate::contexts::frame::infrastructure::{FrameExtractor, PerceptualHasher};
use crate::contexts::ocr::domain::commands::ExtractTextCommand;
use crate::contexts::ocr::domain::handlers::handle_extract_text;
use crate::contexts::session::domain::commands::StartExtractionSessionCommand;
use crate::contexts::session::domain::events::{DocumentGenerated, OcrStats};
use crate::contexts::session::domain::state::{SessionState, SessionStatus};
use crate::contexts::transcription::domain::handlers::TranscriptionHandler;
use crate::contexts::transcription::domain::ports::TranscriberPort;
use crate::contexts::transcription::infrastructure::{
    audio_extractor::AudioExtractor, native_transcriber::NativeWhisperTranscriber,
    openai_transcriber::OpenAiTranscriber,
};
use crate::contexts::video::domain::commands::DownloadVideoCommand;
use crate::contexts::video::infrastructure::{AvailabilityChecker, UrlValidator, VideoDownloader};
use crate::shared::domain::{DomainResult, ExtractionError, Id};
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

const CACHE_DIR: &str = "/tmp/yt-sl-cache";

/// Central orchestrator for the video slide extraction pipeline.
pub struct SessionOrchestrator;

impl SessionOrchestrator {
    /// Executes the full extraction pipeline from URL to Markdown document.
    /// Supports session recovery if a session state file exists.
    pub async fn run_session(
        command: StartExtractionSessionCommand,
        progress: Option<Arc<Mutex<CliProgressReporter>>>,
    ) -> DomainResult<DocumentGenerated> {
        let session_id = command.session_id.clone();
        let session_dir = format!("{}/{}", command.output_dir, session_id);
        let frames_dir = format!("{}/frames", session_dir);
        let slides_dir = format!("{}/slides", session_dir);
        let doc_path = format!("{}/report.md", session_dir);
        let cleaned_doc_path = format!("{}/report_cleaned.md", session_dir);
        let state_path = format!("{}/session.json", session_dir);
        let validator = UrlValidator::new();
        let keep_temp = command.keep_temp;

        // Ensure session directory exists
        fs::create_dir_all(&session_dir).map_err(|e| {
            ExtractionError::FileSystemError(format!("Failed to create session directory: {}", e))
        })?;

        // Try to load existing state
        let mut state = if Path::new(&state_path).exists() {
            info!("Existing session found. Attempting recovery...");
            let content = fs::read_to_string(&state_path).map_err(|e| {
                ExtractionError::FileSystemError(format!("Failed to read session state: {}", e))
            })?;
            serde_json::from_str::<SessionState>(&content).unwrap_or_else(|e| {
                warn!("Failed to parse session state: {}. Starting fresh.", e);
                SessionState::new(session_id.clone())
            })
        } else {
            SessionState::new(session_id.clone())
        };

        // Track total frames for summary
        let mut total_frames: u32 = 0;

        // Run pipeline with cleanup on failure
        let result = Self::run_pipeline(
            &command,
            &mut state,
            &state_path,
            &frames_dir,
            &slides_dir,
            &doc_path,
            &cleaned_doc_path,
            &validator,
            &progress,
            &mut total_frames,
        )
        .await;

        // Cleanup on failure
        if result.is_err() && !keep_temp {
            info!("Cleaning up temporary resources after failure...");
            if let Some(frames_dir) = &state.frames_dir {
                let _ = fs::remove_dir_all(frames_dir);
            }
            if let Some(video_path) = &state.video_path {
                if !video_path.contains(CACHE_DIR) {
                    let _ = fs::remove_file(video_path);
                }
            }
            // Save failed state
            state.status = SessionStatus::Failed(
                result
                    .as_ref()
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_default(),
            );
            let _ = Self::save_state(&state_path, &state);
        }

        if let Some(p) = &progress {
            p.lock().await.finish_all();
        }

        result
    }

    #[allow(clippy::too_many_arguments)]
    async fn run_pipeline(
        command: &StartExtractionSessionCommand,
        state: &mut SessionState,
        state_path: &str,
        frames_dir: &str,
        slides_dir: &str,
        doc_path: &str,
        cleaned_doc_path: &str,
        validator: &UrlValidator,
        progress: &Option<Arc<Mutex<CliProgressReporter>>>,
        total_frames: &mut u32,
    ) -> DomainResult<DocumentGenerated> {
        // 1. Validate URL & Fetch Metadata
        if state.status == SessionStatus::Starting {
            if let Some(p) = progress {
                p.lock().await.set_stage("Validating URL...");
            }
            info!("Step 1: Validating URL and fetching metadata...");
            let (url, video_id) = validator.validate_and_extract(&command.youtube_url)?;

            fs::create_dir_all(CACHE_DIR).map_err(|e| {
                ExtractionError::FileSystemError(format!("Failed to create cache directory: {}", e))
            })?;

            let checker = AvailabilityChecker::new();
            let metadata = checker.check_availability(&video_id, &url).await?;
            info!("Video found: '{}' ({}s)", metadata.title, metadata.duration);

            state.video_metadata = Some(metadata);
            state.status = SessionStatus::MetadataFetched;
            Self::save_state(state_path, state)?;
        }

        let metadata = state.video_metadata.as_ref().unwrap().clone();

        // 2. Download Video
        if state.status == SessionStatus::MetadataFetched {
            if let Some(p) = progress {
                p.lock().await.set_stage("Downloading video...");
            }
            info!("Step 2: Downloading video...");

            let raw_id = validator
                .extract_video_id_public(&command.youtube_url)
                .unwrap_or_else(|_| "unknown".to_string());

            let url = command.youtube_url.clone();
            let downloader = VideoDownloader::new();
            let download_cmd = DownloadVideoCommand {
                video_id: Id::new(),
                youtube_video_id: raw_id.clone(),
            };

            let download_event = downloader
                .download_video(download_cmd, &url, CACHE_DIR)
                .await?;

            state.video_path = Some(download_event.path);
            state.status = SessionStatus::VideoDownloaded;
            Self::save_state(state_path, state)?;
        }
        info!("Video available at: {}", state.video_path.as_ref().unwrap());

        // 2b. Transcribe Audio
        if state.status == SessionStatus::VideoDownloaded {
            if let Some(p) = progress {
                p.lock().await.set_stage("Transcribing audio...");
            }
            info!("Step 2b: Extracting audio and transcribing...");

            let raw_id = validator
                .extract_video_id_public(&command.youtube_url)
                .unwrap_or_else(|_| "unknown".to_string());

            let audio_extractor = Arc::new(AudioExtractor::new());

            let transcriber: Arc<dyn TranscriberPort> =
                if let Ok(base_url) = std::env::var("WHISPER_API_BASE_URL") {
                    let api_key = std::env::var("WHISPER_API_KEY")
                        .unwrap_or_else(|_| "sk-placeholder".to_string());
                    info!("Using OpenAI Whisper API at {}", base_url);
                    Arc::new(OpenAiTranscriber::new(base_url, api_key))
                } else {
                    info!("Using Native Whisper (local model)");
                    Arc::new(NativeWhisperTranscriber::new().await?)
                };

            let handler = TranscriptionHandler::new(audio_extractor, transcriber);

            let (_audio_event, text_event) = handler
                .handle(
                    Id::new(),
                    raw_id,
                    state.video_path.as_ref().unwrap().clone(),
                    CACHE_DIR.to_string(),
                )
                .await?;

            state.transcription = Some(text_event.result);
            state.status = SessionStatus::AudioTranscribed;
            Self::save_state(state_path, state)?;
        }

        // 3. Extract Frames
        if state.status == SessionStatus::AudioTranscribed {
            if let Some(p) = progress {
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
                output_dir: frames_dir.to_string(),
                interval_secs: command.frame_interval_secs,
                output_format: FrameFormat::Jpeg,
                jpeg_quality: Some(85),
            };
            let (frames_extracted, _frames) = extractor
                .extract_frames(extract_cmd, metadata.duration)
                .await?;

            *total_frames = frames_extracted.total_frames;

            // Validate frames and skip corrupt ones
            let frame_files: Vec<_> = fs::read_dir(frames_dir)
                .map_err(|e| {
                    ExtractionError::FileSystemError(format!(
                        "Failed to read frames directory: {}",
                        e
                    ))
                })?
                .filter_map(Result::ok)
                .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("jpg"))
                .collect();

            let total_frame_count = frame_files.len();
            let mut corrupt_count = 0;
            for entry in &frame_files {
                let path = entry.path();
                if FrameExtractor::validate_frame(path.to_str().unwrap_or("")).is_err() {
                    warn!("Corrupt frame detected: {:?}. Removing.", path);
                    let _ = fs::remove_file(&path);
                    corrupt_count += 1;
                }
            }

            if total_frame_count > 0 {
                let corrupt_ratio = corrupt_count as f64 / total_frame_count as f64;
                if corrupt_ratio > 0.1 {
                    return Err(ExtractionError::TooManyCorruptFrames {
                        count: corrupt_count,
                        max: (total_frame_count as f64 * 0.1) as u32,
                    });
                }
            }

            if corrupt_count > 0 {
                info!(
                    "Skipped {} corrupt frames out of {}.",
                    corrupt_count, total_frame_count
                );
            }

            state.frames_dir = Some(frames_dir.to_string());
            state.status = SessionStatus::FramesExtracted;
            Self::save_state(state_path, state)?;
        }
        info!(
            "Frames extracted to: {}",
            state.frames_dir.as_ref().unwrap()
        );

        // 4. Compute Hashes & Identify Unique Slides
        if state.status == SessionStatus::FramesExtracted {
            if let Some(p) = progress {
                p.lock().await.set_stage("Deduplicating slides...");
            }
            info!("Step 4: Computing hashes and identifying unique slides...");

            let mut frames_with_hashes = Vec::new();

            let entries = fs::read_dir(state.frames_dir.as_ref().unwrap()).map_err(|e| {
                ExtractionError::FileSystemError(format!("Failed to read frames directory: {}", e))
            })?;

            let hasher = PerceptualHasher::new();
            let mut entries_vec: Vec<_> = entries.filter_map(Result::ok).collect();
            entries_vec.sort_by_key(|e| e.path());

            if let Some(p) = progress {
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
                if let Some(p) = progress {
                    p.lock().await.update_progress((i + 1) as u64);
                }
            }

            let dedup_cmd = IdentifyUniqueSlidesCommand {
                video_id: Id::new(),
                frames: frames_with_hashes.clone(),
                slides_dir: slides_dir.to_string(),
                similarity_threshold: command.similarity_threshold,
                selection_strategy: SelectionStrategy::Middle,
            };
            let (_dedup_summary, slide_preserved_events) =
                handle_identify_unique_slides(dedup_cmd)?;

            if let Some(p) = progress {
                let msg = format!(
                    "Found {} unique slides from {} frames",
                    slide_preserved_events.len(),
                    frames_with_hashes.len()
                );
                p.lock().await.set_stage(&msg);
            }
            info!("Found {} unique slides.", slide_preserved_events.len());

            // Preserve Slide Images
            SlideSelector::preserve_slides(&slide_preserved_events, &frames_with_hashes)?;

            // Save slides metadata to state
            let mut slides_state = Vec::new();
            for event in &slide_preserved_events {
                let frame = frames_with_hashes
                    .iter()
                    .find(|f| f.frame_id == event.frame_id)
                    .unwrap();
                slides_state.push(crate::contexts::session::domain::state::SlideState {
                    slide_index: event.slide_index,
                    timestamp: frame.timestamp,
                    image_path: event.slide_path.clone(),
                    requires_human_review: false,
                });
            }
            slides_state.sort_by_key(|s| s.slide_index);
            state.slides = slides_state;

            state.slides_dir = Some(slides_dir.to_string());
            state.status = SessionStatus::UniqueSlidesIdentified;
            Self::save_state(state_path, state)?;
        }

        // 4b. Optional LLM Verification
        if state.status == SessionStatus::UniqueSlidesIdentified {
            if let Some(llm_config) = &command.llm_config {
                if let Some(p) = progress {
                    p.lock().await.set_stage("Verifying slides with LLM...");
                }
                info!("Step 4b: Verifying identified slides with Cloud LLM...");

                let mut join_set = tokio::task::JoinSet::new();

                if let Some(p) = progress {
                    p.lock().await.start_progress(state.slides.len() as u64);
                }

                for slide in &state.slides {
                    let image_path = slide.image_path.clone();
                    let config = llm_config.clone();
                    let slide_index = slide.slide_index;
                    join_set.spawn(async move {
                        let result = LlmVerifier::verify_slide(&image_path, &config).await;
                        (slide_index, result)
                    });
                }

                let mut verified_count = 0;
                while let Some(res) = join_set.join_next().await {
                    match res {
                        Ok((index, Ok(is_slide))) => {
                            if let Some(slide) =
                                state.slides.iter_mut().find(|s| s.slide_index == index)
                            {
                                if !is_slide {
                                    info!("Slide {} identified as NOT a slide by LLM.", index);
                                    slide.requires_human_review = true;
                                }
                            }
                        }
                        Ok((index, Err(e))) => {
                            warn!(
                                "LLM verification failed for slide {}: {}. Skipping.",
                                index, e
                            );
                        }
                        Err(e) => {
                            warn!("LLM verification task panicked: {}.", e);
                        }
                    }
                    verified_count += 1;
                    if let Some(p) = progress {
                        p.lock().await.update_progress(verified_count);
                    }
                }

                Self::save_state(state_path, state)?;
            }
        }

        // 5. OCR & Generate Document
        if state.status == SessionStatus::UniqueSlidesIdentified {
            if let Some(p) = progress {
                p.lock().await.set_stage("Running OCR on slides...");
            }
            info!("Step 5: Performing OCR and generating report...");

            let mut slides_data = Vec::new();
            let languages = command.languages.clone();

            if let Some(p) = progress {
                p.lock().await.start_progress(state.slides.len() as u64);
            }

            // OCR stats tracking
            let mut ocr_success_count: u32 = 0;
            let mut ocr_failure_count: u32 = 0;
            let mut confidence_sum: f64 = 0.0;
            let mut low_confidence_count: u32 = 0;

            for (i, slide_state) in state.slides.iter().enumerate() {
                let ocr_cmd = ExtractTextCommand {
                    slide_id: Id::new(),
                    image_path: slide_state.image_path.clone(),
                    languages: languages.clone(),
                    confidence_threshold: command.confidence_threshold,
                };

                let (ocr_text, ocr_confidence, is_low_confidence) =
                    match handle_extract_text(ocr_cmd) {
                        Ok((result, low_conf)) => {
                            ocr_success_count += 1;
                            confidence_sum += result.confidence;
                            let is_low = low_conf.is_some();
                            if is_low {
                                low_confidence_count += 1;
                            }
                            (result.text, result.confidence, is_low)
                        }
                        Err(e) => {
                            warn!("OCR failed for slide {}: {}. Using placeholder.", i + 1, e);
                            ocr_failure_count += 1;
                            ("[OCR failed for this slide]".to_string(), 0.0, false)
                        }
                    };

                // Check OCR failure threshold (20%)
                let total_attempted = ocr_success_count + ocr_failure_count;
                if total_attempted > 0 {
                    let failure_rate = ocr_failure_count as f64 / total_attempted as f64;
                    if failure_rate > 0.2 && total_attempted >= 5 {
                        return Err(ExtractionError::OcrFailed(
                            Id::new(),
                            format!(
                                "OCR failure rate ({:.0}%) exceeds 20% threshold. {} of {} slides failed.",
                                failure_rate * 100.0,
                                ocr_failure_count,
                                total_attempted
                            ),
                        ));
                    }
                }

                // Map transcription segments to this slide
                let mut slide_transcription = None;
                if let Some(transcription) = &state.transcription {
                    let start_time = slide_state.timestamp;
                    let end_time = if i + 1 < state.slides.len() {
                        state.slides[i + 1].timestamp
                    } else {
                        metadata.duration as f64
                    };

                    let texts: Vec<String> = transcription
                        .segments
                        .iter()
                        .filter(|s| s.start >= start_time && s.start < end_time)
                        .map(|s| s.text.clone())
                        .collect();

                    if !texts.is_empty() {
                        slide_transcription = Some(texts.join(" "));
                    }
                }

                slides_data.push(SlideData {
                    slide_index: slide_state.slide_index,
                    timestamp: slide_state.timestamp,
                    image_path: slide_state.image_path.clone(),
                    text: ocr_text,
                    confidence: ocr_confidence,
                    is_low_confidence,
                    transcription: slide_transcription,
                    requires_human_review: slide_state.requires_human_review,
                });
                if let Some(p) = progress {
                    p.lock().await.update_progress((i + 1) as u64);
                }
            }

            let ocr_total = ocr_success_count + ocr_failure_count;
            let ocr_stats = if ocr_total > 0 {
                Some(OcrStats {
                    success_rate: ocr_success_count as f64 / ocr_total as f64,
                    avg_confidence: if ocr_success_count > 0 {
                        confidence_sum / ocr_success_count as f64
                    } else {
                        0.0
                    },
                    low_confidence_count,
                    failure_count: ocr_failure_count,
                })
            } else {
                None
            };

            if let Some(p) = progress {
                p.lock().await.set_stage("Generating reports...");
            }

            // Report 1: Full report with warnings
            let doc_cmd = GenerateDocumentCommand {
                video_id: Id::new(),
                title: metadata.title.clone(),
                url: command.youtube_url.clone(),
                duration: metadata.duration,
                slides: slides_data.clone(),
                transcription: state.transcription.as_ref().map(|t| t.full_text.clone()),
                output_path: doc_path.to_string(),
                include_timeline_diagram: true,
                generate_pdf: command.generate_pdf,
                pdf_template: command.pdf_template.clone(),
                template: command.template.clone(),
                session_id: Some(command.session_id.to_string()),
            };

            let doc_event = handle_generate_document(doc_cmd)?;
            state.report_path = Some(doc_event.file_path);
            let pdf_path = doc_event.pdf_path;

            // Report 2: Cleaned report removing non-slides
            let cleaned_slides_data: Vec<SlideData> = slides_data
                .into_iter()
                .filter(|s| !s.requires_human_review)
                .collect();

            let mut cleaned_pdf_path = None;
            if !cleaned_slides_data.is_empty() {
                let cleaned_doc_cmd = GenerateDocumentCommand {
                    video_id: Id::new(),
                    title: format!("{} (Cleaned)", metadata.title),
                    url: command.youtube_url.clone(),
                    duration: metadata.duration,
                    slides: cleaned_slides_data,
                    transcription: state.transcription.as_ref().map(|t| t.full_text.clone()),
                    output_path: cleaned_doc_path.to_string(),
                    include_timeline_diagram: true,
                    generate_pdf: command.generate_pdf,
                    pdf_template: command.pdf_template.clone(),
                    template: command.template.clone(),
                    session_id: Some(command.session_id.to_string()),
                };

                let cleaned_doc_event = handle_generate_document(cleaned_doc_cmd)?;
                state.cleaned_report_path = Some(cleaned_doc_event.file_path);
                cleaned_pdf_path = cleaned_doc_event.pdf_path;
            }

            state.status = SessionStatus::Completed;
            Self::save_state(state_path, state)?;

            // 6. Cleanup
            if !command.keep_temp {
                if let Some(p) = progress {
                    p.lock().await.set_stage("Cleaning up...");
                }
                info!("Step 6: Cleaning up temporary resources...");
                if let Some(video_path) = &state.video_path {
                    if !video_path.contains(CACHE_DIR) {
                        let _ = fs::remove_file(video_path);
                    }
                }
                if let Some(frames_dir) = &state.frames_dir {
                    let _ = fs::remove_dir_all(frames_dir);
                }
            } else {
                info!("Keeping temporary files as requested.");
            }

            let review_slides: Vec<String> = state
                .slides
                .iter()
                .filter(|s| s.requires_human_review)
                .map(|s| s.image_path.clone())
                .collect();

            return Ok(DocumentGenerated {
                video_id: Id::new(),
                file_path: state.report_path.clone().unwrap(),
                pdf_path,
                cleaned_file_path: state.cleaned_report_path.clone(),
                cleaned_pdf_path,
                slide_count: state.slides.len() as u32,
                total_frames: *total_frames,
                review_count: review_slides.len() as u32,
                review_slides,
                ocr_stats,
            });
        }

        Err(ExtractionError::InternalError(
            "Session failed to reach completion state".to_string(),
        ))
    }

    fn save_state(path: &str, state: &SessionState) -> DomainResult<()> {
        let content = serde_json::to_string_pretty(state).map_err(|e| {
            ExtractionError::InternalError(format!("Failed to serialize session state: {}", e))
        })?;
        fs::write(path, content).map_err(|e| {
            ExtractionError::FileSystemError(format!("Failed to write session state: {}", e))
        })?;
        Ok(())
    }
}
