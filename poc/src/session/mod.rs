//! Session module for the YouTube Video Slide Extractor.
//!
//! This module provides session management functionality as specified in
//! US-SESSION-01: Create Processing Session.
//!
//! Features:
//! - Unique session ID generation
//! - Session state tracking (created, processing, completed, failed)
//! - Session metadata storage (start time, configuration, progress)
//! - Session persistence for recovery
//! - Thread-safe session management

use crate::shared::domain::config::ExtractionConfig;
use crate::shared::domain::error::{DomainResult, ExtractionError};
use crate::shared::domain::Id;
use crate::shared::infrastructure::logging::{
    log_error_with_context, log_info, session_span, Session as LoggingSession,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::Span;

/// Represents the state of a processing session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session has been created but not started
    Created,
    /// Session is actively processing
    Processing,
    /// Session completed successfully
    Completed,
    /// Session failed with an error
    Failed,
}

impl SessionState {
    /// Returns true if the session is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, SessionState::Completed | SessionState::Failed)
    }

    /// Returns true if the session can be started.
    pub fn can_start(&self) -> bool {
        matches!(self, SessionState::Created)
    }

    /// Returns true if the session can transition to processing.
    pub fn can_process(&self) -> bool {
        matches!(self, SessionState::Created)
    }

    /// Returns true if the session can be marked as completed.
    pub fn can_complete(&self) -> bool {
        matches!(self, SessionState::Processing)
    }

    /// Returns true if the session can be marked as failed.
    pub fn can_fail(&self) -> bool {
        matches!(self, SessionState::Created | SessionState::Processing)
    }
}

impl std::fmt::Display for SessionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionState::Created => write!(f, "Created"),
            SessionState::Processing => write!(f, "Processing"),
            SessionState::Completed => write!(f, "Completed"),
            SessionState::Failed => write!(f, "Failed"),
        }
    }
}

/// Represents the progress of a processing session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionProgress {
    /// Current processing stage
    pub stage: String,

    /// Percentage complete (0.0 - 1.0)
    pub percentage: f64,

    /// Number of items processed
    pub processed: u64,

    /// Total number of items to process
    pub total: u64,

    /// Optional message about current operation
    pub message: Option<String>,
}

impl SessionProgress {
    pub fn new(stage: impl Into<String>, total: u64) -> Self {
        Self {
            stage: stage.into(),
            percentage: 0.0,
            processed: 0,
            total,
            message: None,
        }
    }

    pub fn update(&mut self, processed: u64, message: Option<String>) {
        self.processed = processed;
        self.percentage = if self.total > 0 {
            (processed as f64 / self.total as f64).min(1.0)
        } else {
            0.0
        };
        self.message = message;
    }

    pub fn increment(&mut self) {
        self.update(self.processed.saturating_add(1), None);
    }
}

impl Default for SessionProgress {
    fn default() -> Self {
        Self::new("Initializing", 0)
    }
}

/// Represents a processing session.
///
/// This struct contains all session metadata and state as specified in
/// US-SESSION-01.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingSession {
    pub id: Id<Session>,
    pub youtube_url: String,
    pub config: ExtractionConfig,
    pub state: SessionState,
    /// Session creation timestamp (Unix timestamp in seconds)
    pub created_at: u64,
    /// Session completion timestamp (None if not completed)
    pub completed_at: Option<u64>,
    pub progress: SessionProgress,
    pub error_message: Option<String>,
    pub unique_slides: u64,
    pub frames_processed: u64,
    pub metadata: HashMap<String, String>,
}

impl ProcessingSession {
    /// Creates a new processing session.
    ///
    /// This method creates a new session with a unique ID, initial state,
    /// and metadata as specified in US-SESSION-01.
    pub fn new(youtube_url: String, config: ExtractionConfig) -> Self {
        let id = Id::<Session>::new();
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let session = Self {
            id,
            youtube_url,
            config,
            state: SessionState::Created,
            created_at,
            completed_at: None,
            progress: SessionProgress::default(),
            error_message: None,
            unique_slides: 0,
            frames_processed: 0,
            metadata: HashMap::new(),
        };

        log_info(
            &format!("Created new session: {}", session.id),
            "session_creation",
        );

        session
    }

    pub fn start_processing(&mut self) -> DomainResult<()> {
        if !self.state.can_process() {
            return Err(ExtractionError::InvalidConfig(format!(
                "Cannot start processing session in state: {}",
                self.state
            )));
        }

        self.state = SessionState::Processing;
        self.progress.stage = "Processing".to_string();
        log_info(
            &format!("Session {} started processing", self.id),
            "session_state",
        );

        Ok(())
    }

    pub fn complete(&mut self) -> DomainResult<()> {
        if !self.state.can_complete() {
            return Err(ExtractionError::InvalidConfig(format!(
                "Cannot complete session in state: {}",
                self.state
            )));
        }

        self.state = SessionState::Completed;
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
        self.progress.stage = "Completed".to_string();
        self.progress.percentage = 1.0;

        log_info(
            &format!("Session {} completed successfully", self.id),
            "session_state",
        );

        Ok(())
    }

    pub fn fail(&mut self, error_message: String) -> DomainResult<()> {
        if !self.state.can_fail() {
            return Err(ExtractionError::InvalidConfig(format!(
                "Cannot fail session in state: {}",
                self.state
            )));
        }

        self.state = SessionState::Failed;
        self.error_message = Some(error_message.clone());
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );

        log_error_with_context(
            &ExtractionError::InternalError(error_message.clone()),
            &format!("Session {} failed", self.id),
        );

        Ok(())
    }

    pub fn update_progress(
        &mut self,
        stage: impl Into<String>,
        processed: u64,
        total: u64,
        message: Option<String>,
    ) {
        self.progress.stage = stage.into();
        self.progress.total = total;
        self.progress.update(processed, message);
    }

    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.insert(key.into(), value.into());
    }

    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }

    /// Returns the duration of the session in seconds.
    ///
    /// Returns the duration from creation to completion, or the current
    /// duration if the session is still active.
    pub fn duration(&self) -> u64 {
        let end = self.completed_at.unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        });

        end.saturating_sub(self.created_at)
    }

    pub fn span(&self) -> Span {
        let logging_session_id = Id::<LoggingSession>::from_uuid(self.id.as_uuid());
        session_span(Some(logging_session_id), &self.state.to_string(), "session")
    }

    pub fn to_json(&self) -> DomainResult<String> {
        serde_json::to_string(self).map_err(|e| {
            ExtractionError::InternalError(format!("Failed to serialize session: {}", e))
        })
    }

    pub fn from_json(json: &str) -> DomainResult<Self> {
        serde_json::from_str(json).map_err(|e| {
            ExtractionError::InternalError(format!("Failed to deserialize session: {}", e))
        })
    }
}

/// Marker type for session IDs in the session module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Session;

/// Thread-safe session manager for handling multiple sessions.
#[derive(Debug, Clone)]
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<Id<Session>, ProcessingSession>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn create_session(
        &self,
        youtube_url: String,
        config: ExtractionConfig,
    ) -> DomainResult<Id<Session>> {
        let session = ProcessingSession::new(youtube_url, config);
        let id = session.id;

        let mut sessions = self.sessions.write().map_err(|e| {
            ExtractionError::InternalError(format!("Failed to acquire write lock: {}", e))
        })?;

        sessions.insert(id, session);

        Ok(id)
    }

    pub fn get_session(&self, id: Id<Session>) -> DomainResult<ProcessingSession> {
        let sessions = self.sessions.read().map_err(|e| {
            ExtractionError::InternalError(format!("Failed to acquire read lock: {}", e))
        })?;

        sessions.get(&id).cloned().ok_or_else(|| {
            ExtractionError::SessionNotFound(
                Id::<crate::shared::domain::error::Session>::from_uuid(id.as_uuid()),
            )
        })
    }

    pub fn update_session<F>(&self, id: Id<Session>, update_fn: F) -> DomainResult<()>
    where
        F: FnOnce(&mut ProcessingSession) -> DomainResult<()>,
    {
        let mut sessions = self.sessions.write().map_err(|e| {
            ExtractionError::InternalError(format!("Failed to acquire write lock: {}", e))
        })?;

        let session = sessions.get_mut(&id).ok_or_else(|| {
            ExtractionError::SessionNotFound(
                Id::<crate::shared::domain::error::Session>::from_uuid(id.as_uuid()),
            )
        })?;

        update_fn(session)?;

        Ok(())
    }

    pub fn remove_session(&self, id: Id<Session>) -> DomainResult<ProcessingSession> {
        let mut sessions = self.sessions.write().map_err(|e| {
            ExtractionError::InternalError(format!("Failed to acquire write lock: {}", e))
        })?;

        sessions.remove(&id).ok_or_else(|| {
            ExtractionError::SessionNotFound(
                Id::<crate::shared::domain::error::Session>::from_uuid(id.as_uuid()),
            )
        })
    }

    pub fn list_sessions(&self) -> DomainResult<Vec<Id<Session>>> {
        let sessions = self.sessions.read().map_err(|e| {
            ExtractionError::InternalError(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(sessions.keys().copied().collect())
    }

    pub fn persist_session(&self, id: Id<Session>, path: PathBuf) -> DomainResult<()> {
        let session = self.get_session(id)?;
        let json = session.to_json()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|_e| {
                ExtractionError::OutputDirectoryNotFound(format!("{}", parent.display()))
            })?;
        }

        std::fs::write(&path, json).map_err(|e| {
            ExtractionError::InternalError(format!("Failed to write session file: {}", e))
        })?;

        Ok(())
    }

    pub fn recover_session(&self, path: PathBuf) -> DomainResult<ProcessingSession> {
        let json = std::fs::read_to_string(&path).map_err(|e| {
            ExtractionError::SessionRecoveryFailed(format!("Failed to read session file: {}", e))
        })?;

        let session = ProcessingSession::from_json(&json)?;

        // Add the recovered session to the manager
        let id = session.id;
        let mut sessions = self.sessions.write().map_err(|e| {
            ExtractionError::InternalError(format!("Failed to acquire write lock: {}", e))
        })?;

        sessions.insert(id, session.clone());

        Ok(session)
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::domain::config::ExtractionConfig;

    fn create_test_config() -> ExtractionConfig {
        ExtractionConfig::new("https://www.youtube.com/watch?v=test".to_string())
    }

    #[test]
    fn test_session_state_is_terminal() {
        assert!(!SessionState::Created.is_terminal());
        assert!(!SessionState::Processing.is_terminal());
        assert!(SessionState::Completed.is_terminal());
        assert!(SessionState::Failed.is_terminal());
    }

    #[test]
    fn test_session_state_can_start() {
        assert!(SessionState::Created.can_start());
        assert!(!SessionState::Processing.can_start());
        assert!(!SessionState::Completed.can_start());
        assert!(!SessionState::Failed.can_start());
    }

    #[test]
    fn test_session_state_can_process() {
        assert!(SessionState::Created.can_process());
        assert!(!SessionState::Processing.can_process());
        assert!(!SessionState::Completed.can_process());
        assert!(!SessionState::Failed.can_process());
    }

    #[test]
    fn test_session_state_can_complete() {
        assert!(!SessionState::Created.can_complete());
        assert!(SessionState::Processing.can_complete());
        assert!(!SessionState::Completed.can_complete());
        assert!(!SessionState::Failed.can_complete());
    }

    #[test]
    fn test_session_state_can_fail() {
        assert!(SessionState::Created.can_fail());
        assert!(SessionState::Processing.can_fail());
        assert!(!SessionState::Completed.can_fail());
        assert!(!SessionState::Failed.can_fail());
    }

    #[test]
    fn test_session_creation() {
        let config = create_test_config();
        let session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        assert_eq!(session.state, SessionState::Created);
        assert_eq!(session.youtube_url, "https://www.youtube.com/watch?v=test");
        assert!(session.created_at > 0);
        assert!(session.completed_at.is_none());
        assert_eq!(session.unique_slides, 0);
        assert_eq!(session.frames_processed, 0);
    }

    #[test]
    fn test_session_start_processing() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        assert!(session.start_processing().is_ok());
        assert_eq!(session.state, SessionState::Processing);
    }

    #[test]
    fn test_session_start_processing_invalid_state() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        session.state = SessionState::Processing;
        assert!(session.start_processing().is_err());
    }

    #[test]
    fn test_session_complete() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        session.start_processing().unwrap();
        assert!(session.complete().is_ok());
        assert_eq!(session.state, SessionState::Completed);
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn test_session_complete_invalid_state() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        assert!(session.complete().is_err());
    }

    #[test]
    fn test_session_fail() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        assert!(session.fail("Test error".to_string()).is_ok());
        assert_eq!(session.state, SessionState::Failed);
        assert_eq!(session.error_message, Some("Test error".to_string()));
        assert!(session.completed_at.is_some());
    }

    #[test]
    fn test_session_fail_invalid_state() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        session.state = SessionState::Completed;
        assert!(session.fail("Test error".to_string()).is_err());
    }

    #[test]
    fn test_session_update_progress() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        session.update_progress(
            "Downloading",
            50,
            100,
            Some("Downloading video".to_string()),
        );

        assert_eq!(session.progress.stage, "Downloading");
        assert_eq!(session.progress.processed, 50);
        assert_eq!(session.progress.total, 100);
        assert_eq!(session.progress.percentage, 0.5);
        assert_eq!(
            session.progress.message,
            Some("Downloading video".to_string())
        );
    }

    #[test]
    fn test_session_metadata() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        session.set_metadata("video_title", "Test Video");
        session.set_metadata("video_duration", "300");

        assert_eq!(
            session.get_metadata("video_title"),
            Some(&"Test Video".to_string())
        );
        assert_eq!(
            session.get_metadata("video_duration"),
            Some(&"300".to_string())
        );
        assert_eq!(session.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_session_duration() {
        let config = create_test_config();
        let mut session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        // Duration should be captured
        let _duration = session.duration();

        // After completion, duration should be captured
        session.start_processing().unwrap();
        session.complete().unwrap();
        let _completed_duration = session.duration();
    }

    #[test]
    fn test_session_serialization() {
        let config = create_test_config();
        let session =
            ProcessingSession::new("https://www.youtube.com/watch?v=test".to_string(), config);

        let json = session.to_json().unwrap();
        let deserialized = ProcessingSession::from_json(&json).unwrap();

        assert_eq!(session.id, deserialized.id);
        assert_eq!(session.youtube_url, deserialized.youtube_url);
        assert_eq!(session.state, deserialized.state);
    }

    #[test]
    fn test_session_manager_create() {
        let manager = SessionManager::new();
        let config = create_test_config();

        let id = manager
            .create_session("https://www.youtube.com/watch?v=test".to_string(), config)
            .unwrap();

        let session = manager.get_session(id).unwrap();
        assert_eq!(session.youtube_url, "https://www.youtube.com/watch?v=test");
    }

    #[test]
    fn test_session_manager_get_nonexistent() {
        let manager = SessionManager::new();
        let id = Id::<Session>::new();

        let result = manager.get_session(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_session_manager_update() {
        let manager = SessionManager::new();
        let config = create_test_config();

        let id = manager
            .create_session("https://www.youtube.com/watch?v=test".to_string(), config)
            .unwrap();

        manager
            .update_session(id, |session| {
                session.start_processing()?;
                Ok(())
            })
            .unwrap();

        let session = manager.get_session(id).unwrap();
        assert_eq!(session.state, SessionState::Processing);
    }

    #[test]
    fn test_session_manager_remove() {
        let manager = SessionManager::new();
        let config = create_test_config();

        let id = manager
            .create_session("https://www.youtube.com/watch?v=test".to_string(), config)
            .unwrap();

        let removed = manager.remove_session(id).unwrap();
        assert_eq!(removed.youtube_url, "https://www.youtube.com/watch?v=test");

        let result = manager.get_session(id);
        assert!(result.is_err());
    }

    #[test]
    fn test_session_manager_list() {
        let manager = SessionManager::new();
        let config = create_test_config();

        let id1 = manager
            .create_session(
                "https://www.youtube.com/watch?v=test1".to_string(),
                config.clone(),
            )
            .unwrap();
        let id2 = manager
            .create_session("https://www.youtube.com/watch?v=test2".to_string(), config)
            .unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        assert!(sessions.contains(&id1));
        assert!(sessions.contains(&id2));
    }

    #[test]
    fn test_session_progress_increment() {
        let mut progress = SessionProgress::new("Testing", 10);
        assert_eq!(progress.processed, 0);

        progress.increment();
        assert_eq!(progress.processed, 1);
        assert_eq!(progress.percentage, 0.1);

        progress.update(5, Some("Halfway".to_string()));
        assert_eq!(progress.processed, 5);
        assert_eq!(progress.percentage, 0.5);
        assert_eq!(progress.message, Some("Halfway".to_string()));
    }
}
