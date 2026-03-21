use crate::contexts::frame::infrastructure::PerceptualHasher;

pub struct HashComparer;

impl HashComparer {
    pub fn calculate_similarity(hash1: &str, hash2: &str) -> f64 {
        PerceptualHasher::compute_similarity(hash1, hash2)
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
