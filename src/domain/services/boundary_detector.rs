//! Boundary Detector - Groups videos into modules.

/// Groups videos into modules using a simple batch size approach.
/// Each module contains up to `batch_size` videos, keeping related content together.
#[derive(Debug)]
pub struct BoundaryDetector {
    batch_size: usize,
}

impl BoundaryDetector {
    /// Creates a boundary detector with default batch size (8 videos per module).
    pub fn new() -> Self {
        Self { batch_size: 5 }
    }

    /// Creates a boundary detector with a custom batch size.
    pub fn with_batch_size(batch_size: usize) -> Self {
        Self { batch_size: batch_size.max(1) }
    }

    /// Groups video indices into modules (each module has up to `batch_size` videos).
    /// Returns a vector of vectors, where each inner vector contains video indices for a module.
    pub fn group_into_modules(&self, video_count: usize) -> Vec<Vec<usize>> {
        if video_count == 0 {
            return vec![];
        }

        (0..video_count)
            .collect::<Vec<_>>()
            .chunks(self.batch_size)
            .map(|chunk| chunk.to_vec())
            .collect()
    }
}

impl Default for BoundaryDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_videos() {
        let detector = BoundaryDetector::new();
        let modules = detector.group_into_modules(0);
        assert!(modules.is_empty());
    }

    #[test]
    fn test_single_module() {
        let detector = BoundaryDetector::new();
        let modules = detector.group_into_modules(5);
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0], vec![0, 1, 2, 3, 4]);
    }

    #[test]
    fn test_multiple_modules() {
        let detector = BoundaryDetector::with_batch_size(3);
        let modules = detector.group_into_modules(7);
        assert_eq!(modules.len(), 3);
        assert_eq!(modules[0], vec![0, 1, 2]);
        assert_eq!(modules[1], vec![3, 4, 5]);
        assert_eq!(modules[2], vec![6]);
    }

    #[test]
    fn test_exact_batch_size() {
        let detector = BoundaryDetector::with_batch_size(4);
        let modules = detector.group_into_modules(8);
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0], vec![0, 1, 2, 3]);
        assert_eq!(modules[1], vec![4, 5, 6, 7]);
    }
}
