//! Text embedding port for ML.

use crate::domain::value_objects::Embedding;

/// Error type for embedding operations.
#[derive(Debug, thiserror::Error)]
pub enum EmbedError {
    #[error("Model loading error: {0}")]
    ModelLoad(String),
    #[error("Embedding generation error: {0}")]
    Generation(String),
}

/// Port for generating text embeddings.
pub trait TextEmbedder: Send + Sync {
    /// Generates an embedding for a single text.
    fn embed(&self, text: &str) -> Result<Embedding, EmbedError>;

    /// Generates embeddings for multiple texts (batch).
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError>;
}
