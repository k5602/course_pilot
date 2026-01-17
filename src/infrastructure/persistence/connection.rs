//! Database connection management.

use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Establishes a connection pool to the SQLite database.
pub fn establish_connection(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(5)
        .build(manager)
        .expect("Failed to create database connection pool");

    let mut conn = pool.get().expect("Failed to get database connection");
    conn.run_pending_migrations(MIGRATIONS).expect("Failed to run database migrations");

    pool
}
