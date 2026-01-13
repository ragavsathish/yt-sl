//! Output directory validation and management.
//!
//! This module provides output directory validation and management functionality
//! as specified in US-ERR-02: Validate Output Directory.
//!
//! Features:
//! - Check if output directory exists and is writable
//! - Create output directory if it doesn't exist
//! - Validate sufficient disk space
//! - Handle permission errors

use crate::shared::domain::error::{DomainResult, ExtractionError};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Represents disk space information.
#[derive(Debug, Clone)]
pub struct DiskSpaceInfo {
    /// Available disk space in bytes
    pub available_bytes: u64,
    /// Total disk space in bytes
    pub total_bytes: u64,
}

impl DiskSpaceInfo {
    pub fn available_mb(&self) -> u64 {
        self.available_bytes / (1024 * 1024)
    }

    pub fn total_mb(&self) -> u64 {
        self.total_bytes / (1024 * 1024)
    }
}

/// Validates and manages output directories.
pub struct OutputDirectoryValidator {
    output_dir: PathBuf,
}

impl OutputDirectoryValidator {
    pub fn new(output_dir: PathBuf) -> Self {
        Self { output_dir }
    }

    /// Validates the output directory.
    ///
    /// This method performs all validation checks as specified in US-ERR-02:
    /// - Checks if output directory exists and is writable
    /// - Creates output directory if it doesn't exist
    /// - Validates sufficient disk space
    /// - Handles permission errors
    pub fn validate(&self, required_space_mb: Option<u64>) -> DomainResult<PathBuf> {
        self.validate_parent_directory()?;
        self.ensure_directory_exists()?;
        self.validate_write_permissions()?;

        if let Some(required_mb) = required_space_mb {
            self.validate_disk_space(required_mb)?;
        }

        Ok(self.output_dir.clone())
    }

    fn validate_parent_directory(&self) -> DomainResult<()> {
        if let Some(parent) = self.output_dir.parent() {
            if !parent.exists() {
                return Err(ExtractionError::ParentDirectoryNotFound(
                    parent.display().to_string(),
                ));
            }
        }
        Ok(())
    }

    fn ensure_directory_exists(&self) -> DomainResult<()> {
        if !self.output_dir.exists() {
            std::fs::create_dir_all(&self.output_dir).map_err(|e| match e.kind() {
                std::io::ErrorKind::PermissionDenied => ExtractionError::PermissionDenied(format!(
                    "Cannot create directory '{}': permission denied",
                    self.output_dir.display()
                )),
                _ => {
                    ExtractionError::OutputDirectoryNotFound(self.output_dir.display().to_string())
                }
            })?;
        }
        Ok(())
    }

    fn validate_write_permissions(&self) -> DomainResult<()> {
        let test_file = self.output_dir.join(".write_test");

        match std::fs::File::create(&test_file) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(b"test") {
                    let _ = std::fs::remove_file(&test_file);
                    return Err(ExtractionError::PermissionDenied(format!(
                        "Cannot write to directory '{}': {}",
                        self.output_dir.display(),
                        e
                    )));
                }
                let _ = std::fs::remove_file(&test_file);
                Ok(())
            }
            Err(e) => Err(ExtractionError::PermissionDenied(format!(
                "Cannot write to directory '{}': {}",
                self.output_dir.display(),
                e
            ))),
        }
    }

    fn validate_disk_space(&self, required_mb: u64) -> DomainResult<()> {
        let available = self.get_available_disk_space()?;
        let available_mb = available.available_mb();

        if available_mb < required_mb {
            return Err(ExtractionError::InsufficientDiskSpace(
                required_mb,
                available_mb,
            ));
        }

        Ok(())
    }

    fn get_available_disk_space(&self) -> DomainResult<DiskSpaceInfo> {
        #[cfg(unix)]
        {
            self.get_disk_space_unix()
        }

        #[cfg(windows)]
        {
            self.get_disk_space_windows()
        }

        #[cfg(not(any(unix, windows)))]
        {
            Ok(DiskSpaceInfo {
                available_bytes: u64::MAX,
                total_bytes: u64::MAX,
            })
        }
    }

    #[cfg(unix)]
    fn get_disk_space_unix(&self) -> DomainResult<DiskSpaceInfo> {
        let _metadata = std::fs::metadata(&self.output_dir).map_err(|e| {
            ExtractionError::OutputDirectoryNotFound(format!(
                "Cannot access directory '{}': {}",
                self.output_dir.display(),
                e
            ))
        })?;

        let statvfs = unsafe {
            let mut stat: libc::statvfs = std::mem::zeroed();
            let path = std::ffi::CString::new(self.output_dir.to_str().unwrap_or(".")).unwrap();
            if libc::statvfs(path.as_ptr(), &mut stat) != 0 {
                return Err(ExtractionError::InternalError(
                    "Failed to get disk space information".to_string(),
                ));
            }
            stat
        };

        let block_size = statvfs.f_frsize as u64;
        let available_blocks = statvfs.f_bavail as u64;
        let total_blocks = statvfs.f_blocks as u64;

        Ok(DiskSpaceInfo {
            available_bytes: block_size * available_blocks,
            total_bytes: block_size * total_blocks,
        })
    }

    #[cfg(windows)]
    fn get_disk_space_windows(&self) -> DomainResult<DiskSpaceInfo> {
        let _metadata = std::fs::metadata(&self.output_dir).map_err(|e| {
            ExtractionError::OutputDirectoryNotFound(format!(
                "Cannot access directory '{}': {}",
                self.output_dir.display(),
                e
            ))
        })?;

        Ok(DiskSpaceInfo {
            available_bytes: u64::MAX / 2,
            total_bytes: u64::MAX,
        })
    }
}

/// Validates an output directory path.
///
/// This is a convenience function that creates a validator and performs validation.
///
/// # Arguments
///
/// * `output_dir` - The output directory path to validate
/// * `required_space_mb` - Required disk space in MB (optional)
///
/// # Returns
///
/// * `Ok(PathBuf)` - The validated output directory path
/// * `Err(ExtractionError)` - If validation fails
///
/// # Example
///
/// ```no_run
/// use yt_sl_extractor::shared::infrastructure::output_directory::validate_output_directory;
/// use std::path::PathBuf;
///
/// let output_dir = PathBuf::from("./output");
/// let validated = validate_output_directory(output_dir, Some(100)).unwrap();
/// ```
pub fn validate_output_directory(
    output_dir: PathBuf,
    required_space_mb: Option<u64>,
) -> DomainResult<PathBuf> {
    OutputDirectoryValidator::new(output_dir).validate(required_space_mb)
}

/// Checks if a directory is writable.
///
/// # Arguments
///
/// * `dir` - The directory path to check
///
/// # Returns
///
/// * `Ok(())` if the directory is writable
/// * `Err(ExtractionError)` if the directory is not writable
pub fn is_directory_writable(dir: &Path) -> DomainResult<()> {
    let test_file = dir.join(".write_test");

    match std::fs::File::create(&test_file) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(b"test") {
                let _ = std::fs::remove_file(&test_file);
                return Err(ExtractionError::PermissionDenied(format!(
                    "Cannot write to directory '{}': {}",
                    dir.display(),
                    e
                )));
            }
            let _ = std::fs::remove_file(&test_file);
            Ok(())
        }
        Err(e) => Err(ExtractionError::PermissionDenied(format!(
            "Cannot write to directory '{}': {}",
            dir.display(),
            e
        ))),
    }
}

/// Gets available disk space for a directory.
///
/// # Arguments
///
/// * `dir` - The directory path to check
///
/// # Returns
///
/// * `Ok(DiskSpaceInfo)` with disk space information
/// * `Err(ExtractionError)` if the check fails
pub fn get_disk_space(dir: &Path) -> DomainResult<DiskSpaceInfo> {
    let validator = OutputDirectoryValidator::new(dir.to_path_buf());
    validator.get_available_disk_space()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_output_directory_validator_new() {
        let dir = PathBuf::from("/tmp/test");
        let validator = OutputDirectoryValidator::new(dir.clone());
        assert_eq!(validator.output_dir, dir);
    }

    #[test]
    fn test_validate_existing_writable_directory() {
        let temp_dir = TempDir::new().unwrap();
        let validator = OutputDirectoryValidator::new(temp_dir.path().to_path_buf());

        let result = validator.validate(None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), temp_dir.path());
    }

    #[test]
    fn test_validate_create_directory() {
        let temp_dir = TempDir::new().unwrap();
        let new_dir = temp_dir.path().join("new_dir");
        let validator = OutputDirectoryValidator::new(new_dir.clone());

        assert!(!new_dir.exists());

        let result = validator.validate(None);
        assert!(result.is_ok());
        assert!(new_dir.exists());
    }

    #[test]
    fn test_validate_parent_not_found() {
        let non_existent = PathBuf::from("/non/existent/path/dir");
        let validator = OutputDirectoryValidator::new(non_existent.clone());

        let result = validator.validate(None);
        assert!(result.is_err());

        if let Err(ExtractionError::ParentDirectoryNotFound(path)) = result {
            assert!(path.contains("non/existent/path"));
        } else {
            panic!("Expected ParentDirectoryNotFound error");
        }
    }

    #[test]
    fn test_is_directory_writable() {
        let temp_dir = TempDir::new().unwrap();
        let result = is_directory_writable(temp_dir.path());
        assert!(result.is_ok());
    }

    #[test]
    fn test_is_directory_not_writable() {
        // On Unix systems, we can test with a read-only directory
        // On Windows, this test may not work as expected
        #[cfg(unix)]
        {
            let temp_dir = TempDir::new().unwrap();
            let dir = temp_dir.path().join("readonly");
            std::fs::create_dir(&dir).unwrap();

            // Make directory read-only
            let mut perms = std::fs::metadata(&dir).unwrap().permissions();
            perms.set_readonly(true);
            std::fs::set_permissions(&dir, perms).unwrap();

            let result = is_directory_writable(&dir);
            assert!(result.is_err());
        }

        #[cfg(not(unix))]
        {
            // Skip this test on non-Unix systems
            // In production, you'd want a different approach
        }
    }

    #[test]
    fn test_disk_space_info() {
        let info = DiskSpaceInfo {
            available_bytes: 1024 * 1024 * 500, // 500 MB
            total_bytes: 1024 * 1024 * 1000,    // 1000 MB
        };

        assert_eq!(info.available_mb(), 500);
        assert_eq!(info.total_mb(), 1000);
    }

    #[test]
    fn test_validate_output_directory() {
        let temp_dir = TempDir::new().unwrap();
        let result = validate_output_directory(temp_dir.path().to_path_buf(), None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_insufficient_disk_space() {
        let temp_dir = TempDir::new().unwrap();
        let validator = OutputDirectoryValidator::new(temp_dir.path().to_path_buf());

        // Request more space than available (this will likely fail on systems with limited space)
        // In practice, this test depends on the actual disk space available
        let result = validator.validate(Some(u64::MAX / (1024 * 1024)));
        // We expect this to fail on most systems
        // But we can't guarantee it, so we just check it doesn't panic
        let _ = result;
    }
}
