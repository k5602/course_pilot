//! Ports - Trait definitions for external dependencies.
//! These define the contracts that infrastructure adapters must implement.

mod keystore;
mod llm;
mod repository;
mod youtube;

pub use keystore::{KeystoreError, SecretStore};
pub use llm::{CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion, SummarizerAI};
pub use repository::{
    CourseRepository, ExamRepository, ModuleRepository, NoteRepository, RepositoryError,
    SearchRepository, TagRepository, VideoRepository,
};
pub use youtube::{FetchError, PlaylistFetcher, RawVideoMetadata};
