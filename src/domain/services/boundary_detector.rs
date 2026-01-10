//! Boundary Detector - Identifies module boundaries using embeddings.

use crate::domain::value_objects::Embedding;

/// Threshold for similarity drop to indicate a boundary.
const DEFAULT_THRESHOLD: f32 = 0.7;

/// Detects module boundaries by analyzing similarity between adjacent embeddings.
#[derive(Debug)]
pub struct BoundaryDetector {
    threshold: f32,
}

impl BoundaryDetector {
    /// Creates a new boundary detector with the default threshold.
    pub fn new() -> Self {
        Self { threshold: DEFAULT_THRESHOLD }
    }

    /// Creates a new boundary detector with a custom threshold.
    /// Threshold should be between 0.0 and 1.0.
    pub fn with_threshold(threshold: f32) -> Self {
        Self { threshold: threshold.clamp(0.0, 1.0) }
    }

    /// Detects boundaries in a sequence of embeddings.
    /// Returns indices where boundaries should be placed (AFTER that index).
    ///
    /// For example, if returns [2, 5], boundaries are after videos at indices 2 and 5:
    /// - Module 1: videos 0, 1, 2
    /// - Module 2: videos 3, 4, 5  
    /// - Module 3: videos 6, ...
    pub fn detect_boundaries(&self, embeddings: &[Embedding]) -> Vec<usize> {
        if embeddings.len() < 2 {
            return vec![];
        }

        let mut boundaries = Vec::new();

        for i in 0..embeddings.len() - 1 {
            let similarity = embeddings[i].cosine_similarity(&embeddings[i + 1]);
            if similarity < self.threshold {
                boundaries.push(i);
            }
        }

        boundaries
    }

    /// Groups video indices into modules based on detected boundaries.
    /// Returns a vector of vectors, where each inner vector contains video indices for a module.
    pub fn group_into_modules(&self, embeddings: &[Embedding]) -> Vec<Vec<usize>> {
        let boundaries = self.detect_boundaries(embeddings);
        let mut modules = Vec::new();
        let mut current_module = Vec::new();

        for i in 0..embeddings.len() {
            current_module.push(i);
            if boundaries.contains(&i) {
                modules.push(current_module);
                current_module = Vec::new();
            }
        }

        // Don't forget the last module
        if !current_module.is_empty() {
            modules.push(current_module);
        }

        modules
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
    fn test_no_boundaries_similar_embeddings() {
        let detector = BoundaryDetector::new();
        let embeddings = vec![
            Embedding::new(vec![1.0, 0.0, 0.0]),
            Embedding::new(vec![0.95, 0.1, 0.0]),
            Embedding::new(vec![0.9, 0.2, 0.0]),
        ];
        let boundaries = detector.detect_boundaries(&embeddings);
        assert!(boundaries.is_empty());
    }

    #[test]
    fn test_boundary_on_topic_shift() {
        let detector = BoundaryDetector::with_threshold(0.5);
        let embeddings = vec![
            Embedding::new(vec![1.0, 0.0, 0.0]),  // Topic A
            Embedding::new(vec![0.9, 0.1, 0.0]),  // Topic A
            Embedding::new(vec![0.0, 1.0, 0.0]),  // Topic B - shift!
            Embedding::new(vec![0.1, 0.95, 0.0]), // Topic B
        ];
        let boundaries = detector.detect_boundaries(&embeddings);
        assert_eq!(boundaries, vec![1]); // Boundary after index 1
    }

    #[test]
    fn test_group_into_modules() {
        let detector = BoundaryDetector::with_threshold(0.5);
        let embeddings = vec![
            Embedding::new(vec![1.0, 0.0, 0.0]),
            Embedding::new(vec![0.9, 0.1, 0.0]),
            Embedding::new(vec![0.0, 1.0, 0.0]), // Shift here
            Embedding::new(vec![0.1, 0.95, 0.0]),
        ];
        let modules = detector.group_into_modules(&embeddings);
        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0], vec![0, 1]);
        assert_eq!(modules[1], vec![2, 3]);
    }
}
