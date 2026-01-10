//! ML adapter using fastembed.

use crate::domain::ports::{EmbedError, TextEmbedder};
use crate::domain::value_objects::Embedding;

/// FastEmbed adapter for local text embeddings.
pub struct FastEmbedAdapter {
    // TODO: Add fastembed::TextEmbedding
}

impl FastEmbedAdapter {
    pub fn new() -> Result<Self, EmbedError> {
        // TODO: Initialize fastembed model
        // fastembed::TextEmbedding::try_new(Default::default())
        Ok(Self {})
    }
}

impl Default for FastEmbedAdapter {
    fn default() -> Self {
        Self::new().expect("Failed to initialize FastEmbed")
    }
}

impl TextEmbedder for FastEmbedAdapter {
    fn embed(&self, text: &str) -> Result<Embedding, EmbedError> {
        // TODO: Implement with fastembed
        let _ = text;
        todo!("Implement with fastembed crate")
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        // TODO: Implement batch embedding with fastembed
        let _ = texts;
        todo!("Implement with fastembed crate")
    }
}
