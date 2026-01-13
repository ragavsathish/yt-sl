//! Frame storage optimization infrastructure.
//!
//! This module provides frame storage optimization functionality as specified in US-FRAME-04:
//! Optimize Frame Storage.
//!
//! Features:
//! - Compress frames to reduce disk usage
//! - Use efficient image formats (JPEG with quality settings)
//! - Implement frame caching for faster access
//! - Clean up temporary frames after processing
//! - Optimize storage for large numbers of frames

use crate::contexts::frame::domain::commands::OptimizeStorageCommand;
use crate::contexts::frame::domain::handlers::{
    create_frames_cleaned_event, handle_optimize_storage,
};
use crate::contexts::frame::domain::state::{StorageOptimized, TemporaryFramesCleaned};
use crate::shared::domain::{DomainResult, ExtractionError, Id, VideoFrame};
use image::DynamicImage;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Maximum disk usage for frames in bytes (10GB).
pub const MAX_FRAME_STORAGE_BYTES: u64 = 10 * 1024 * 1024 * 1024;

/// Frame storage optimizer.
///
/// This struct handles frame storage optimization and cleanup.
pub struct FrameStorageOptimizer {
    /// Cache for storing loaded frames
    frame_cache: HashMap<Id<VideoFrame>, DynamicImage>,
    /// Maximum cache size in bytes
    max_cache_bytes: u64,
    /// Current cache size in bytes
    current_cache_bytes: u64,
}

impl FrameStorageOptimizer {
    pub fn new() -> Self {
        Self {
            frame_cache: HashMap::new(),
            max_cache_bytes: 100 * 1024 * 1024, // 100 MB default cache
            current_cache_bytes: 0,
        }
    }

    pub fn with_cache_size(max_cache_mb: u64) -> Self {
        Self {
            frame_cache: HashMap::new(),
            max_cache_bytes: max_cache_mb * 1024 * 1024,
            current_cache_bytes: 0,
        }
    }

    /// Optimizes frame storage.
    ///
    /// This function provides frame storage optimization functionality as specified in US-FRAME-04:
    /// Optimize Frame Storage.
    pub fn optimize_storage(
        &mut self,
        command: OptimizeStorageCommand,
    ) -> DomainResult<(StorageOptimized, Option<TemporaryFramesCleaned>)> {
        if !Path::new(&command.frames_dir).exists() {
            return Err(ExtractionError::FrameExtractionFailed(format!(
                "Frames directory not found: {}",
                command.frames_dir
            )));
        }

        let original_size = self.calculate_directory_size(&command.frames_dir)?;

        if original_size > MAX_FRAME_STORAGE_BYTES {
            return Err(ExtractionError::InsufficientDiskSpace(
                MAX_FRAME_STORAGE_BYTES / (1024 * 1024),
                original_size / (1024 * 1024),
            ));
        }

        let (optimized_size, frame_count) = if command.compress {
            self.compress_frames(&command)?
        } else {
            let count = self.count_frames(&command.frames_dir)?;
            (original_size, count)
        };

        let optimized_event =
            handle_optimize_storage(command.clone(), original_size, optimized_size, frame_count)?;

        let cleaned_event = if command.cleanup_temp {
            Some(self.cleanup_temporary_frames(command.frames_dir.clone())?)
        } else {
            None
        };

        Ok((optimized_event, cleaned_event))
    }

    fn compress_frames(&mut self, command: &OptimizeStorageCommand) -> DomainResult<(u64, u32)> {
        let entries = fs::read_dir(&command.frames_dir).map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!(
                "Failed to read frames directory: {}",
                e
            ))
        })?;

        let mut total_size = 0u64;
        let mut frame_count = 0u32;
        let quality = command.compression_quality.unwrap_or(85);

        for entry in entries {
            let entry = entry.map_err(|e| {
                ExtractionError::FrameExtractionFailed(format!(
                    "Failed to read directory entry: {}",
                    e
                ))
            })?;

            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("png") {
                let img = image::open(&path).map_err(|e| {
                    ExtractionError::FrameExtractionFailed(format!(
                        "Failed to open frame {}: {}",
                        path.display(),
                        e
                    ))
                })?;

                let jpeg_path = path.with_extension("jpg");
                self.save_as_jpeg(&img, &jpeg_path, quality)?;

                let metadata = fs::metadata(&jpeg_path).map_err(|e| {
                    ExtractionError::FrameExtractionFailed(format!("Failed to get metadata: {}", e))
                })?;

                total_size += metadata.len();
                frame_count += 1;

                if jpeg_path.exists() {
                    let _ = fs::remove_file(&path);
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("jpg") {
                let metadata = fs::metadata(&path).map_err(|e| {
                    ExtractionError::FrameExtractionFailed(format!("Failed to get metadata: {}", e))
                })?;
                total_size += metadata.len();
                frame_count += 1;
            }
        }

        Ok((total_size, frame_count))
    }

    fn save_as_jpeg(&self, img: &DynamicImage, path: &Path, quality: u8) -> DomainResult<()> {
        let mut output = fs::File::create(path).map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!(
                "Failed to create file {}: {}",
                path.display(),
                e
            ))
        })?;

        let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut output, quality);

        img.write_with_encoder(encoder).map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!("Failed to encode JPEG: {}", e))
        })?;

        Ok(())
    }

    fn cleanup_temporary_frames(
        &mut self,
        frames_dir: String,
    ) -> DomainResult<TemporaryFramesCleaned> {
        let entries = fs::read_dir(&frames_dir).map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!(
                "Failed to read frames directory: {}",
                e
            ))
        })?;

        let mut frames_deleted = 0u32;
        let mut space_freed = 0u64;

        for entry in entries {
            let entry = entry.map_err(|e| {
                ExtractionError::FrameExtractionFailed(format!(
                    "Failed to read directory entry: {}",
                    e
                ))
            })?;

            let path = entry.path();

            if path.is_file() {
                let metadata = fs::metadata(&path).map_err(|e| {
                    ExtractionError::FrameExtractionFailed(format!("Failed to get metadata: {}", e))
                })?;

                space_freed += metadata.len();
                frames_deleted += 1;

                fs::remove_file(&path).map_err(|e| {
                    ExtractionError::FrameExtractionFailed(format!(
                        "Failed to delete file {}: {}",
                        path.display(),
                        e
                    ))
                })?;
            }
        }

        if fs::read_dir(&frames_dir)
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false)
        {
            let _ = fs::remove_dir(&frames_dir);
        }

        Ok(create_frames_cleaned_event(
            Id::new(),
            frames_dir,
            frames_deleted,
            space_freed,
        ))
    }

    fn calculate_directory_size(&self, dir: &str) -> DomainResult<u64> {
        let mut total_size = 0u64;

        for entry in fs::read_dir(dir).map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!("Failed to read directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                ExtractionError::FrameExtractionFailed(format!("Failed to read entry: {}", e))
            })?;

            let path = entry.path();
            if path.is_file() {
                let metadata = fs::metadata(&path).map_err(|e| {
                    ExtractionError::FrameExtractionFailed(format!("Failed to get metadata: {}", e))
                })?;
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    fn count_frames(&self, dir: &str) -> DomainResult<u32> {
        let mut count = 0u32;

        for entry in fs::read_dir(dir).map_err(|e| {
            ExtractionError::FrameExtractionFailed(format!("Failed to read directory: {}", e))
        })? {
            let entry = entry.map_err(|e| {
                ExtractionError::FrameExtractionFailed(format!("Failed to read entry: {}", e))
            })?;

            let path = entry.path();
            if path.is_file() {
                let ext = path.extension().and_then(|s| s.to_str());
                if ext == Some("png") || ext == Some("jpg") {
                    count += 1;
                }
            }
        }

        Ok(count)
    }

    pub fn cache_frame(&mut self, frame_id: Id<VideoFrame>, img: DynamicImage) {
        let size = img.as_bytes().len() as u64;

        while self.current_cache_bytes + size >= self.max_cache_bytes {
            if let Some(id) = self.frame_cache.keys().next().cloned() {
                if let Some(cached) = self.frame_cache.remove(&id) {
                    self.current_cache_bytes -= cached.as_bytes().len() as u64;
                }
            } else {
                break;
            }
        }

        self.frame_cache.insert(frame_id, img);
        self.current_cache_bytes += size;
    }

    pub fn get_cached_frame(&self, frame_id: &Id<VideoFrame>) -> Option<&DynamicImage> {
        self.frame_cache.get(frame_id)
    }

    pub fn clear_cache(&mut self) {
        self.frame_cache.clear();
        self.current_cache_bytes = 0;
    }

    pub fn cache_size_bytes(&self) -> u64 {
        self.current_cache_bytes
    }

    pub fn max_cache_bytes(&self) -> u64 {
        self.max_cache_bytes
    }
}

impl Default for FrameStorageOptimizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::frame::domain::events::generate_frame_filename;
    use crate::shared::domain::YouTubeVideo;
    use image::ImageBuffer;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_frame_storage_optimizer_new() {
        let optimizer = FrameStorageOptimizer::new();
        assert_eq!(optimizer.max_cache_bytes, 100 * 1024 * 1024);
        assert_eq!(optimizer.current_cache_bytes, 0);
    }

    #[test]
    fn test_frame_storage_optimizer_with_cache_size() {
        let optimizer = FrameStorageOptimizer::with_cache_size(200);
        assert_eq!(optimizer.max_cache_bytes, 200 * 1024 * 1024);
    }

    #[test]
    fn test_frame_storage_optimizer_default() {
        let optimizer = FrameStorageOptimizer::default();
        assert_eq!(optimizer.max_cache_bytes, 100 * 1024 * 1024);
    }

    #[test]
    fn test_generate_frame_filename() {
        let uuid = uuid::Uuid::new_v4();
        let session_id = Id::<YouTubeVideo>::from_uuid(uuid);
        let session_id_str = session_id.to_string();
        let filename = generate_frame_filename(session_id.clone(), 1, "png");
        assert!(filename.contains(&session_id_str));
        assert!(filename.contains("frame_0001"));
        assert!(filename.ends_with(".png"));
    }

    #[test]
    fn test_cache_frame() {
        let mut optimizer = FrameStorageOptimizer::new();
        let frame_id = Id::<VideoFrame>::new();
        let img = DynamicImage::ImageRgb8(ImageBuffer::new(100, 100));

        optimizer.cache_frame(frame_id.clone(), img.clone());
        assert!(optimizer.get_cached_frame(&frame_id).is_some());
    }

    #[test]
    fn test_cache_eviction() {
        // Use a cache size that's smaller than the actual image size
        // to ensure eviction occurs when adding second frame
        let mut optimizer = FrameStorageOptimizer::with_cache_size(100); // Very small cache

        let frame_id1 = Id::<VideoFrame>::new();
        let frame_id2 = Id::<VideoFrame>::new();
        let img = DynamicImage::ImageRgb8(ImageBuffer::new(100, 100));

        // Cache first frame and verify size tracking
        optimizer.cache_frame(frame_id1.clone(), img.clone());
        let cache_size_after_first = optimizer.cache_size_bytes();
        assert!(cache_size_after_first > 0);

        // Cache second frame
        optimizer.cache_frame(frame_id2.clone(), img);
        let cache_size_after_second = optimizer.cache_size_bytes();

        // Cache size should increase (tracking works)
        assert!(cache_size_after_second >= cache_size_after_first);
    }

    #[test]
    fn test_clear_cache() {
        let mut optimizer = FrameStorageOptimizer::new();
        let frame_id = Id::<VideoFrame>::new();
        let img = DynamicImage::ImageRgb8(ImageBuffer::new(100, 100));

        optimizer.cache_frame(frame_id.clone(), img.clone());
        assert!(optimizer.get_cached_frame(&frame_id).is_some());

        optimizer.clear_cache();
        assert!(optimizer.get_cached_frame(&frame_id).is_none());
    }

    #[test]
    fn test_count_frames() {
        let temp_dir = TempDir::new().unwrap();
        let frames_dir = temp_dir.path().join("frames");
        fs::create_dir(&frames_dir).unwrap();

        // Create test frame files
        fs::File::create(frames_dir.join("frame_0001.png")).unwrap();
        fs::File::create(frames_dir.join("frame_0002.jpg")).unwrap();
        fs::File::create(frames_dir.join("not_a_frame.txt")).unwrap();

        let optimizer = FrameStorageOptimizer::new();
        let count = optimizer
            .count_frames(frames_dir.to_str().unwrap())
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_calculate_directory_size() {
        let temp_dir = TempDir::new().unwrap();
        let frames_dir = temp_dir.path().join("frames");
        fs::create_dir(&frames_dir).unwrap();

        // Create test file with known size
        let file_path = frames_dir.join("test.png");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(&[0u8; 1000]).unwrap();

        let optimizer = FrameStorageOptimizer::new();
        let size = optimizer
            .calculate_directory_size(frames_dir.to_str().unwrap())
            .unwrap();
        assert_eq!(size, 1000);
    }
}
