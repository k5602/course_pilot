//! Database connection management.

use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::application::context::AppContextError;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

/// Establishes a connection pool to the SQLite database.
pub fn establish_connection(database_url: &str) -> Result<DbPool, AppContextError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder().max_size(5).build(manager).map_err(|e| {
        AppContextError::Database(format!("Failed to create connection pool: {}", e))
    })?;

    let mut conn = pool
        .get()
        .map_err(|e| AppContextError::Database(format!("Failed to get connection: {}", e)))?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| AppContextError::Database(format!("Failed to run migrations: {}", e)))?;

    Ok(pool)
}
