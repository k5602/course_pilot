//! Persistence - SQLite adapter using Diesel.

mod connection;
pub mod models;
mod repositories;
mod search_repository;
mod tag_repository;

pub use connection::{DbPool, establish_connection};
pub use models::{CourseTagRow, NewTag, TagRow, UpdatePreferences, UserPreferencesRow};
pub use repositories::{
    SqliteCourseRepository, SqliteExamRepository, SqliteModuleRepository, SqliteNoteRepository,
    SqliteVideoRepository,
};
pub use search_repository::SqliteSearchRepository;
pub use tag_repository::SqliteTagRepository;
