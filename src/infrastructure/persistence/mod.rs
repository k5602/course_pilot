//! Persistence - SQLite adapter using Diesel.

mod connection;
mod models;
mod repositories;

pub use connection::{DbPool, establish_connection};
pub use repositories::{
    SqliteCourseRepository, SqliteExamRepository, SqliteModuleRepository, SqliteVideoRepository,
};
