//! Ports - Trait definitions for external dependencies.
//! These define the contracts that infrastructure adapters must implement.

mod embedder;
mod keystore;
mod llm;
mod repository;
mod youtube;

pub use embedder::{EmbedError, TextEmbedder};
pub use keystore::{KeystoreError, SecretStore};
pub use llm::{CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion};
pub use repository::{
    CourseRepository, ExamRepository, ModuleRepository, NoteRepository, RepositoryError,
    VideoRepository,
};
pub use youtube::{FetchError, PlaylistFetcher, RawVideoMetadata};
