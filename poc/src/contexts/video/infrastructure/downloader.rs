//! Video downloader infrastructure using yt-dlp.
//!
//! This module provides video download functionality as specified in US-VIDEO-02:
//! Download Video.

use crate::contexts::video::domain::commands::DownloadVideoCommand;
use crate::contexts::video::domain::state::VideoDownloaded;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::path::Path;
use tokio::process::Command;

/// Video downloader using yt-dlp.
pub struct VideoDownloader;

impl VideoDownloader {
    /// Creates a new video downloader.
    pub fn new() -> Self {
        Self
    }

    /// Downloads a video from YouTube.
    ///
    /// # Arguments
    ///
    /// * `command` - The download command
    /// * `url` - The YouTube URL
    /// * `output_dir` - Directory to save the video
    ///
    /// # Returns
    ///
    /// A VideoDownloaded event
    pub async fn download_video(
        &self,
        command: DownloadVideoCommand,
        url: &str,
        output_dir: &str,
    ) -> DomainResult<VideoDownloaded> {
        let video_path = format!("{}/{}.mp4", output_dir, command.video_id);

        // Ensure output directory exists
        if let Some(parent) = Path::new(&video_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ExtractionError::FileSystemError(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        // Execute yt-dlp
        // -f mp4: preferred format
        // -o: output path
        let output = Command::new("yt-dlp")
            .args([
                "-f",
                "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best",
                "-o",
                &video_path,
                url,
            ])
            .output()
            .await
            .map_err(|e| {
                ExtractionError::ExternalDependencyUnavailable(format!(
                    "yt-dlp execution failed: {}",
                    e
                ))
            })?;

        if !output.status.success() {
            return Err(ExtractionError::DownloadFailed(
                0,
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // In a real app, we'd probe for resolution/size, but for now we'll use defaults or mock
        Ok(VideoDownloaded {
            video_id: command.video_id,
            path: video_path,
            duration_sec: 0, // Will be updated by caller
            width: 1920,
            height: 1080,
            file_size: 0,
        })
    }
}

impl Default for VideoDownloader {
    fn default() -> Self {
        Self::new()
    }
}
