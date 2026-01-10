//! Database connection management.

use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

/// Establishes a connection pool to the SQLite database.
pub fn establish_connection(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    Pool::builder().max_size(5).build(manager).expect("Failed to create database connection pool")
}
