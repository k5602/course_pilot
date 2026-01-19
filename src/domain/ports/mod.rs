//! Ports - Trait definitions for external dependencies.
//! These define the contracts that infrastructure adapters must implement.

mod keystore;
mod llm;
mod local_media;
mod repository;
mod transcript;
mod youtube;

pub use keystore::{KeystoreError, SecretStore};
pub use llm::{CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion, SummarizerAI};
pub use local_media::{LocalMediaError, LocalMediaScanner, RawLocalMediaMetadata};
pub use repository::{
    CourseRepository, ExamRepository, ModuleRepository, NoteRepository, RepositoryError,
    SearchRepository, TagRepository, UserPreferencesRepository, VideoRepository,
};
pub use transcript::{TranscriptError, TranscriptProvider};
pub use youtube::{FetchError, PlaylistFetcher, RawVideoMetadata};
