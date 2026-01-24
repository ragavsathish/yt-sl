use crate::contexts::transcription::domain::commands::TranscribeAudioCommand;
use crate::contexts::transcription::domain::ports::TranscriberPort;
use crate::contexts::transcription::domain::state::{TranscriptionResult, TranscriptionSegment};
use crate::shared::domain::{DomainResult, ExtractionError};
use async_trait::async_trait;
use reqwest::{multipart::Form, Client};
use serde::Deserialize;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

#[derive(Debug, Deserialize)]
struct OpenAiSegment {
    start: f64,
    end: f64,
    text: String,
}

#[derive(Debug, Deserialize)]
struct WhisperResponse {
    text: String,
    segments: Option<Vec<OpenAiSegment>>,
}

pub struct OpenAiTranscriber {
    client: Client,
    base_url: String,
    api_key: String,
}

impl OpenAiTranscriber {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(300)) // 5 minutes timeout for large files
                .build()
                .unwrap_or_default(),
            base_url,
            api_key,
        }
    }
}

#[async_trait]
impl TranscriberPort for OpenAiTranscriber {
    async fn transcribe(
        &self,
        command: TranscribeAudioCommand,
    ) -> DomainResult<TranscriptionResult> {
        let mut file = File::open(&command.audio_path).await.map_err(|e| {
            ExtractionError::TranscriptionFailed(format!("Failed to open audio file: {}", e))
        })?;

        let mut file_content = Vec::new();
        file.read_to_end(&mut file_content).await.map_err(|e| {
            ExtractionError::TranscriptionFailed(format!("Failed to read audio file: {}", e))
        })?;

        let part = reqwest::multipart::Part::bytes(file_content)
            .file_name("audio.wav")
            .mime_str("audio/wav")
            .map_err(|e| ExtractionError::TranscriptionFailed(e.to_string()))?;

        let form = Form::new()
            .part("file", part)
            .text("model", "whisper-1")
            .text("response_format", "verbose_json");

        let response = self
            .client
            .post(format!("{}/audio/transcriptions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .multipart(form)
            .send()
            .await
            .map_err(|e| ExtractionError::TranscriptionFailed(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExtractionError::TranscriptionFailed(format!(
                "API Error: {}",
                response.status()
            )));
        }

        let whisper_response: WhisperResponse = response.json().await.map_err(|e| {
            ExtractionError::TranscriptionFailed(format!("Failed to parse response: {}", e))
        })?;

        let segments = whisper_response
            .segments
            .unwrap_or_default()
            .into_iter()
            .map(|s| TranscriptionSegment {
                start: s.start,
                end: s.end,
                text: s.text,
            })
            .collect();

        Ok(TranscriptionResult {
            segments,
            full_text: whisper_response.text,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain::{Id, YouTubeVideo};
    use std::io::Write;
    use tempfile::NamedTempFile;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_transcribe_success() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/audio/transcriptions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "text": "Hello world",
                "segments": [
                    {
                        "start": 0.0,
                        "end": 1.0,
                        "text": "Hello world"
                    }
                ]
            })))
            .mount(&mock_server)
            .await;

        let transcriber = OpenAiTranscriber::new(mock_server.uri(), "test-key".to_string());

        // Create a dummy audio file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "dummy audio content").unwrap();
        let audio_path = temp_file.path().to_str().unwrap().to_string();

        let command = TranscribeAudioCommand {
            video_id: Id::<YouTubeVideo>::new(),
            audio_path,
        };

        let result = transcriber.transcribe(command).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.full_text, "Hello world");
        assert_eq!(result.segments.len(), 1);
        assert_eq!(result.segments[0].text, "Hello world");
    }
}
