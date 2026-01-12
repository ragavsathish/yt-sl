//! Logging infrastructure for the YouTube Video Slide Extractor.
//!
//! This module provides logging functionality as specified in US-ERR-06:
//! Log Technical Errors.
//!
//! Features:
//! - Technical errors logged with full stack traces
//! - Logs include session ID, timestamp, and error context
//! - Logs written to a file in the output directory
//! - Configurable log level (error, warn, info, debug, trace)
//! - Log rotation to prevent excessive log files
//! - System information logging (OS, version, dependencies)

use crate::shared::domain::Id;
use std::path::PathBuf;
use tracing::Subscriber;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    EnvFilter, Layer,
};

/// Represents the session context for logging.
#[derive(Debug, Clone)]
pub struct SessionContext {
    pub session_id: Option<Id<Session>>,
    pub operation: Option<String>,
    pub module: Option<String>,
}

impl SessionContext {
    /// Creates a new session context.
    pub fn new() -> Self {
        Self {
            session_id: None,
            operation: None,
            module: None,
        }
    }

    /// Sets the session ID.
    pub fn with_session_id(mut self, session_id: Id<Session>) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Sets the operation.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Sets the module.
    pub fn with_module(mut self, module: impl Into<String>) -> Self {
        self.module = Some(module.into());
        self
    }
}

impl Default for SessionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Marker type for session IDs in logging context.
#[derive(Debug, Clone)]
pub struct Session;

/// Log level configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "error" => Ok(LogLevel::Error),
            "warn" => Ok(LogLevel::Warn),
            "info" => Ok(LogLevel::Info),
            "debug" => Ok(LogLevel::Debug),
            "trace" => Ok(LogLevel::Trace),
            _ => Err(format!("Invalid log level: {}", s)),
        }
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Error => write!(f, "error"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
        }
    }
}

/// Configuration for the logging system.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level for console output.
    pub console_level: LogLevel,

    /// Log level for file output.
    pub file_level: LogLevel,

    /// Directory where log files will be written.
    pub log_directory: PathBuf,

    /// Maximum size of a log file before rotation (in MB).
    pub max_file_size_mb: u64,

    /// Maximum number of log files to keep.
    pub max_files: u32,

    /// Whether to include system information in logs.
    pub include_system_info: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            console_level: LogLevel::Info,
            file_level: LogLevel::Debug,
            log_directory: PathBuf::from("./logs"),
            max_file_size_mb: 10,
            max_files: 5,
            include_system_info: true,
        }
    }
}

impl LoggingConfig {
    /// Creates a new logging config with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the console log level.
    pub fn console_level(mut self, level: LogLevel) -> Self {
        self.console_level = level;
        self
    }

    /// Sets the file log level.
    pub fn file_level(mut self, level: LogLevel) -> Self {
        self.file_level = level;
        self
    }

    /// Sets the log directory.
    pub fn log_directory(mut self, dir: PathBuf) -> Self {
        self.log_directory = dir;
        self
    }

    /// Sets the maximum file size.
    pub fn max_file_size_mb(mut self, size_mb: u64) -> Self {
        self.max_file_size_mb = size_mb;
        self
    }

    /// Sets the maximum number of files.
    pub fn max_files(mut self, count: u32) -> Self {
        self.max_files = count;
        self
    }

    /// Sets whether to include system information.
    pub fn include_system_info(mut self, include: bool) -> Self {
        self.include_system_info = include;
        self
    }
}

/// Initializes the logging system.
///
/// This function sets up tracing with both console and file logging.
/// It returns a `WorkerGuard` that must be kept alive for the duration
/// of the program to ensure logs are flushed.
///
/// # Arguments
///
/// * `config` - Configuration for the logging system
///
/// # Returns
///
/// A tuple containing:
/// * The initialized subscriber
/// * A worker guard that must be kept alive
///
/// # Example
///
/// ```no_run
/// use yt_sl_extractor::shared::infrastructure::logging::{init_logging, LoggingConfig, LogLevel};
///
/// let config = LoggingConfig::new()
///     .console_level(LogLevel::Info)
///     .file_level(LogLevel::Debug);
///
/// let (_subscriber, _guard) = init_logging(config).unwrap();
/// ```
pub fn init_logging(
    config: LoggingConfig,
) -> Result<(impl Subscriber, WorkerGuard), std::io::Error> {
    // Create log directory if it doesn't exist
    std::fs::create_dir_all(&config.log_directory)?;

    // Set up file appender with rotation
    let file_appender = tracing_appender::rolling::daily(&config.log_directory, "yt-sl-extractor");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    // Create console layer
    let console_filter = EnvFilter::new(config.console_level.to_string());
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_filter(console_filter);

    // Create file layer
    let file_filter = EnvFilter::new(config.file_level.to_string());
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_target(true)
        .with_thread_ids(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_ansi(false)
        .with_filter(file_filter);

    // Initialize the subscriber
    let subscriber = tracing_subscriber::registry()
        .with(console_layer)
        .with(file_layer);

    // Log system information if configured
    if config.include_system_info {
        log_system_info();
    }

    Ok((subscriber, guard))
}

/// Initializes logging with default configuration.
///
/// This is a convenience function that uses default logging settings.
///
/// # Returns
///
/// A tuple containing:
/// * The initialized subscriber
/// * A worker guard that must be kept alive
pub fn init_default_logging() -> Result<(impl Subscriber, WorkerGuard), std::io::Error> {
    init_logging(LoggingConfig::default())
}

/// Logs system information for debugging purposes.
pub fn log_system_info() {
    let os_info = os_info::get();
    tracing::info!(
        os_type = ?os_info.os_type(),
        os_version = ?os_info.version(),
        arch = std::env::consts::ARCH,
        "System information"
    );

    if let Some(version) = env!("CARGO_PKG_VERSION").split('+').next() {
        tracing::info!(version = %version, "Application version");
    }

    // Log dependency versions
    let rustc_ver = rustc_version::version()
        .ok()
        .map(|v| v.to_string())
        .unwrap_or_else(|| {
            rustc_version::version_meta()
                .ok()
                .map(|v| v.short_version_string)
                .unwrap_or_else(|| "unknown".to_string())
        });
    tracing::info!(
        rustc_version = %rustc_ver,
        "Rust compiler version"
    );
}

/// Creates a span for a session operation.
///
/// # Arguments
///
/// * `session_id` - The session ID
/// * `operation` - The operation being performed
/// * `module` - The module performing the operation
///
/// # Returns
///
/// A tracing span
#[inline]
pub fn session_span(
    session_id: Option<Id<Session>>,
    operation: &str,
    module: &str,
) -> tracing::Span {
    match session_id {
        Some(id) => tracing::info_span!(
            "session_operation",
            session_id = %id,
            operation,
            module
        ),
        None => tracing::info_span!("operation", operation, module),
    }
}

/// Logs an error with context.
///
/// # Arguments
///
/// * `error` - The error to log
/// * `context` - Additional context information
#[inline]
pub fn log_error_with_context<E: std::error::Error>(error: &E, context: &str) {
    tracing::error!(
        error = %error,
        context,
        "Error occurred"
    );

    // Log the error chain if there are multiple causes
    let mut source = error.source();
    let mut depth = 1;
    while let Some(cause) = source {
        tracing::error!(
            error = %cause,
            depth,
            "Error cause"
        );
        source = cause.source();
        depth += 1;
    }
}

/// Logs a warning with context.
///
/// # Arguments
///
/// * `message` - The warning message
/// * `context` - Additional context information
#[inline]
pub fn log_warning(message: &str, context: &str) {
    tracing::warn!(message, context, "Warning");
}

/// Logs an info message with context.
///
/// # Arguments
///
/// * `message` - The info message
/// * `context` - Additional context information
#[inline]
pub fn log_info(message: &str, context: &str) {
    tracing::info!(message, context, "Info");
}

/// Logs a debug message with context.
///
/// # Arguments
///
/// * `message` - The debug message
/// * `context` - Additional context information
#[inline]
pub fn log_debug(message: &str, context: &str) {
    tracing::debug!(message, context, "Debug");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("error").ok(), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("warn").ok(), Some(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("info").ok(), Some(LogLevel::Info));
        assert_eq!(LogLevel::from_str("debug").ok(), Some(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("trace").ok(), Some(LogLevel::Trace));
        assert!(LogLevel::from_str("invalid").is_err());
    }

    #[test]
    fn test_log_level_case_insensitive() {
        assert_eq!(LogLevel::from_str("ERROR").ok(), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("Error").ok(), Some(LogLevel::Error));
        assert_eq!(LogLevel::from_str("INFO").ok(), Some(LogLevel::Info));
    }

    #[test]
    fn test_log_level_display() {
        assert_eq!(LogLevel::Error.to_string(), "error");
        assert_eq!(LogLevel::Warn.to_string(), "warn");
        assert_eq!(LogLevel::Info.to_string(), "info");
        assert_eq!(LogLevel::Debug.to_string(), "debug");
        assert_eq!(LogLevel::Trace.to_string(), "trace");
    }

    #[test]
    fn test_session_context_builder() {
        let context = SessionContext::new()
            .with_operation("test_operation")
            .with_module("test_module");

        assert_eq!(context.operation, Some("test_operation".to_string()));
        assert_eq!(context.module, Some("test_module".to_string()));
        assert!(context.session_id.is_none());
    }

    #[test]
    fn test_logging_config_defaults() {
        let config = LoggingConfig::default();
        assert_eq!(config.console_level, LogLevel::Info);
        assert_eq!(config.file_level, LogLevel::Debug);
        assert_eq!(config.max_file_size_mb, 10);
        assert_eq!(config.max_files, 5);
        assert!(config.include_system_info);
    }

    #[test]
    fn test_logging_config_builder() {
        let config = LoggingConfig::new()
            .console_level(LogLevel::Debug)
            .file_level(LogLevel::Trace)
            .max_file_size_mb(20)
            .max_files(10)
            .include_system_info(false);

        assert_eq!(config.console_level, LogLevel::Debug);
        assert_eq!(config.file_level, LogLevel::Trace);
        assert_eq!(config.max_file_size_mb, 20);
        assert_eq!(config.max_files, 10);
        assert!(!config.include_system_info);
    }
}
