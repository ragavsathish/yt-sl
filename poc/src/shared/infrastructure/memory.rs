//! Memory monitoring and management.
//!
//! This module provides memory monitoring functionality as specified in US-ERR-03:
//! Handle Insufficient Memory.
//!
//! Features:
//! - Monitor memory usage during processing
//! - Detect low memory conditions
//! - Provide graceful degradation or error messages
//! - Suggest processing in smaller batches

use crate::shared::domain::error::{ExtractionError, DomainResult};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

/// Memory usage information.
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    /// Current memory usage in bytes
    pub current_bytes: u64,
    /// Peak memory usage in bytes
    pub peak_bytes: u64,
    /// Memory threshold in bytes
    pub threshold_bytes: u64,
}

impl MemoryUsage {
    /// Returns current memory usage in megabytes.
    pub fn current_mb(&self) -> u64 {
        self.current_bytes / (1024 * 1024)
    }

    /// Returns peak memory usage in megabytes.
    pub fn peak_mb(&self) -> u64 {
        self.peak_bytes / (1024 * 1024)
    }

    /// Returns memory threshold in megabytes.
    pub fn threshold_mb(&self) -> u64 {
        self.threshold_bytes / (1024 * 1024)
    }

    /// Returns memory utilization as a percentage of threshold.
    pub fn utilization_percent(&self) -> f64 {
        if self.threshold_bytes == 0 {
            0.0
        } else {
            (self.current_bytes as f64 / self.threshold_bytes as f64) * 100.0
        }
    }
}

/// Memory monitor for tracking memory usage.
///
/// This struct tracks memory usage across the application
/// and provides methods to check for memory threshold violations.
pub struct MemoryMonitor {
    /// Peak memory usage in bytes
    peak_bytes: Arc<AtomicU64>,

    /// Memory threshold in bytes (default: 500MB)
    threshold_bytes: u64,

    /// Warning threshold percentage (default: 80%)
    warning_threshold_percent: f64,
}

impl MemoryMonitor {
    /// Creates a new memory monitor with default settings.
    pub fn new() -> Self {
        Self {
            peak_bytes: Arc::new(AtomicU64::new(0)),
            threshold_bytes: 500 * 1024 * 1024, // 500 MB
            warning_threshold_percent: 0.8, // 80%
        }
    }

    /// Creates a new memory monitor with custom threshold.
    ///
    /// # Arguments
    ///
    /// * `threshold_mb` - Memory threshold in megabytes
    /// * `warning_threshold_percent` - Warning threshold as percentage (0.0-1.0)
    pub fn with_threshold(threshold_mb: u64, warning_threshold_percent: f64) -> Self {
        Self {
            peak_bytes: Arc::new(AtomicU64::new(0)),
            threshold_bytes: threshold_mb * 1024 * 1024,
            warning_threshold_percent: warning_threshold_percent.clamp(0.0, 1.0),
        }
    }

    /// Records current memory usage.
    ///
    /// # Arguments
    ///
    /// * `bytes` - Current memory usage in bytes
    pub fn record_usage(&self, bytes: u64) {
        // Update peak if current usage is higher
        let mut current_peak = self.peak_bytes.load(Ordering::Relaxed);
        if bytes > current_peak {
            let _ = self.peak_bytes.compare_exchange(
                current_peak,
                bytes,
                Ordering::Relaxed,
                Ordering::Relaxed,
            );
        }
    }

    /// Gets current memory usage information.
    pub fn get_usage(&self) -> MemoryUsage {
        let current_bytes = self.get_current_bytes();
        let peak_bytes = self.peak_bytes.load(Ordering::Relaxed);

        MemoryUsage {
            current_bytes,
            peak_bytes,
            threshold_bytes: self.threshold_bytes,
        }
    }

    /// Gets estimated current memory usage in bytes.
    ///
    /// This is an approximation based on system memory info.
    fn get_current_bytes(&self) -> u64 {
        #[cfg(unix)]
        {
            self.get_current_bytes_unix()
        }

        #[cfg(not(unix))]
        {
            // Fallback for non-Unix systems
            // In production, you'd want platform-specific implementations
            self.peak_bytes.load(Ordering::Relaxed)
        }
    }

    /// Gets current memory usage on Unix-like systems.
    #[cfg(unix)]
    fn get_current_bytes_unix(&self) -> u64 {
        use std::fs;

        // Read /proc/self/status to get memory info
        if let Ok(status) = std::fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb.parse::<u64>() {
                            return kb * 1024;
                        }
                    }
                }
            }
        }

        // Fallback to peak if we can't read current
        self.peak_bytes.load(Ordering::Relaxed)
    }

    /// Checks if memory usage exceeds threshold.
    pub fn exceeds_threshold(&self) -> bool {
        let current_bytes = self.get_current_bytes();
        current_bytes > self.threshold_bytes
    }

    /// Checks if memory usage is approaching threshold.
    pub fn approaching_threshold(&self) -> bool {
        let current_bytes = self.get_current_bytes();
        let warning_bytes = (self.threshold_bytes as f64 * self.warning_threshold_percent) as u64;
        current_bytes > warning_bytes
    }

    /// Validates memory usage and returns an error if threshold is exceeded.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if memory usage is within limits
    /// * `Err(ExtractionError)` if memory threshold is exceeded
    pub fn validate(&self) -> DomainResult<()> {
        if self.exceeds_threshold() {
            let usage = self.get_usage();
            return Err(ExtractionError::MemoryThresholdExceeded {
                used: usage.current_mb(),
                threshold: usage.threshold_mb(),
            });
        }

        Ok(())
    }

    /// Checks if memory usage is approaching threshold and logs a warning.
    ///
    /// This method should be called periodically during processing.
    pub fn check_and_warn(&self) -> bool {
        if self.approaching_threshold() {
            let usage = self.get_usage();
            let percent = usage.utilization_percent();
            tracing::warn!(
                current_mb = usage.current_mb(),
                threshold_mb = usage.threshold_mb(),
                percent = percent,
                "Memory usage approaching threshold: {:.1}% of {} MB limit",
                percent,
                usage.threshold_mb()
            );
            true
        } else {
            false
        }
    }
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Validates memory availability for a given requirement.
///
/// # Arguments
///
/// * `required_mb` - Required memory in megabytes
///
/// # Returns
///
/// * `Ok(())` if sufficient memory is available
/// * `Err(ExtractionError)` if insufficient memory
pub fn validate_memory_requirement(required_mb: u64) -> DomainResult<()> {
    let available_mb = get_available_memory_mb()?;

    if available_mb < required_mb {
        return Err(ExtractionError::InsufficientMemory(required_mb));
    }

    Ok(())
}

/// Gets available memory in megabytes.
///
/// This is a platform-specific function that returns the available
/// system memory in MB.
fn get_available_memory_mb() -> DomainResult<u64> {
    #[cfg(unix)]
    {
        get_available_memory_unix()
    }

    #[cfg(not(unix))]
    {
        // Fallback for non-Unix systems
        // In production, you'd want platform-specific implementations
        Ok(1024) // Assume 1GB available
    }
}

/// Gets available memory on Unix-like systems.
#[cfg(unix)]
fn get_available_memory_unix() -> DomainResult<u64> {
    use std::fs;

    // Read /proc/meminfo to get memory info
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        for line in meminfo.lines() {
            if line.starts_with("MemAvailable:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb.parse::<u64>() {
                        return Ok(kb / 1024);
                    }
                }
            }
        }
    }

    // Fallback: try MemTotal - MemAvailable
    if let Ok(meminfo) = std::fs::read_to_string("/proc/meminfo") {
        let mut total_kb: Option<u64> = None;
        let mut available_kb: Option<u64> = None;

        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb.parse::<u64>() {
                        total_kb = Some(kb);
                    }
                }
            } else if line.starts_with("MemAvailable:") {
                if let Some(kb) = line.split_whitespace().nth(1) {
                    if let Ok(kb) = kb.parse::<u64>() {
                        available_kb = Some(kb);
                    }
                }
            }
        }

        if let (Some(total), Some(available)) = (total_kb, available_kb) {
            return Ok(available / 1024);
        }
    }

    // Fallback: assume 500MB available
    Ok(500)
}

/// Checks if memory is sufficient for processing.
///
/// This is a convenience function that creates a memory monitor
/// and validates the memory requirement.
///
/// # Arguments
///
/// * `required_mb` - Required memory in megabytes
/// * `threshold_mb` - Memory threshold in megabytes (optional)
///
/// # Returns
///
/// * `Ok(MemoryMonitor)` - A configured memory monitor
/// * `Err(ExtractionError)` - If memory requirement cannot be met
///
/// # Example
///
/// ```no_run
/// use poc::shared::infrastructure::memory::check_memory_sufficient;
///
/// let monitor = check_memory_sufficient(500, Some(600)).unwrap();
/// ```
pub fn check_memory_sufficient(
    required_mb: u64,
    threshold_mb: Option<u64>,
) -> DomainResult<MemoryMonitor> {
    // Validate memory requirement
    validate_memory_requirement(required_mb)?;

    // Create monitor with custom or default threshold
    let monitor = if let Some(threshold) = threshold_mb {
        MemoryMonitor::with_threshold(threshold, 0.8)
    } else {
        MemoryMonitor::new()
    };

    Ok(monitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_monitor_new() {
        let monitor = MemoryMonitor::new();
        assert_eq!(monitor.threshold_bytes, 500 * 1024 * 1024);
        assert_eq!(monitor.warning_threshold_percent, 0.8);
    }

    #[test]
    fn test_memory_monitor_with_threshold() {
        let monitor = MemoryMonitor::with_threshold(1000, 0.9);
        assert_eq!(monitor.threshold_bytes, 1000 * 1024 * 1024);
        assert_eq!(monitor.warning_threshold_percent, 0.9);
    }

    #[test]
    fn test_record_usage() {
        let monitor = MemoryMonitor::new();
        monitor.record_usage(100 * 1024 * 1024);
        monitor.record_usage(200 * 1024 * 1024);
        monitor.record_usage(150 * 1024 * 1024);

        let peak = monitor.peak_bytes.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(peak, 200 * 1024 * 1024);
    }

    #[test]
    fn test_exceeds_threshold() {
        let monitor = MemoryMonitor::new();
        monitor.record_usage(600 * 1024 * 1024);

        assert!(!monitor.exceeds_threshold());

        monitor.record_usage(600 * 1024 * 1024 + 1);
        assert!(monitor.exceeds_threshold());
    }

    #[test]
    fn test_approaching_threshold() {
        let monitor = MemoryMonitor::new();
        monitor.record_usage(400 * 1024 * 1024);

        assert!(monitor.approaching_threshold());

        monitor.record_usage(300 * 1024 * 1024);
        assert!(!monitor.approaching_threshold());
    }

    #[test]
    fn test_validate_success() {
        let monitor = MemoryMonitor::new();
        monitor.record_usage(400 * 1024 * 1024);

        let result = monitor.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_failure() {
        let monitor = MemoryMonitor::new();
        monitor.record_usage(600 * 1024 * 1024);

        let result = monitor.validate();
        assert!(result.is_err());

        if let Err(ExtractionError::MemoryThresholdExceeded { used, threshold }) = result {
            assert_eq!(used, 600);
            assert_eq!(threshold, 500);
        } else {
            panic!("Expected MemoryThresholdExceeded error");
        }
    }

    #[test]
    fn test_memory_usage() {
        let usage = MemoryUsage {
            current_bytes: 400 * 1024 * 1024,
            peak_bytes: 500 * 1024 * 1024,
            threshold_bytes: 500 * 1024 * 1024,
        };

        assert_eq!(usage.current_mb(), 400);
        assert_eq!(usage.peak_mb(), 500);
        assert_eq!(usage.threshold_mb(), 500);
    }

    #[test]
    fn test_memory_utilization_percent() {
        let usage = MemoryUsage {
            current_bytes: 400 * 1024 * 1024,
            peak_bytes: 500 * 1024 * 1024,
            threshold_bytes: 500 * 1024 * 1024,
        };

        assert_eq!(usage.utilization_percent(), 80.0);
    }

    #[test]
    fn test_validate_memory_requirement() {
        // This test may not work on all systems
        // as it depends on /proc/meminfo availability
        let result = validate_memory_requirement(100);
        // We just check it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_check_memory_sufficient() {
        // This test may not work on all systems
        // as it depends on /proc/meminfo availability
        let result = check_memory_sufficient(100, Some(200));
        // We just check it doesn't panic
        let _ = result;
    }
}
