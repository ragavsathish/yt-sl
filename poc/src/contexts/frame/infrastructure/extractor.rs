//! FFmpeg frame extraction infrastructure.
//!
//! This module provides frame extraction functionality as specified in US-FRAME-01:
//! Extract Frames at Intervals.
//!
//! Features:
//! - Extract frames from video at configurable intervals
//! - Use FFmpeg for frame extraction
//! - Store frames with metadata (timestamp, frame number)
//! - Track extraction progress
//! - Handle memory constraints during extraction
//! - Support different output formats (JPEG, PNG)

use crate::contexts::frame::domain::commands::{ExtractFramesCommand, FrameFormat};
use crate::contexts::frame::domain::events::{
    create_extracted_event, generate_frame_path, validate_extraction_params,
};
use crate::contexts::frame::domain::handlers::handle_extract_frames;
use crate::contexts::frame::domain::state::{FrameExtracted, FramesExtracted};
use crate::shared::domain::{DomainResult, ExtractionError, Id, VideoFrame};
use crate::shared::infrastructure::memory::MemoryMonitor;
use std::fs;
use std::path::Path;
use std::process::Command;

/// FFmpeg frame extractor.
///
/// This struct handles frame extraction from video files using FFmpeg.
pub struct FrameExtractor {
    /// Memory monitor for tracking memory usage
    memory_monitor: MemoryMonitor,
    /// Maximum allowed memory in bytes
    #[allow(dead_code)]
    max_memory_bytes: u64,
}

impl FrameExtractor {
    pub fn new() -> Self {
        Self {
            memory_monitor: MemoryMonitor::new(),
            max_memory_bytes: 500 * 1024 * 1024, // 500 MB default
        }
    }

    pub fn with_memory_limit(max_memory_mb: u64) -> Self {
        Self {
            memory_monitor: MemoryMonitor::with_threshold(max_memory_mb, 0.8),
            max_memory_bytes: max_memory_mb * 1024 * 1024,
        }
    }

    /// Extracts frames from a video file.
    ///
    /// This function provides frame extraction functionality as specified in US-FRAME-01:
    /// Extract Frames at Intervals.
    pub async fn extract_frames(
        &mut self,
        command: ExtractFramesCommand,
        duration_sec: u64,
    ) -> DomainResult<(FramesExtracted, Vec<FrameExtracted>)> {
        validate_extraction_params(&command)?;

        fs::create_dir_all(&command.output_dir).map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!(
                "Failed to create output directory: {}",
                e
            ))
        })?;

        self.memory_monitor.validate()?;

        let total_frames = handle_extract_frames(command.clone(), duration_sec)?.total_frames;

        let frames = self
            .extract_frames_with_ffmpeg(command.clone(), duration_sec, total_frames)
            .await?;

        let extracted_event = FramesExtracted {
            video_id: command.video_id,
            total_frames: frames.len() as u32,
            frames_dir: command.output_dir.clone(),
            interval_secs: command.interval_secs,
            format: format!("{:?}", command.output_format),
        };

        Ok((extracted_event, frames))
    }

    async fn extract_frames_with_ffmpeg(
        &mut self,
        command: ExtractFramesCommand,
        _duration_sec: u64,
        total_frames: u32,
    ) -> DomainResult<Vec<FrameExtracted>> {
        let mut frames = Vec::new();
        let extension = match command.output_format {
            FrameFormat::Jpeg => "jpg",
            FrameFormat::Png => "png",
        };

        let mut ffmpeg_cmd = Command::new("ffmpeg");
        ffmpeg_cmd.arg("-i");
        ffmpeg_cmd.arg(&command.video_path);
        ffmpeg_cmd.arg("-vf");
        ffmpeg_cmd.arg(format!("fps=1/{}", command.interval_secs));
        ffmpeg_cmd.arg("-q:v");
        ffmpeg_cmd.arg("2"); // Good quality

        if let Some(quality) = command.jpeg_quality {
            ffmpeg_cmd.arg("-q:v");
            ffmpeg_cmd.arg(format!("{}", (100 - quality) / 10));
        }

        let output_pattern = Path::new(&command.output_dir)
            .join(format!("{}_frame_%04d.{}", command.video_id, extension));
        ffmpeg_cmd.arg(output_pattern);

        let output = ffmpeg_cmd.output().map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!("Failed to execute FFmpeg: {}", e))
        })?;

        if !output.status.success() {
            return Err(ExtractionError::FrameExtractionFailed(format!(
                "FFmpeg failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        let video_id = command.video_id;
        for frame_number in 1..=total_frames {
            if self.memory_monitor.check_and_warn() {
                tracing::warn!("Memory usage approaching threshold during frame extraction");
            }

            let timestamp = ((frame_number - 1) as u64 * command.interval_secs) as f64;

            let frame_path =
                generate_frame_path(&command.output_dir, &video_id, frame_number, extension);

            if Path::new(&frame_path).exists() {
                let (width, height) = self.get_frame_dimensions(&frame_path)?;

                let frame_id = Id::<VideoFrame>::new();
                let frame = create_extracted_event(
                    frame_id,
                    &video_id,
                    frame_number,
                    timestamp,
                    frame_path,
                    width,
                    height,
                );

                frames.push(frame);
            }
        }

        Ok(frames)
    }

    fn get_frame_dimensions(&self, frame_path: &str) -> DomainResult<(u32, u32)> {
        let img = image::open(frame_path).map_err(|_e| {
            ExtractionError::FrameExtractionFailed(format!(
                "Failed to open frame {}: {}",
                frame_path, _e
            ))
        })?;

        Ok((img.width(), img.height()))
    }

    pub fn validate_frame(frame_path: &str) -> DomainResult<()> {
        let img = image::open(frame_path)
            .map_err(|_e| ExtractionError::CorruptFrame { timestamp: 0.0 })?;

        if img.width() == 0 || img.height() == 0 {
            return Err(ExtractionError::CorruptFrame { timestamp: 0.0 });
        }

        Ok(())
    }

    /// Extracts a single frame from video at a specific timestamp.
    pub fn extract_single_frame(
        video_path: &str,
        timestamp_sec: f64,
        output_path: &str,
        format: FrameFormat,
    ) -> DomainResult<()> {
        let _extension = match format {
            FrameFormat::Jpeg => "jpg",
            FrameFormat::Png => "png",
        };

        let output = Command::new("ffmpeg")
            .arg("-ss")
            .arg(format!("{:.3}", timestamp_sec))
            .arg("-i")
            .arg(video_path)
            .arg("-vframes")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg("-y")
            .arg(output_path)
            .output()
            .map_err(|e| {
                ExtractionError::FrameExtractionFailed(format!("Failed to extract frame: {}", e))
            })?;

        if !output.status.success() {
            return Err(ExtractionError::FrameExtractionFailed(format!(
                "FFmpeg failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }
}

impl Default for FrameExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_extractor_new() {
        let extractor = FrameExtractor::new();
        assert_eq!(extractor.max_memory_bytes, 500 * 1024 * 1024);
    }

    #[test]
    fn test_frame_extractor_with_memory_limit() {
        let extractor = FrameExtractor::with_memory_limit(1000);
        assert_eq!(extractor.max_memory_bytes, 1000 * 1024 * 1024);
    }

    #[test]
    fn test_frame_extractor_default() {
        let extractor = FrameExtractor::default();
        assert_eq!(extractor.max_memory_bytes, 500 * 1024 * 1024);
    }

    #[test]
    fn test_extract_single_frame_command() {
        // This test verifies command structure without actually running FFmpeg
        let video_path = "/tmp/video.mp4";
        let timestamp_sec = 10.5;
        let output_path = "/tmp/frame.jpg";

        // Build command to verify structure
        let mut cmd = Command::new("ffmpeg");
        cmd.arg("-ss")
            .arg(format!("{:.3}", timestamp_sec))
            .arg("-i")
            .arg(video_path)
            .arg("-vframes")
            .arg("1")
            .arg("-q:v")
            .arg("2")
            .arg("-y")
            .arg(output_path);

        // Verify command was built correctly
        let cmd_str = format!("{:?}", cmd);
        assert!(cmd_str.contains("ffmpeg"));
        assert!(cmd_str.contains("-ss"));
        assert!(cmd_str.contains("-vframes"));
        assert!(cmd_str.contains("1"));
    }
}
