//! Persistence - SQLite adapter using Diesel.

mod connection;
pub mod models;
mod repositories;

pub use connection::{DbPool, establish_connection};
pub use models::{UpdatePreferences, UserPreferencesRow};
pub use repositories::{
    SqliteCourseRepository, SqliteExamRepository, SqliteModuleRepository, SqliteNoteRepository,
    SqliteVideoRepository,
};
