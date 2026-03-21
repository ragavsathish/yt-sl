//! Video downloader infrastructure using yt-dlp.
//!
//! This module provides video download functionality as specified in US-VIDEO-02:
//! Download Video.

use crate::contexts::video::domain::commands::DownloadVideoCommand;
use crate::contexts::video::domain::state::VideoDownloaded;
use crate::shared::domain::{DomainResult, ExtractionError};
use std::path::Path;
use tokio::process::Command;
use tracing::{info, warn};

const MAX_RETRIES: u8 = 3;
const INITIAL_BACKOFF_SECS: u64 = 2;

/// Video downloader using yt-dlp.
pub struct VideoDownloader;

impl VideoDownloader {
    pub fn new() -> Self {
        Self
    }

    /// Downloads a video from YouTube with retry and exponential backoff.
    pub async fn download_video(
        &self,
        command: DownloadVideoCommand,
        url: &str,
        output_dir: &str,
    ) -> DomainResult<VideoDownloaded> {
        let video_path = format!("{}/{}.mp4", output_dir, command.youtube_video_id);

        if let Some(parent) = Path::new(&video_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ExtractionError::FileSystemError(format!(
                    "Failed to create output directory: {}",
                    e
                ))
            })?;
        }

        if Path::new(&video_path).exists() {
            info!(
                "Video file already exists at {}. Skipping download.",
                video_path
            );
            return Ok(VideoDownloaded {
                video_id: command.video_id,
                path: video_path,
                duration_sec: 0,
                width: 1920,
                height: 1080,
                file_size: 0,
            });
        }

        let mut last_error = String::new();
        for attempt in 0..=MAX_RETRIES {
            if attempt > 0 {
                let backoff = INITIAL_BACKOFF_SECS * 2u64.pow((attempt - 1) as u32);
                warn!(
                    "Download attempt {} failed. Retrying in {}s...",
                    attempt, backoff
                );
                tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                // Clean up partial download
                let _ = std::fs::remove_file(&video_path);
            }

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

            if output.status.success() {
                return Ok(VideoDownloaded {
                    video_id: command.video_id,
                    path: video_path,
                    duration_sec: 0,
                    width: 1920,
                    height: 1080,
                    file_size: 0,
                });
            }

            last_error = String::from_utf8_lossy(&output.stderr).to_string();
        }

        // Clean up partial download on final failure
        let _ = std::fs::remove_file(&video_path);

        Err(ExtractionError::DownloadFailed(
            MAX_RETRIES,
            format!(
                "Download failed after {} retries. Last error: {}",
                MAX_RETRIES, last_error
            ),
        ))
    }
}

impl Default for VideoDownloader {
    fn default() -> Self {
        Self::new()
    }
}
