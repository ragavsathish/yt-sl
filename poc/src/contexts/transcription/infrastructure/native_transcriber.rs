use crate::contexts::transcription::domain::commands::TranscribeAudioCommand;
use crate::contexts::transcription::domain::ports::TranscriberPort;
use crate::contexts::transcription::domain::state::{TranscriptionResult, TranscriptionSegment};
use crate::shared::domain::{DomainResult, ExtractionError};
use async_trait::async_trait;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

pub struct NativeWhisperTranscriber {
    model_path: PathBuf,
}

impl NativeWhisperTranscriber {
    pub async fn new() -> DomainResult<Self> {
        let model_path = Self::ensure_model_exists().await?;
        Ok(Self { model_path })
    }

    async fn ensure_model_exists() -> DomainResult<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("yt-sl/models");

        tokio::fs::create_dir_all(&cache_dir).await.map_err(|e| {
            ExtractionError::FileSystemError(format!("Failed to create model cache dir: {}", e))
        })?;

        let model_name = "ggml-small.en.bin";
        let model_path = cache_dir.join(model_name);

        if !model_path.exists() {
            let url = format!(
                "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
                model_name
            );

            // Download using reqwest
            let response = reqwest::get(&url)
                .await
                .map_err(|_e| ExtractionError::NetworkTimeout(std::time::Duration::from_secs(0)))?;

            if !response.status().is_success() {
                return Err(ExtractionError::DownloadFailed(
                    0,
                    format!("Failed to download model: {}", response.status()),
                ));
            }

            let content = response
                .bytes()
                .await
                .map_err(|e| ExtractionError::DownloadFailed(0, e.to_string()))?;

            // Blocking file write
            let path_clone = model_path.clone();
            tokio::task::spawn_blocking(move || {
                let mut file = File::create(&path_clone).map_err(|e| e.to_string())?;
                file.write_all(&content).map_err(|e| e.to_string())
            })
            .await
            .map_err(|e| ExtractionError::InternalError(e.to_string()))?
            .map_err(|e| {
                ExtractionError::FileSystemError(format!("Failed to write model: {}", e))
            })?;
        }

        Ok(model_path)
    }
}

#[async_trait]
impl TranscriberPort for NativeWhisperTranscriber {
    async fn transcribe(
        &self,
        command: TranscribeAudioCommand,
    ) -> DomainResult<TranscriptionResult> {
        let model_path = self.model_path.clone();
        let audio_path = command.audio_path.clone();

        tokio::task::spawn_blocking(move || {
            // Load context
            let ctx = WhisperContext::new_with_params(
                &model_path.to_string_lossy(),
                WhisperContextParameters::default(),
            )
            .map_err(|e| {
                ExtractionError::TranscriptionFailed(format!("Failed to load model: {}", e))
            })?;

            let mut state = ctx.create_state().map_err(|e| {
                ExtractionError::TranscriptionFailed(format!("Failed to create state: {}", e))
            })?;

            // Read Audio
            let reader = hound::WavReader::open(&audio_path).map_err(|e| {
                ExtractionError::TranscriptionFailed(format!("Failed to open WAV: {}", e))
            })?;

            let samples: Vec<i16> = reader
                .into_samples::<i16>()
                .filter_map(Result::ok)
                .collect();
            let samples_f32: Vec<f32> = samples.iter().map(|&s| s as f32 / 32768.0).collect();

            // Transcribe
            let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
            params.set_language(Some("en"));
            params.set_print_special(false);
            params.set_print_progress(false);
            params.set_print_realtime(false);
            params.set_print_timestamps(true); // Enable timestamps

            state.full(params, &samples_f32).map_err(|e| {
                ExtractionError::TranscriptionFailed(format!("Failed to run whisper: {}", e))
            })?;

            // Extract text
            let num_segments = state.full_n_segments().map_err(|e| {
                ExtractionError::TranscriptionFailed(format!("Failed to get segments: {}", e))
            })?;

            let mut segments = Vec::new();
            let mut full_text = String::new();

            for i in 0..num_segments {
                let segment_text = state.full_get_segment_text(i).map_err(|e| {
                    ExtractionError::TranscriptionFailed(format!(
                        "Failed to get segment text: {}",
                        e
                    ))
                })?;

                let t0 = state.full_get_segment_t0(i).map_err(|e| {
                    ExtractionError::TranscriptionFailed(format!("Failed to get t0: {}", e))
                })?;

                let t1 = state.full_get_segment_t1(i).map_err(|e| {
                    ExtractionError::TranscriptionFailed(format!("Failed to get t1: {}", e))
                })?;

                // Whisper returns time in 10ms units (centiseconds)
                let start = t0 as f64 / 100.0;
                let end = t1 as f64 / 100.0;

                segments.push(TranscriptionSegment {
                    start,
                    end,
                    text: segment_text.clone(),
                });

                full_text.push_str(&segment_text);
                full_text.push(' ');
            }

            Ok(TranscriptionResult {
                segments,
                full_text: full_text.trim().to_string(),
            })
        })
        .await
        .map_err(|e| ExtractionError::InternalError(format!("Task failed: {}", e)))?
    }
}
