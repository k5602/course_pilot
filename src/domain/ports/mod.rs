//! Ports - Trait definitions for external dependencies.
//! These define the contracts that infrastructure adapters must implement.

mod chat_repository;
mod event_bus;
mod keystore;
mod llm;
mod local_media;
mod presence;
mod repository;
mod stream;
mod transcript;
mod youtube;

pub use event_bus::{DomainEvent, EventBus};

pub use chat_repository::{ChatMessage, ChatMessageRepository, ChatRole};
pub use keystore::{KeystoreError, SecretStore};
pub use llm::{
    CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion, ModuleTitleGenerator,
    SummarizerAI,
};
pub use local_media::{
    LocalMediaError, LocalMediaScanner, RawLocalMediaMetadata, RawSubtitleMetadata,
};
pub use presence::{Activity, PresenceProvider};
pub use repository::{
    CourseRepository, ExamRepository, ModuleRepository, NoteRepository, RepositoryError,
    SearchEntry, SearchRepository, TagRepository, UserPreferencesRepository, VideoRepository,
};
pub use stream::StreamResolver;
pub use transcript::{TranscriptError, TranscriptProvider};
pub use youtube::{FetchError, PlaylistFetcher, RawVideoMetadata};
