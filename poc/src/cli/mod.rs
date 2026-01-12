//! CLI module for the YouTube Video Slide Extractor.
//!
//! This module provides command line argument parsing and validation
//! as specified in US-CLI-01: Parse Command Line Arguments and
//! US-CLI-04: Validate Input Configuration.

use crate::shared::domain::config::{get_supported_languages, ExtractionConfig};
use crate::shared::domain::error::{DomainResult, ExtractionError};
use crate::shared::infrastructure::output_directory::validate_output_directory;
use clap::Parser;
use std::path::PathBuf;

pub mod progress;

pub use progress::*;

/// Command line arguments for the YouTube Video Slide Extractor.
///
/// This struct defines all command line arguments that can be passed
/// to the application, as specified in US-CLI-01.
#[derive(Parser, Debug, Clone)]
#[command(
    name = "yt-sl-extractor",
    author = "YouTube Slide Extractor Team",
    version = "0.1.0",
    about = "Extract unique slides from YouTube videos with OCR and Markdown output",
    long_about = "A CLI tool that extracts unique slides from YouTube videos, \
                  performs OCR text recognition, and generates a Markdown document \
                  with embedded slide images and extracted text."
)]
pub struct CliArgs {
    /// YouTube video URL to process
    #[arg(
        short = 'u',
        long = "url",
        value_name = "URL",
        help = "The YouTube video URL to process"
    )]
    pub youtube_url: String,

    /// Frame extraction interval in seconds
    #[arg(
        short = 'i',
        long = "interval",
        value_name = "SECONDS",
        default_value = "5.0",
        help = "Frame extraction interval in seconds (default: 5.0, range: 0.1 - 60.0)"
    )]
    pub interval: f64,

    /// Slide similarity threshold for deduplication
    #[arg(
        short = 't',
        long = "threshold",
        value_name = "THRESHOLD",
        default_value = "0.85",
        help = "Slide similarity threshold for deduplication (default: 0.85, range: 0.0 - 1.0)"
    )]
    pub threshold: f64,

    /// Output directory for generated files
    #[arg(
        short = 'o',
        long = "output",
        value_name = "DIR",
        default_value = ".",
        help = "Output directory for generated files (default: current directory)"
    )]
    pub output_dir: PathBuf,

    /// OCR languages to use
    #[arg(
        short = 'l',
        long = "languages",
        value_name = "LANGS",
        value_delimiter = ',',
        default_value = "eng",
        help = "OCR languages to use, comma-separated (default: eng). \
                Supported: eng, spa, fra, deu, jpn, chi_sim, chi_tra, kor, rus, ara, hin, por, ita, nld, pol, tur"
    )]
    pub languages: Vec<String>,

    /// Include timestamps in output
    #[arg(
        short = 's',
        long = "timestamps",
        help = "Include timestamps in the output"
    )]
    pub timestamps: bool,

    /// Memory threshold in MB
    #[arg(
        short = 'm',
        long = "memory-threshold",
        value_name = "MB",
        default_value = "500",
        help = "Memory threshold in MB (default: 500, minimum: 100)"
    )]
    pub memory_threshold_mb: u64,
}

impl CliArgs {
    /// Creates a new CliArgs instance from the command line arguments.
    ///
    /// This is a convenience method that uses `Parser::parse()` to parse
    /// the command line arguments.
    ///
    /// # Returns
    ///
    /// A new `CliArgs` instance parsed from the command line.
    pub fn from_args() -> Self {
        Self::parse()
    }

    /// Creates a new CliArgs instance from the provided arguments.
    ///
    /// This is useful for testing and programmatic use.
    ///
    /// # Arguments
    ///
    /// * `args` - A slice of string slices representing the command line arguments
    ///
    /// # Returns
    ///
    /// A new `CliArgs` instance parsed from the provided arguments.
    pub fn try_parse_from<I, T>(args: I) -> Result<Self, clap::Error>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Parser::try_parse_from(args)
    }

    /// Validates the command line arguments.
    ///
    /// This method validates all command line arguments according to the
    /// acceptance criteria in US-CLI-04.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if all arguments are valid
    /// * `Err(ExtractionError)` if any argument is invalid
    pub fn validate(&self) -> DomainResult<()> {
        // Validate interval range (must be between 0.1 and 60 seconds)
        if self.interval < 0.1 || self.interval > 60.0 {
            return Err(ExtractionError::InvalidConfig(format!(
                "Invalid interval: {}. Must be between 0.1 and 60.0 seconds.",
                self.interval
            )));
        }

        // Validate threshold range (must be between 0.0 and 1.0)
        if self.threshold < 0.0 || self.threshold > 1.0 {
            return Err(ExtractionError::InvalidConfig(format!(
                "Invalid threshold: {}. Must be between 0.0 and 1.0.",
                self.threshold
            )));
        }

        // Validate languages
        if self.languages.is_empty() {
            return Err(ExtractionError::InvalidConfig(
                "At least one OCR language must be specified.".to_string(),
            ));
        }

        let supported_languages = get_supported_languages();
        for lang in &self.languages {
            if !supported_languages.contains(&lang.as_str()) {
                return Err(ExtractionError::InvalidConfig(format!(
                    "Unsupported language code: {}. Supported languages: {}",
                    lang,
                    supported_languages.join(", ")
                )));
            }
        }

        // Validate memory threshold (must be at least 100 MB)
        if self.memory_threshold_mb < 100 {
            return Err(ExtractionError::InvalidConfig(format!(
                "Invalid memory threshold: {} MB. Must be at least 100 MB.",
                self.memory_threshold_mb
            )));
        }

        // Validate output directory
        validate_output_directory(self.output_dir.clone(), Some(self.memory_threshold_mb))?;

        Ok(())
    }

    /// Converts the CLI arguments to an ExtractionConfig.
    ///
    /// This method creates an `ExtractionConfig` from the validated CLI arguments.
    ///
    /// # Returns
    ///
    /// * `Ok(ExtractionConfig)` if conversion succeeds
    /// * `Err(ExtractionError)` if validation fails
    pub fn to_config(&self) -> DomainResult<ExtractionConfig> {
        self.validate()?;

        let config = ExtractionConfig {
            youtube_url: self.youtube_url.clone(),
            interval: self.interval,
            threshold: self.threshold,
            output_dir: self.output_dir.clone(),
            languages: self.languages.clone(),
            timestamps: self.timestamps,
            memory_threshold_mb: self.memory_threshold_mb,
        };

        // Validate the config using the existing validation logic
        config.validated()
    }
}

impl Default for CliArgs {
    fn default() -> Self {
        Self {
            youtube_url: String::new(),
            interval: 5.0,
            threshold: 0.85,
            output_dir: PathBuf::from("."),
            languages: vec!["eng".to_string()],
            timestamps: false,
            memory_threshold_mb: 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_temp_dir() -> PathBuf {
        let temp_dir = TempDir::new().unwrap();
        temp_dir.path().to_path_buf()
    }

    #[test]
    fn test_cli_args_default() {
        let args = CliArgs::default();
        assert_eq!(args.interval, 5.0);
        assert_eq!(args.threshold, 0.85);
        assert_eq!(args.languages, vec!["eng"]);
        assert!(!args.timestamps);
        assert_eq!(args.memory_threshold_mb, 500);
    }

    #[test]
    fn test_cli_args_from_valid_arguments() {
        let temp_dir = create_temp_dir();
        let args = CliArgs::try_parse_from(vec![
            "yt-sl-extractor",
            "--url",
            "https://www.youtube.com/watch?v=test",
            "--interval",
            "10.0",
            "--threshold",
            "0.9",
            "--languages",
            "eng,spa",
            "--timestamps",
            "--output",
            temp_dir.to_str().unwrap(),
        ])
        .unwrap();

        assert_eq!(args.youtube_url, "https://www.youtube.com/watch?v=test");
        assert_eq!(args.interval, 10.0);
        assert_eq!(args.threshold, 0.9);
        assert_eq!(args.languages, vec!["eng", "spa"]);
        assert!(args.timestamps);
    }

    #[test]
    fn test_validate_valid_interval() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            interval: 5.0,
            output_dir: temp_dir.clone(),
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_interval_too_low() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            interval: 0.05,
            output_dir: temp_dir.clone(),
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_interval_too_high() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            interval: 65.0,
            output_dir: temp_dir.clone(),
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_valid_threshold() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            threshold: 0.85,
            output_dir: temp_dir.clone(),
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_threshold_too_low() {
        let args = CliArgs {
            threshold: -0.1,
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_threshold_too_high() {
        let args = CliArgs {
            threshold: 1.5,
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_empty_languages() {
        let args = CliArgs {
            languages: vec![],
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_unsupported_language() {
        let args = CliArgs {
            languages: vec!["xyz".to_string()],
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_valid_languages() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            languages: vec!["eng".to_string(), "spa".to_string()],
            output_dir: temp_dir.clone(),
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_validate_memory_threshold_too_low() {
        let args = CliArgs {
            memory_threshold_mb: 50,
            ..Default::default()
        };
        assert!(args.validate().is_err());
    }

    #[test]
    fn test_validate_valid_memory_threshold() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            memory_threshold_mb: 500,
            output_dir: temp_dir.clone(),
            ..Default::default()
        };
        assert!(args.validate().is_ok());
    }

    #[test]
    fn test_to_config() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            youtube_url: "https://www.youtube.com/watch?v=test".to_string(),
            interval: 10.0,
            threshold: 0.9,
            languages: vec!["eng".to_string()],
            timestamps: true,
            memory_threshold_mb: 600,
            output_dir: temp_dir.clone(),
        };

        let config = args.to_config().unwrap();
        assert_eq!(config.youtube_url, "https://www.youtube.com/watch?v=test");
        assert_eq!(config.interval, 10.0);
        assert_eq!(config.threshold, 0.9);
        assert!(config.timestamps);
        assert_eq!(config.memory_threshold_mb, 600);
    }

    #[test]
    fn test_to_config_with_invalid_interval() {
        let temp_dir = create_temp_dir();
        let args = CliArgs {
            youtube_url: "https://www.youtube.com/watch?v=test".to_string(),
            interval: 0.05,
            output_dir: temp_dir.clone(),
            ..Default::default()
        };

        assert!(args.to_config().is_err());
    }

    #[test]
    fn test_cli_args_with_short_options() {
        let args = CliArgs::try_parse_from(vec![
            "yt-sl-extractor",
            "-u",
            "https://www.youtube.com/watch?v=test",
            "-i",
            "10.0",
            "-t",
            "0.9",
            "-l",
            "eng,spa",
            "-s",
        ])
        .unwrap();

        assert_eq!(args.youtube_url, "https://www.youtube.com/watch?v=test");
        assert_eq!(args.interval, 10.0);
        assert_eq!(args.threshold, 0.9);
        assert_eq!(args.languages, vec!["eng", "spa"]);
        assert!(args.timestamps);
    }

    #[test]
    fn test_cli_args_missing_required_url() {
        let result = CliArgs::try_parse_from(vec!["yt-sl-extractor", "--interval", "10.0"]);
        assert!(result.is_err());
    }
}
