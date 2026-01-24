use crate::contexts::transcription::domain::commands::{
    ExtractAudioCommand, TranscribeAudioCommand,
};
use crate::contexts::transcription::domain::events::{AudioExtracted, TextTranscribed};
use crate::contexts::transcription::domain::ports::{AudioExtractorPort, TranscriberPort};
use crate::shared::domain::{DomainResult, Id, YouTubeVideo};
use std::sync::Arc;

pub struct TranscriptionHandler {
    extractor: Arc<dyn AudioExtractorPort>,
    transcriber: Arc<dyn TranscriberPort>,
}

impl TranscriptionHandler {
    pub fn new(
        extractor: Arc<dyn AudioExtractorPort>,
        transcriber: Arc<dyn TranscriberPort>,
    ) -> Self {
        Self {
            extractor,
            transcriber,
        }
    }

    pub async fn handle(
        &self,
        video_id: Id<YouTubeVideo>,
        video_path: String,
        output_dir: String,
    ) -> DomainResult<(AudioExtracted, TextTranscribed)> {
        let extract_command = ExtractAudioCommand {
            video_id: video_id.clone(),
            video_path,
            output_dir,
        };

        let audio_path = self.extractor.extract_audio(extract_command).await?;
        let audio_extracted_event = AudioExtracted {
            video_id: video_id.clone(),
            audio_path: audio_path.clone(),
        };

        let transcribe_command = TranscribeAudioCommand {
            video_id: video_id.clone(),
            audio_path,
        };

        let result = self.transcriber.transcribe(transcribe_command).await?;
        let text_transcribed_event = TextTranscribed { video_id, result };

        Ok((audio_extracted_event, text_transcribed_event))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::transcription::domain::ports::{
        MockAudioExtractorPort, MockTranscriberPort,
    };
    use crate::contexts::transcription::domain::state::TranscriptionResult;
    use mockall::predicate;

    #[tokio::test]
    async fn test_transcription_flow() {
        let video_id = Id::<YouTubeVideo>::new();
        let video_path = "/path/to/video.mp4".to_string();
        let output_dir = "/path/to/output".to_string();
        let expected_audio_path = "/path/to/output/video.wav".to_string();
        let expected_result = TranscriptionResult {
            segments: vec![],
            full_text: "Transcribed text".to_string(),
        };

        let mut mock_extractor = MockAudioExtractorPort::new();
        mock_extractor
            .expect_extract_audio()
            .with(predicate::function(|cmd: &ExtractAudioCommand| {
                cmd.video_path == "/path/to/video.mp4"
            }))
            .times(1)
            .returning(move |_| Ok(expected_audio_path.clone()));

        let mut mock_transcriber = MockTranscriberPort::new();
        let result_clone = expected_result.clone();
        mock_transcriber
            .expect_transcribe()
            .with(predicate::function(|cmd: &TranscribeAudioCommand| {
                cmd.audio_path == "/path/to/output/video.wav"
            }))
            .times(1)
            .returning(move |_| Ok(result_clone.clone()));

        let handler =
            TranscriptionHandler::new(Arc::new(mock_extractor), Arc::new(mock_transcriber));

        let result = handler.handle(video_id, video_path, output_dir).await;

        assert!(result.is_ok());
        let (audio_event, text_event) = result.unwrap();
        assert_eq!(audio_event.audio_path, "/path/to/output/video.wav");
        assert_eq!(text_event.result.full_text, "Transcribed text");
    }
}
