// Hash comparer for perceptual hashes
pub struct HashComparer;

impl HashComparer {
    /// Calculates the similarity score between two perceptual hashes (0.0 to 1.0).
    /// Uses Hamming distance on the hexadecimal representation.
    pub fn calculate_similarity(hash1: &str, hash2: &str) -> f64 {
        if hash1.is_empty() || hash2.is_empty() || hash1.len() != hash2.len() {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_similarity_identical() {
        let hash1 = "ffffffffffffffff";
        let hash2 = "ffffffffffffffff";
        assert_eq!(HashComparer::calculate_similarity(hash1, hash2), 1.0);
    }

    #[test]
    fn test_calculate_similarity_completely_different() {
        let hash1 = "ffffffffffffffff";
        let hash2 = "0000000000000000";
        assert_eq!(HashComparer::calculate_similarity(hash1, hash2), 0.0);
    }

    #[test]
    fn test_calculate_similarity_partial() {
        // One bit difference in one nibble
        // f is 1111, e is 1110 -> 1 bit difference
        let hash1 = "ffffffffffffffff";
        let hash2 = "fffffffffffffffe";
        let total_bits = 16 * 4; // 64 bits
        let expected = 1.0 - (1.0 / total_bits as f64);
        assert_eq!(HashComparer::calculate_similarity(hash1, hash2), expected);
    }

    #[test]
    fn test_calculate_similarity_different_lengths() {
        let hash1 = "ffff";
        let hash2 = "ffffff";
        assert_eq!(HashComparer::calculate_similarity(hash1, hash2), 0.0);
    }
}
