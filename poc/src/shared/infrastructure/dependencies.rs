//! External dependency checking and management.
//!
//! This module provides functionality for checking and validating external dependencies
//! as specified in US-ERR-04: Handle External Dependency Failures.
//!
//! Features:
//! - Detect failures in external dependencies (yt-dlp, ffmpeg, tesseract)
//! - Provide clear error messages indicating which dependency failed
//! - Suggest remediation steps (install missing dependency, check version)
//! - Gracefully handle missing or unavailable dependencies

use crate::shared::domain::error::{DomainResult, ExtractionError};
use std::collections::HashMap;
use std::process::Command;

/// Represents an external dependency that the application requires.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Dependency {
    /// yt-dlp - YouTube video downloader
    YtDlp,
    /// FFmpeg - Video processing tool
    FFmpeg,
    /// Tesseract OCR - Text recognition
    Tesseract,
}

impl Dependency {
    pub fn command_name(&self) -> &str {
        match self {
            Dependency::YtDlp => "yt-dlp",
            Dependency::FFmpeg => "ffmpeg",
            Dependency::Tesseract => "tesseract",
        }
    }

    pub fn display_name(&self) -> &str {
        match self {
            Dependency::YtDlp => "yt-dlp",
            Dependency::FFmpeg => "FFmpeg",
            Dependency::Tesseract => "Tesseract OCR",
        }
    }

    pub fn minimum_version(&self) -> Option<&str> {
        match self {
            Dependency::YtDlp => Some("2023.01.01"),
            Dependency::FFmpeg => Some("4.0"),
            Dependency::Tesseract => Some("4.0"),
        }
    }

    pub fn installation_instructions(&self) -> &str {
        match self {
            Dependency::YtDlp => {
                "Install yt-dlp using pip: pip install yt-dlp\n\
                 Or download from: https://github.com/yt-dlp/yt-dlp/releases"
            }
            Dependency::FFmpeg => {
                "Install FFmpeg using your package manager:\n\
                 - macOS: brew install ffmpeg\n\
                 - Ubuntu/Debian: sudo apt install ffmpeg\n\
                 - Windows: Download from https://ffmpeg.org/download.html"
            }
            Dependency::Tesseract => {
                "Install Tesseract OCR using your package manager:\n\
                 - macOS: brew install tesseract\n\
                 - Ubuntu/Debian: sudo apt install tesseract-ocr\n\
                 - Windows: Download from https://github.com/UB-Mannheim/tesseract/wiki"
            }
        }
    }

    pub fn troubleshooting_steps(&self) -> Vec<&str> {
        match self {
            Dependency::YtDlp => vec![
                "Ensure yt-dlp is in your system PATH",
                "Try running 'yt-dlp --version' to verify installation",
                "Update yt-dlp: pip install --upgrade yt-dlp",
                "Check if a firewall or proxy is blocking network access",
            ],
            Dependency::FFmpeg => vec![
                "Ensure FFmpeg is in your system PATH",
                "Try running 'ffmpeg -version' to verify installation",
                "Reinstall FFmpeg if the version is too old",
                "Check if required codecs are installed",
            ],
            Dependency::Tesseract => vec![
                "Ensure Tesseract is in your system PATH",
                "Try running 'tesseract --version' to verify installation",
                "Install required language data (e.g., tesseract-ocr-eng)",
                "Check TESSDATA_PREFIX environment variable",
            ],
        }
    }
}

impl std::fmt::Display for Dependency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Result of checking a single dependency.
#[derive(Debug, Clone)]
pub struct DependencyCheckResult {
    pub dependency: Dependency,
    pub available: bool,
    pub version: Option<String>,
    pub version_ok: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

impl DependencyCheckResult {
    /// Returns true if the dependency is available and meets all requirements.
    pub fn is_ok(&self) -> bool {
        self.available && self.version_ok
    }

    /// Returns a user-friendly message about the dependency status.
    pub fn status_message(&self) -> String {
        if !self.available {
            format!(
                "{} is not installed or not found in PATH. {}",
                self.dependency.display_name(),
                self.dependency.installation_instructions()
            )
        } else if !self.version_ok {
            format!(
                "{} version {} is installed, but version {} or higher is required. {}",
                self.dependency.display_name(),
                self.version.as_deref().unwrap_or("unknown"),
                self.dependency.minimum_version().unwrap_or("unknown"),
                self.dependency.installation_instructions()
            )
        } else {
            format!(
                "{} is available (version {}).",
                self.dependency.display_name(),
                self.version.as_deref().unwrap_or("unknown")
            )
        }
    }
}

/// Checker for external dependencies.
pub struct DependencyChecker {
    /// Custom paths to search for dependencies
    custom_paths: HashMap<Dependency, String>,
}

impl DependencyChecker {
    pub fn new() -> Self {
        Self {
            custom_paths: HashMap::new(),
        }
    }

    pub fn with_custom_path(mut self, dependency: Dependency, path: String) -> Self {
        self.custom_paths.insert(dependency, path);
        self
    }

    pub fn check(&self, dependency: &Dependency) -> DependencyCheckResult {
        let command = self.get_command_path(dependency);

        let version_result = self.get_version(dependency, &command);

        match version_result {
            Ok(version) => {
                let version_ok = self.check_version_requirement(dependency, &version);
                DependencyCheckResult {
                    dependency: dependency.clone(),
                    available: true,
                    version: Some(version.clone()),
                    version_ok,
                    path: Some(command),
                    error: None,
                }
            }
            Err(e) => {
                let exists = self.command_exists(&command);
                DependencyCheckResult {
                    dependency: dependency.clone(),
                    available: exists,
                    version: None,
                    version_ok: false,
                    path: if exists { Some(command) } else { None },
                    error: Some(e.to_string()),
                }
            }
        }
    }

    pub fn check_all(&self) -> Vec<DependencyCheckResult> {
        vec![
            self.check(&Dependency::YtDlp),
            self.check(&Dependency::FFmpeg),
            self.check(&Dependency::Tesseract),
        ]
    }

    /// Checks all dependencies and returns an error if any are missing or invalid.
    pub fn validate_all(&self) -> DomainResult<()> {
        let results = self.check_all();
        let failed: Vec<_> = results.iter().filter(|r| !r.is_ok()).collect();

        if failed.is_empty() {
            Ok(())
        } else {
            let error_messages: Vec<String> = failed.iter().map(|r| r.status_message()).collect();

            Err(ExtractionError::ExternalDependencyUnavailable(
                error_messages.join("\n\n"),
            ))
        }
    }

    fn get_command_path(&self, dependency: &Dependency) -> String {
        self.custom_paths
            .get(dependency)
            .cloned()
            .unwrap_or_else(|| dependency.command_name().to_string())
    }

    fn command_exists(&self, command: &str) -> bool {
        Command::new(command).arg("--version").output().is_ok()
            || Command::new(command).arg("-version").output().is_ok()
    }

    fn get_version(&self, dependency: &Dependency, command: &str) -> Result<String, String> {
        // Try -version first for FFmpeg, --version for others
        let primary_flag = if matches!(dependency, Dependency::FFmpeg) {
            "-version"
        } else {
            "--version"
        };
        let secondary_flag = if matches!(dependency, Dependency::FFmpeg) {
            "--version"
        } else {
            "-version"
        };

        let output = Command::new(command)
            .arg(primary_flag)
            .output()
            .map_err(|e| format!("Failed to execute {}: {}", command, e))?;

        if !output.status.success() {
            let output = Command::new(command)
                .arg(secondary_flag)
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", command, e))?;

            if !output.status.success() {
                return Err(format!("{} returned non-zero exit code", command));
            }

            return self.parse_version(dependency, &output.stdout);
        }

        self.parse_version(dependency, &output.stdout)
    }

    fn parse_version(&self, dependency: &Dependency, output: &[u8]) -> Result<String, String> {
        let stdout = String::from_utf8_lossy(output);

        match dependency {
            Dependency::YtDlp => {
                if let Some(line) = stdout.lines().next() {
                    if let Some(version) = line.split_whitespace().nth(1) {
                        Ok(version.to_string())
                    } else {
                        Ok(line.to_string())
                    }
                } else {
                    Err("No version output found".to_string())
                }
            }
            Dependency::FFmpeg => {
                for line in stdout.lines() {
                    if line.contains("version") {
                        if let Some(version) = line.split("version ").nth(1) {
                            if let Some(version) = version.split_whitespace().next() {
                                return Ok(version.to_string());
                            }
                        }
                    }
                }
                Err("No version output found".to_string())
            }
            Dependency::Tesseract => {
                if let Some(line) = stdout.lines().next() {
                    if let Some(version) = line.split_whitespace().nth(1) {
                        Ok(version.to_string())
                    } else {
                        Ok(line.to_string())
                    }
                } else {
                    Err("No version output found".to_string())
                }
            }
        }
    }

    fn check_version_requirement(&self, dependency: &Dependency, version: &str) -> bool {
        if let Some(min_version) = dependency.minimum_version() {
            self.compare_versions(version, min_version) >= 0
        } else {
            true
        }
    }

    /// Compares two version strings.
    fn compare_versions(&self, v1: &str, v2: &str) -> i32 {
        let v1_parts: Vec<u32> = v1
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse().ok())
            .collect();
        let v2_parts: Vec<u32> = v2
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|s| s.parse().ok())
            .collect();

        let max_len = std::cmp::max(v1_parts.len(), v2_parts.len());

        for i in 0..max_len {
            let p1 = v1_parts.get(i).copied().unwrap_or(0);
            let p2 = v2_parts.get(i).copied().unwrap_or(0);

            if p1 != p2 {
                return (p1 as i32) - (p2 as i32);
            }
        }

        0
    }
}

impl Default for DependencyChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Checks if all required dependencies are available.
///
/// This is a convenience function that creates a default checker
/// and validates all dependencies.
///
/// # Returns
///
/// * `Ok(())` if all dependencies are available and meet requirements
/// * `Err(ExtractionError)` if any dependency is missing or invalid
pub fn check_dependencies() -> DomainResult<()> {
    DependencyChecker::new().validate_all()
}

/// Gets detailed information about all dependencies.
///
/// # Returns
///
/// A vector of `DependencyCheckResult` for all dependencies
pub fn get_dependency_info() -> Vec<DependencyCheckResult> {
    DependencyChecker::new().check_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_command_name() {
        assert_eq!(Dependency::YtDlp.command_name(), "yt-dlp");
        assert_eq!(Dependency::FFmpeg.command_name(), "ffmpeg");
        assert_eq!(Dependency::Tesseract.command_name(), "tesseract");
    }

    #[test]
    fn test_dependency_display_name() {
        assert_eq!(Dependency::YtDlp.display_name(), "yt-dlp");
        assert_eq!(Dependency::FFmpeg.display_name(), "FFmpeg");
        assert_eq!(Dependency::Tesseract.display_name(), "Tesseract OCR");
    }

    #[test]
    fn test_dependency_minimum_version() {
        assert!(Dependency::YtDlp.minimum_version().is_some());
        assert!(Dependency::FFmpeg.minimum_version().is_some());
        assert!(Dependency::Tesseract.minimum_version().is_some());
    }

    #[test]
    fn test_dependency_installation_instructions() {
        assert!(!Dependency::YtDlp.installation_instructions().is_empty());
        assert!(!Dependency::FFmpeg.installation_instructions().is_empty());
        assert!(!Dependency::Tesseract.installation_instructions().is_empty());
    }

    #[test]
    fn test_dependency_troubleshooting_steps() {
        assert!(!Dependency::YtDlp.troubleshooting_steps().is_empty());
        assert!(!Dependency::FFmpeg.troubleshooting_steps().is_empty());
        assert!(!Dependency::Tesseract.troubleshooting_steps().is_empty());
    }

    #[test]
    fn test_dependency_display() {
        assert_eq!(Dependency::YtDlp.to_string(), "yt-dlp");
        assert_eq!(Dependency::FFmpeg.to_string(), "FFmpeg");
        assert_eq!(Dependency::Tesseract.to_string(), "Tesseract OCR");
    }

    #[test]
    fn test_dependency_checker_default() {
        let checker = DependencyChecker::default();
        assert!(checker.custom_paths.is_empty());
    }

    #[test]
    fn test_dependency_checker_with_custom_path() {
        let checker = DependencyChecker::new()
            .with_custom_path(Dependency::FFmpeg, "/custom/path/ffmpeg".to_string());

        assert_eq!(
            checker.custom_paths.get(&Dependency::FFmpeg),
            Some(&"/custom/path/ffmpeg".to_string())
        );
    }

    #[test]
    fn test_compare_versions() {
        let checker = DependencyChecker::new();
        assert!(checker.compare_versions("5.0.0", "4.0.0") > 0);
        assert!(checker.compare_versions("4.0.0", "5.0.0") < 0);
        assert!(checker.compare_versions("4.0.0", "4.0.0") == 0);
        assert!(checker.compare_versions("4.1.0", "4.0.0") > 0);
        assert!(checker.compare_versions("4.0.1", "4.0.0") > 0);
        assert!(checker.compare_versions("4.0", "4.0.0") == 0);
        assert!(checker.compare_versions("4.0.0", "4.0") == 0);
        assert!(checker.compare_versions("4", "4.0.0") == 0);
        assert!(checker.compare_versions("4.1", "4.0.0") > 0);
    }

    #[test]
    fn test_dependency_check_result_is_ok() {
        let result = DependencyCheckResult {
            dependency: Dependency::FFmpeg,
            available: true,
            version: Some("5.0.0".to_string()),
            version_ok: true,
            path: Some("/usr/bin/ffmpeg".to_string()),
            error: None,
        };
        assert!(result.is_ok());
    }

    #[test]
    fn test_dependency_check_result_not_ok_not_available() {
        let result = DependencyCheckResult {
            dependency: Dependency::FFmpeg,
            available: false,
            version: None,
            version_ok: false,
            path: None,
            error: Some("Command not found".to_string()),
        };
        assert!(!result.is_ok());
    }

    #[test]
    fn test_dependency_check_result_not_ok_version_mismatch() {
        let result = DependencyCheckResult {
            dependency: Dependency::FFmpeg,
            available: true,
            version: Some("3.0.0".to_string()),
            version_ok: false,
            path: Some("/usr/bin/ffmpeg".to_string()),
            error: None,
        };
        assert!(!result.is_ok());
    }

    #[test]
    fn test_dependency_check_result_status_message() {
        let result = DependencyCheckResult {
            dependency: Dependency::FFmpeg,
            available: true,
            version: Some("5.0.0".to_string()),
            version_ok: true,
            path: Some("/usr/bin/ffmpeg".to_string()),
            error: None,
        };
        let msg = result.status_message();
        assert!(msg.contains("FFmpeg"));
        assert!(msg.contains("available"));
    }
}
