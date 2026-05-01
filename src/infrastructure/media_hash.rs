use sha2::{Digest, Sha256};

pub fn compute_source_hash(source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_input_produces_same_hash() {
        let a = compute_source_hash("/videos/course");
        let b = compute_source_hash("/videos/course");
        assert_eq!(a, b);
    }

    #[test]
    fn different_input_produces_different_hash() {
        let a = compute_source_hash("/videos/course");
        let b = compute_source_hash("/videos/other");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_is_non_empty() {
        let h = compute_source_hash("/videos/test");
        assert!(!h.is_empty());
    }
}
