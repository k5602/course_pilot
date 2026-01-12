//! ML adapter using fastembed for local text embeddings.

#[cfg(feature = "ml")]
use fastembed::TextEmbedding;
#[cfg(feature = "ml")]
use std::sync::Mutex;

use crate::domain::ports::{EmbedError, TextEmbedder};
use crate::domain::value_objects::Embedding;

/// FastEmbed adapter for local text embeddings.
///
/// Uses the default model (BGE-small-en-v1.5) which provides good quality
/// embeddings with a small footprint.
#[cfg(feature = "ml")]
pub struct FastEmbedAdapter {
    model: Mutex<TextEmbedding>,
}

#[cfg(feature = "ml")]
impl FastEmbedAdapter {
    /// Creates a new FastEmbed adapter with the default model (bge-small-en-v1.5).
    pub fn new() -> Result<Self, EmbedError> {
        // Default::default() uses BGESmallENV15 model
        let model = TextEmbedding::try_new(Default::default())
            .map_err(|e| EmbedError::ModelLoad(e.to_string()))?;

        Ok(Self { model: Mutex::new(model) })
    }
}

#[cfg(feature = "ml")]
impl TextEmbedder for FastEmbedAdapter {
    fn embed(&self, text: &str) -> Result<Embedding, EmbedError> {
        let mut model = self.model.lock().map_err(|e| EmbedError::Generation(e.to_string()))?;

        let embeddings =
            model.embed(vec![text], None).map_err(|e| EmbedError::Generation(e.to_string()))?;

        embeddings
            .into_iter()
            .next()
            .map(Embedding::new)
            .ok_or_else(|| EmbedError::Generation("No embedding generated".to_string()))
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        let mut model = self.model.lock().map_err(|e| EmbedError::Generation(e.to_string()))?;

        let text_vec: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        let embeddings =
            model.embed(text_vec, None).map_err(|e| EmbedError::Generation(e.to_string()))?;

        Ok(embeddings.into_iter().map(Embedding::new).collect())
    }
}

/// Stub adapter when ML feature is disabled (e.g., on Windows)
#[cfg(not(feature = "ml"))]
pub struct FastEmbedAdapter;

#[cfg(not(feature = "ml"))]
impl FastEmbedAdapter {
    pub fn new() -> Result<Self, EmbedError> {
        Err(EmbedError::ModelLoad("ML feature is disabled".to_string()))
    }
}

#[cfg(not(feature = "ml"))]
impl TextEmbedder for FastEmbedAdapter {
    fn embed(&self, _text: &str) -> Result<Embedding, EmbedError> {
        Err(EmbedError::Generation("ML feature is disabled".to_string()))
    }

    fn embed_batch(&self, _texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        Err(EmbedError::Generation("ML feature is disabled".to_string()))
    }
}
