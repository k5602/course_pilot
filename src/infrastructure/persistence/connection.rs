//! Database connection management.

use diesel::SqliteConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

use crate::application::context::AppContextError;

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

#[derive(Debug)]
struct SqliteCustomizer;

impl diesel::r2d2::CustomizeConnection<SqliteConnection, diesel::r2d2::Error> for SqliteCustomizer {
    fn on_acquire(&self, conn: &mut SqliteConnection) -> Result<(), diesel::r2d2::Error> {
        use diesel::RunQueryDsl;
        diesel::sql_query("PRAGMA foreign_keys = ON;")
            .execute(conn)
            .map_err(diesel::r2d2::Error::QueryError)?;
        Ok(())
    }
}

/// Establishes a connection pool to the SQLite database.
pub fn establish_connection(database_url: &str) -> Result<DbPool, AppContextError> {
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(5)
        .connection_customizer(Box::new(SqliteCustomizer))
        .build(manager)
        .map_err(|e| {
            AppContextError::Database(format!("Failed to create connection pool: {}", e))
        })?;

    let mut conn = pool
        .get()
        .map_err(|e| AppContextError::Database(format!("Failed to get connection: {}", e)))?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| AppContextError::Database(format!("Failed to run migrations: {}", e)))?;

    Ok(pool)
}
