//! Configuration module for the YouTube Video Slide Extractor.
//!
//! This module provides configuration validation and management functionality
//! as specified in US-ERR-05: Validate Configuration.

use crate::shared::domain::error::{DomainResult, ExtractionError};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Configuration for the slide extraction process.
///
/// This struct contains all configurable parameters for the extraction pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionConfig {
    /// YouTube video URL to process
    pub youtube_url: String,

    /// Frame extraction interval in seconds (default: 5, range: 0.1 - 60)
    #[serde(default = "default_interval")]
    pub interval: f64,

    /// Slide similarity threshold for deduplication (default: 0.85, range: 0.0 - 1.0)
    #[serde(default = "default_threshold")]
    pub threshold: f64,

    /// Output directory for generated files (default: current directory)
    #[serde(default = "default_output_dir")]
    pub output_dir: PathBuf,

    /// OCR languages to use (default: ["eng"])
    #[serde(default = "default_languages")]
    pub languages: Vec<String>,

    /// Whether to include timestamps in output (default: false)
    #[serde(default = "default_timestamps")]
    pub timestamps: bool,

    /// Memory threshold in MB (default: 500)
    #[serde(default = "default_memory_threshold")]
    pub memory_threshold_mb: u64,
}

impl ExtractionConfig {
    /// Creates a new configuration with the given YouTube URL and default values.
    ///
    /// # Arguments
    ///
    /// * `youtube_url` - The YouTube URL to process
    ///
    /// # Returns
    ///
    /// A new `ExtractionConfig` with default values for all optional parameters.
    pub fn new(youtube_url: String) -> Self {
        Self {
            youtube_url,
            interval: default_interval(),
            threshold: default_threshold(),
            output_dir: default_output_dir(),
            languages: default_languages(),
            timestamps: default_timestamps(),
            memory_threshold_mb: default_memory_threshold(),
        }
    }

    /// Validates the configuration parameters.
    ///
    /// This method checks all configuration parameters for validity according to
    /// the acceptance criteria in US-ERR-05.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all parameters are valid
    /// * `Err(ExtractionError)` if any parameter is invalid
    pub fn validate(&self) -> DomainResult<()> {
        let mut errors = Vec::new();

        // Validate interval (must be between 0.1 and 60 seconds)
        if self.interval < 0.1 || self.interval > 60.0 {
            errors.push(format!(
                "Invalid interval: {}. Must be between 0.1 and 60.0 seconds.",
                self.interval
            ));
        }

        // Validate threshold (must be between 0.0 and 1.0)
        if self.threshold < 0.0 || self.threshold > 1.0 {
            errors.push(format!(
                "Invalid threshold: {}. Must be between 0.0 and 1.0.",
                self.threshold
            ));
        }

        // Validate languages
        if self.languages.is_empty() {
            errors.push("At least one OCR language must be specified.".to_string());
        } else {
            let supported_languages = get_supported_languages();
            for lang in &self.languages {
                if !supported_languages.contains(&lang.as_str()) {
                    errors.push(format!(
                        "Unsupported language code: {}. Supported languages: {}",
                        lang,
                        supported_languages.join(", ")
                    ));
                }
            }
        }

        // Validate memory threshold (must be at least 100 MB)
        if self.memory_threshold_mb < 100 {
            errors.push(format!(
                "Invalid memory threshold: {} MB. Must be at least 100 MB.",
                self.memory_threshold_mb
            ));
        }

        // If there are any errors, return them
        if !errors.is_empty() {
            return Err(ExtractionError::InvalidConfig(errors.join("; ")));
        }

        Ok(())
    }

    /// Validates the configuration and returns a new validated configuration.
    ///
    /// This is a convenience method that combines validation with returning self.
    ///
    /// # Returns
    ///
    /// * `Ok(Self)` if validation passes
    /// * `Err(ExtractionError)` if validation fails
    pub fn validated(self) -> DomainResult<Self> {
        self.validate()?;
        Ok(self)
    }

    /// Returns a builder for creating configurations with custom values.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }
}

/// Builder for creating `ExtractionConfig` instances.
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    youtube_url: Option<String>,
    interval: Option<f64>,
    threshold: Option<f64>,
    output_dir: Option<PathBuf>,
    languages: Option<Vec<String>>,
    timestamps: Option<bool>,
    memory_threshold_mb: Option<u64>,
}

impl ConfigBuilder {
    /// Creates a new builder with no values set.
    pub fn new() -> Self {
        Self {
            youtube_url: None,
            interval: None,
            threshold: None,
            output_dir: None,
            languages: None,
            timestamps: None,
            memory_threshold_mb: None,
        }
    }

    /// Sets the YouTube URL.
    pub fn youtube_url(mut self, url: String) -> Self {
        self.youtube_url = Some(url);
        self
    }

    /// Sets the frame extraction interval.
    pub fn interval(mut self, interval: f64) -> Self {
        self.interval = Some(interval);
        self
    }

    /// Sets the similarity threshold.
    pub fn threshold(mut self, threshold: f64) -> Self {
        self.threshold = Some(threshold);
        self
    }

    /// Sets the output directory.
    pub fn output_dir(mut self, dir: PathBuf) -> Self {
        self.output_dir = Some(dir);
        self
    }

    /// Sets the OCR languages.
    pub fn languages(mut self, languages: Vec<String>) -> Self {
        self.languages = Some(languages);
        self
    }

    /// Sets whether to include timestamps.
    pub fn timestamps(mut self, timestamps: bool) -> Self {
        self.timestamps = Some(timestamps);
        self
    }

    /// Sets the memory threshold in MB.
    pub fn memory_threshold_mb(mut self, threshold: u64) -> Self {
        self.memory_threshold_mb = Some(threshold);
        self
    }

    /// Builds the configuration.
    ///
    /// # Returns
    ///
    /// * `Ok(ExtractionConfig)` if all required fields are set and validation passes
    /// * `Err(ExtractionError)` if required fields are missing or validation fails
    pub fn build(self) -> DomainResult<ExtractionConfig> {
        let youtube_url = self
            .youtube_url
            .ok_or_else(|| ExtractionError::InvalidConfig("YouTube URL is required".to_string()))?;

        let config = ExtractionConfig {
            youtube_url,
            interval: self.interval.unwrap_or_else(default_interval),
            threshold: self.threshold.unwrap_or_else(default_threshold),
            output_dir: self.output_dir.unwrap_or_else(default_output_dir),
            languages: self.languages.unwrap_or_else(default_languages),
            timestamps: self.timestamps.unwrap_or_else(default_timestamps),
            memory_threshold_mb: self
                .memory_threshold_mb
                .unwrap_or_else(default_memory_threshold),
        };

        config.validated()
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns the default frame extraction interval.
fn default_interval() -> f64 {
    5.0
}

/// Returns the default similarity threshold.
fn default_threshold() -> f64 {
    0.85
}

/// Returns the default output directory (current directory).
fn default_output_dir() -> PathBuf {
    PathBuf::from(".")
}

/// Returns the default OCR languages.
fn default_languages() -> Vec<String> {
    vec!["eng".to_string()]
}

/// Returns the default value for timestamps.
fn default_timestamps() -> bool {
    false
}

/// Returns the default memory threshold.
fn default_memory_threshold() -> u64 {
    500
}

/// Returns the list of supported OCR language codes.
///
/// Based on Tesseract language codes.
pub fn get_supported_languages() -> Vec<&'static str> {
    vec![
        "eng",     // English
        "spa",     // Spanish
        "fra",     // French
        "deu",     // German
        "jpn",     // Japanese
        "chi_sim", // Chinese Simplified
        "chi_tra", // Chinese Traditional
        "kor",     // Korean
        "rus",     // Russian
        "ara",     // Arabic
        "hin",     // Hindi
        "por",     // Portuguese
        "ita",     // Italian
        "nld",     // Dutch
        "pol",     // Polish
        "tur",     // Turkish
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        assert_eq!(config.interval, 5.0);
        assert_eq!(config.threshold, 0.85);
        assert_eq!(config.languages, vec!["eng"]);
        assert!(!config.timestamps);
    }

    #[test]
    fn test_valid_config() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_interval_too_low() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            interval: 0.05,
            ..config
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_interval_too_high() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            interval: 65.0,
            ..config
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_threshold_too_low() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            threshold: -0.1,
            ..config
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_invalid_threshold_too_high() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            threshold: 1.5,
            ..config
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_empty_languages() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            languages: vec![],
            ..config
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_unsupported_language() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            languages: vec!["xyz".to_string()],
            ..config
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_languages() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            languages: vec!["eng".to_string(), "spa".to_string()],
            ..config
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_memory_threshold() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            memory_threshold_mb: 50,
            ..config
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_builder() {
        let config = ConfigBuilder::new()
            .youtube_url("https://youtube.com/watch?v=test".to_string())
            .interval(10.0)
            .threshold(0.9)
            .timestamps(true)
            .build()
            .unwrap();

        assert_eq!(config.interval, 10.0);
        assert_eq!(config.threshold, 0.9);
        assert!(config.timestamps);
    }

    #[test]
    fn test_builder_missing_url() {
        let result = ConfigBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_invalid_interval() {
        let result = ConfigBuilder::new()
            .youtube_url("https://youtube.com/watch?v=test".to_string())
            .interval(100.0)
            .build();
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_validation_errors() {
        let config = ExtractionConfig::new("https://youtube.com/watch?v=test".to_string());
        let config = ExtractionConfig {
            interval: 0.05,
            threshold: 1.5,
            languages: vec![],
            memory_threshold_mb: 50,
            ..config
        };

        let result = config.validate();
        assert!(result.is_err());

        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("interval"));
        assert!(error_msg.contains("threshold"));
        assert!(error_msg.contains("language"));
        assert!(error_msg.contains("memory"));
    }

    #[test]
    fn test_supported_languages() {
        let languages = get_supported_languages();
        assert!(languages.contains(&"eng"));
        assert!(languages.contains(&"spa"));
        assert!(languages.contains(&"fra"));
        assert!(languages.contains(&"deu"));
        assert!(languages.contains(&"jpn"));
    }
}
