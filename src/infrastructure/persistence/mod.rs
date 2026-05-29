//! Persistence - SQLite adapter using Diesel.

mod chat_message_repository;
mod connection;
pub mod models;
mod preferences_repository;
mod repositories;
mod search_repository;
mod tag_repository;

pub use chat_message_repository::SqliteChatMessageRepository;
pub use connection::{DbPool, establish_connection};
pub use models::{
    ChatMessageRow, CourseTagRow, NewChatMessage, NewTag, TagRow, UpdatePreferences,
    UserPreferencesRow,
};
pub use preferences_repository::SqliteUserPreferencesRepository;
pub use repositories::{
    SqliteCourseRepository, SqliteExamRepository, SqliteModuleRepository, SqliteNoteRepository,
    SqliteVideoRepository,
};
pub use search_repository::SqliteSearchRepository;
pub use tag_repository::SqliteTagRepository;
