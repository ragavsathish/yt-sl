use crate::contexts::transcription::domain::commands::{
    ExtractAudioCommand, TranscribeAudioCommand,
};
use crate::contexts::transcription::domain::state::TranscriptionResult;
use crate::shared::domain::DomainResult;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait AudioExtractorPort: Send + Sync {
    async fn extract_audio(&self, command: ExtractAudioCommand) -> DomainResult<String>;
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait TranscriberPort: Send + Sync {
    async fn transcribe(
        &self,
        command: TranscribeAudioCommand,
    ) -> DomainResult<TranscriptionResult>;
}
