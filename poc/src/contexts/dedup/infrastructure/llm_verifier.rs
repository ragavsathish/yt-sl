use crate::shared::domain::config::LlmConfig;
use crate::shared::domain::{DomainResult, ExtractionError};
use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};
use std::fs;
use tracing::{debug, info, warn};

/// Infrastructure component for verifying slides using a Cloud LLM (OpenAI-compatible).
pub struct LlmVerifier;

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: Vec<ContentPart>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ContentPart {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Deserialize)]
struct MessageResponse {
    content: String,
}

impl LlmVerifier {
    /// Verifies if an image contains a slide using a Vision-capable LLM.
    pub async fn verify_slide(image_path: &str, config: &LlmConfig) -> DomainResult<bool> {
        info!("Verifying slide with LLM: {}", image_path);

        // 1. Read and encode image
        let image_data = fs::read(image_path).map_err(|e| {
            ExtractionError::FileSystemError(format!("Failed to read image for LLM: {}", e))
        })?;
        let base64_image = general_purpose::STANDARD.encode(image_data);
        let data_url = format!("data:image/jpeg;base64,{}", base64_image);

        // 2. Prepare request
        let client = reqwest::Client::new();
        let request = ChatCompletionRequest {
            model: config.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: vec![
                    ContentPart::Text {
                        text: config.prompt.clone(),
                    },
                    ContentPart::ImageUrl {
                        image_url: ImageUrl { url: data_url },
                    },
                ],
            }],
        };

        // 3. Send request
        let mut request_builder = client
            .post(format!("{}/chat/completions", config.api_base))
            .json(&request);

        if let Some(key) = &config.api_key {
            request_builder = request_builder.bearer_auth(key);
        }

        let response = request_builder.send().await.map_err(|e| {
            ExtractionError::ExternalDependencyUnavailable(format!("LLM request failed: {}", e))
        })?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(ExtractionError::ExternalDependencyUnavailable(format!(
                "LLM returned error ({}): {}",
                status, error_text
            )));
        }

        let completion: ChatCompletionResponse = response.json().await.map_err(|e| {
            ExtractionError::InternalError(format!("Failed to parse LLM response: {}", e))
        })?;

        // 4. Parse result
        let content = completion
            .choices
            .first()
            .map(|c| c.message.content.trim().to_uppercase())
            .unwrap_or_default();

        debug!("LLM Verification Result for {}: {}", image_path, content);

        if content.contains("NOT_SLIDE") {
            Ok(false)
        } else if content.contains("SLIDE") {
            Ok(true)
        } else {
            // If the LLM is ambiguous, we default to requiring review
            warn!("Ambiguous LLM response for {}: {}", image_path, content);
            Ok(false)
        }
    }
}
