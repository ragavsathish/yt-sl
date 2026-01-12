//! Perceptual hash computation infrastructure.
//!
//! This module provides perceptual hash computation functionality as specified in US-FRAME-02:
//! Compute Perceptual Hash.
//!
//! Features:
//! - Compute perceptual hash for each frame
//! - Use ahash algorithm for efficient similarity comparison
//! - Store hash with frame metadata
//! - Support multiple hash algorithms (average, difference, dhash)
//! - Provide hash comparison functions

use crate::contexts::frame::domain::commands::{ComputeHashCommand, HashAlgorithm};
use crate::contexts::frame::domain::events::validate_hash_params;
use crate::contexts::frame::domain::handlers::handle_compute_hash;
use crate::contexts::frame::domain::state::HashComputed;
use crate::shared::domain::{DomainResult, ExtractionError, Id};
use ahash::AHasher;
use image::{DynamicImage, ImageBuffer, Luma};
use std::hash::Hasher;
use std::time::Instant;

/// Perceptual hash computer.
///
/// This struct handles perceptual hash computation for frame images.
pub struct PerceptualHasher {
    /// Hash algorithm to use
    algorithm: HashAlgorithm,
    /// Hash size (for average and difference hash)
    hash_size: u32,
}

impl PerceptualHasher {
    /// Creates a new perceptual hasher with default settings.
    pub fn new() -> Self {
        Self {
            algorithm: HashAlgorithm::Average,
            hash_size: 8, // 8x8 hash
        }
    }

    /// Creates a new perceptual hasher with custom algorithm.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The hash algorithm to use
    pub fn with_algorithm(algorithm: HashAlgorithm) -> Self {
        Self {
            algorithm,
            hash_size: 8,
        }
    }

    /// Creates a new perceptual hasher with custom hash size.
    ///
    /// # Arguments
    ///
    /// * `algorithm` - The hash algorithm to use
    /// * `hash_size` - The hash size (e.g., 8 for 8x8)
    pub fn with_settings(algorithm: HashAlgorithm, hash_size: u32) -> Self {
        Self {
            algorithm,
            hash_size,
        }
    }

    /// Computes perceptual hash for a frame.
    ///
    /// This function provides perceptual hash computation functionality as specified in US-FRAME-02:
    /// Compute Perceptual Hash.
    ///
    /// # Arguments
    ///
    /// * `command` - The compute hash command
    ///
    /// # Returns
    ///
    /// A HashComputed event
    ///
    /// # Errors
    ///
    /// Returns an error if hash computation fails
    pub fn compute_hash(&self, command: ComputeHashCommand) -> DomainResult<HashComputed> {
        validate_hash_params(command.algorithm, self.hash_size)?;

        let start = Instant::now();
        let frame_id = command.frame_id;

        // Load the image
        let img = image::open(&command.frame_path).map_err(|_e| {
            ExtractionError::HashComputationFailed(frame_id.clone())
        })?;

        // Convert to grayscale
        let gray_img = img.to_luma8();

        // Compute hash based on algorithm
        let hash = match command.algorithm {
            HashAlgorithm::Average => self.compute_average_hash(&gray_img),
            HashAlgorithm::Difference => self.compute_difference_hash(&gray_img),
            HashAlgorithm::Perceptual => self.compute_perceptual_hash(&gray_img),
        };

        let computation_time_ms = start.elapsed().as_millis() as u64;

        Ok(HashComputed {
            frame_id,
            hash,
            algorithm: format!("{:?}", command.algorithm),
            computation_time_ms,
        })
    }

    /// Computes average hash.
    ///
    /// # Arguments
    ///
    /// * `img` - Grayscale image
    ///
    /// # Returns
    ///
    /// The hash as a hexadecimal string
    fn compute_average_hash(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> String {
        // Resize to hash_size x hash_size
        let small = image::imageops::resize(
            img,
            self.hash_size,
            self.hash_size,
            image::imageops::FilterType::Lanczos3,
        );

        // Calculate average pixel value
        let mut sum: u64 = 0;
        for pixel in small.pixels() {
            sum += pixel[0] as u64;
        }
        let avg = (sum / (self.hash_size * self.hash_size) as u64) as u8;

        // Generate hash: 1 if pixel >= avg, 0 otherwise
        let mut hash_bits = Vec::with_capacity((self.hash_size * self.hash_size) as usize);
        for pixel in small.pixels() {
            hash_bits.push(if pixel[0] >= avg { 1u8 } else { 0u8 });
        }

        self.bits_to_hex(&hash_bits)
    }

    /// Computes difference hash.
    ///
    /// # Arguments
    ///
    /// * `img` - Grayscale image
    ///
    /// # Returns
    ///
    /// The hash as a hexadecimal string
    fn compute_difference_hash(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> String {
        // Resize to (hash_size+1) x hash_size
        let width = self.hash_size + 1;
        let small = image::imageops::resize(
            img,
            width,
            self.hash_size,
            image::imageops::FilterType::Lanczos3,
        );

        // Compute differences between adjacent pixels
        let mut hash_bits = Vec::with_capacity((self.hash_size * self.hash_size) as usize);
        for y in 0..self.hash_size {
            for x in 0..self.hash_size {
                let left = small.get_pixel(x, y)[0];
                let right = small.get_pixel(x + 1, y)[0];
                hash_bits.push(if left >= right { 1u8 } else { 0u8 });
            }
        }

        self.bits_to_hex(&hash_bits)
    }

    /// Computes perceptual hash using DCT approximation.
    ///
    /// # Arguments
    ///
    /// * `img` - Grayscale image
    ///
    /// # Returns
    ///
    /// The hash as a hexadecimal string
    fn compute_perceptual_hash(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> String {
        // Resize to 32x32 for DCT approximation
        let small = image::imageops::resize(
            img,
            32,
            32,
            image::imageops::FilterType::Lanczos3,
        );

        // Simple approximation: use average hash on 8x8 version
        let tiny = image::imageops::resize(
            &small,
            8,
            8,
            image::imageops::FilterType::Lanczos3,
        );

        // Calculate average pixel value
        let mut sum: u64 = 0;
        for pixel in tiny.pixels() {
            sum += pixel[0] as u64;
        }
        let avg = (sum / 64) as u8;

        // Generate hash
        let mut hash_bits = Vec::with_capacity(64);
        for pixel in tiny.pixels() {
            hash_bits.push(if pixel[0] >= avg { 1u8 } else { 0u8 });
        }

        self.bits_to_hex(&hash_bits)
    }

    /// Converts bits to hexadecimal string.
    ///
    /// # Arguments
    ///
    /// * `bits` - Vector of bits
    ///
    /// # Returns
    ///
    /// Hexadecimal string representation
    fn bits_to_hex(&self, bits: &[u8]) -> String {
        let mut hex = String::new();
        for chunk in bits.chunks(4) {
            let mut value = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit == 1 {
                    value |= 1 << (3 - i);
                }
            }
            hex.push_str(&format!("{:x}", value));
        }
        hex
    }

    /// Computes similarity between two hashes.
    ///
    /// # Arguments
    ///
    /// * `hash1` - First hash
    /// * `hash2` - Second hash
    ///
    /// # Returns
    ///
    /// Similarity score between 0.0 (completely different) and 1.0 (identical)
    pub fn compute_similarity(hash1: &str, hash2: &str) -> f64 {
        if hash1.len() != hash2.len() {
            return 0.0;
        }

        let mut differences = 0;
        for (c1, c2) in hash1.chars().zip(hash2.chars()) {
            let v1 = c1.to_digit(16).unwrap_or(0);
            let v2 = c2.to_digit(16).unwrap_or(0);
            differences += (v1 ^ v2).count_ones();
        }

        let total_bits = hash1.len() * 4;
        if total_bits == 0 {
            return 1.0;
        }

        1.0 - (differences as f64 / total_bits as f64)
    }

    /// Computes ahash using ahash crate for efficient hashing.
    ///
    /// # Arguments
    ///
    /// * `data` - Data to hash
    ///
    /// # Returns
    ///
    /// The hash as a hexadecimal string
    pub fn compute_ahash(data: &[u8]) -> String {
        let mut hasher = AHasher::default();
        hasher.write(data);
        let hash = hasher.finish();
        format!("{:016x}", hash)
    }

    /// Computes a combined hash using multiple algorithms.
    ///
    /// # Arguments
    ///
    /// * `img` - Grayscale image
    ///
    /// # Returns
    ///
    /// A combined hash string
    pub fn compute_combined_hash(
        &self,
        img: &ImageBuffer<Luma<u8>, Vec<u8>>,
    ) -> String {
        let avg_hash = self.compute_average_hash(img);
        let diff_hash = self.compute_difference_hash(img);
        format!("{}|{}", avg_hash, diff_hash)
    }
}

impl Default for PerceptualHasher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contexts::frame::domain::commands::HashAlgorithm;

    #[test]
    fn test_perceptual_hasher_new() {
        let hasher = PerceptualHasher::new();
        assert_eq!(hasher.algorithm, HashAlgorithm::Average);
        assert_eq!(hasher.hash_size, 8);
    }

    #[test]
    fn test_perceptual_hasher_with_algorithm() {
        let hasher = PerceptualHasher::with_algorithm(HashAlgorithm::Difference);
        assert_eq!(hasher.algorithm, HashAlgorithm::Difference);
        assert_eq!(hasher.hash_size, 8);
    }

    #[test]
    fn test_perceptual_hasher_with_settings() {
        let hasher = PerceptualHasher::with_settings(HashAlgorithm::Perceptual, 16);
        assert_eq!(hasher.algorithm, HashAlgorithm::Perceptual);
        assert_eq!(hasher.hash_size, 16);
    }

    #[test]
    fn test_bits_to_hex() {
        let hasher = PerceptualHasher::new();
        let bits = vec![1, 0, 1, 0, 1, 1, 0, 0, 1, 1];
        let hex = hasher.bits_to_hex(&bits);
        assert_eq!(hex, "acc");
    }

    #[test]
    fn test_compute_similarity_identical() {
        let hash1 = "a1b2c3d4e5f6";
        let hash2 = "a1b2c3d4e5f6";
        let similarity = PerceptualHasher::compute_similarity(hash1, hash2);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_compute_similarity_completely_different() {
        let hash1 = "a1b2c3d4e5f6";
        let hash2 = "fedcba987654";
        let similarity = PerceptualHasher::compute_similarity(hash1, hash2);
        assert!(similarity < 0.5);
    }

    #[test]
    fn test_compute_similarity_partial_match() {
        let hash1 = "a1b2c3d4e5f6";
        let hash2 = "a1b2c3d4e5f0"; // Only last digit differs
        let similarity = PerceptualHasher::compute_similarity(hash1, hash2);
        assert!(similarity > 0.9);
    }

    #[test]
    fn test_compute_ahash() {
        let data = b"test data";
        let hash = PerceptualHasher::compute_ahash(data);
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 16);
    }

    #[test]
    fn test_perceptual_hasher_default() {
        let hasher = PerceptualHasher::default();
        assert_eq!(hasher.algorithm, HashAlgorithm::Average);
        assert_eq!(hasher.hash_size, 8);
    }
}
