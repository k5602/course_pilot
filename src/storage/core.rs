#![allow(dead_code)]
//! Core storage: database connection pool, schema initialization, optimization, and metrics.
//!
//! This module centralizes the SQLite lifecycle for Course Pilot, without any migration
//! subsystem. It creates all required tables (including video_progress) eagerly at startup
//! and provides maintenance utilities like optimization and performance metrics.

use anyhow::{Context, Result};

use log::{info, warn};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::Connection;
use std::path::Path;
use std::sync::Arc;

use uuid::Uuid;

/// Type alias for the database connection pool
type DbPool = Pool<SqliteConnectionManager>;

/// Type alias for a pooled connection
type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Database connection manager that holds the connection pool
#[derive(Clone)]
pub struct Database {
    pool: Arc<DbPool>,
}

impl Database {
    /// Initialize a new database connection pool and schema (no migrations).
    pub fn new(db_path: &Path) -> Result<Self> {
        info!("Initializing database at: {}", db_path.display());

        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                info!("Creating database directory: {}", parent.display());
                std::fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create database directory {}", parent.display())
                })?;
            }
        }

        // Clean up any existing WAL/SHM files from previous runs
        if let Err(e) = Self::cleanup_wal_files(db_path) {
            warn!("Failed to cleanup WAL files: {e}");
            // Continue anyway as this is not critical
        }

        // Create SQLite connection manager
        let manager = SqliteConnectionManager::file(db_path).with_init(|conn| {
            // Enable foreign key support
            conn.pragma_update(None, "foreign_keys", "ON")?;

            // Checkpoint any existing WAL data before changing journal mode
            let _ = conn.pragma_update(None, "wal_checkpoint", "TRUNCATE");

            // Use DELETE journal mode for desktop apps - no WAL files
            conn.pragma_update(None, "journal_mode", "DELETE")?;

            // Set busy timeout to 5 seconds
            conn.busy_timeout(std::time::Duration::from_secs(5))?;

            // Verify journal mode was set correctly
            let journal_mode: String =
                conn.pragma_query_value(None, "journal_mode", |row| row.get(0))?;
            log::info!("Database journal mode set to: {journal_mode}");

            Ok(())
        });

        // Create connection pool
        let pool = Pool::builder()
            .max_size(10) // Maximum number of connections in the pool
            .min_idle(Some(2)) // Minimum idle connections to maintain
            .build(manager)
            .with_context(|| "Failed to create database connection pool")?;

        // Initialize database schema
        let mut conn = pool
            .get()
            .with_context(|| "Failed to get initial database connection")?;

        init_tables(&mut conn).with_context(|| "Failed to initialize database tables")?;

        Ok(Database {
            pool: Arc::new(pool),
        })
    }

    /// Get a connection from the pool with error handling
    pub fn get_conn(&self) -> Result<PooledConnection> {
        self.pool
            .get()
            .with_context(|| "Failed to get database connection from pool")
    }

    /// Get a reference to the underlying pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// Check connection pool health and return metrics
    pub fn check_pool_health(&self) -> Result<ConnectionPoolHealth> {
        let state = self.pool.state();

        // Test a connection to ensure the pool is working
        let test_result = match self.pool.get() {
            Ok(_conn) => true,
            Err(e) => {
                log::warn!("Connection pool health check failed: {}", e);
                false
            }
        };

        Ok(ConnectionPoolHealth {
            total_connections: state.connections,
            idle_connections: state.idle_connections,
            active_connections: state.connections - state.idle_connections,
            is_healthy: test_result,
            max_connections: 10, // From pool configuration
        })
    }

    /// Execute an operation with simple retry logic for transient failures
    pub fn execute_with_retry<F, R>(&self, operation: F) -> Result<R>
    where
        F: Fn(&Connection) -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_DELAY_MS: u64 = 100;

        let mut last_error: Option<anyhow::Error> = None;

        for attempt in 0..MAX_RETRIES {
            match self.get_conn() {
                Ok(conn) => match operation(&conn) {
                    Ok(result) => return Ok(result),
                    Err(e) => {
                        last_error = Some(e);
                        if attempt < MAX_RETRIES - 1 {
                            let delay = INITIAL_DELAY_MS * 2_u64.pow(attempt);
                            log::warn!(
                                "Database operation failed (attempt {}), retrying in {}ms: {}",
                                attempt + 1,
                                delay,
                                last_error.as_ref().unwrap()
                            );
                            std::thread::sleep(std::time::Duration::from_millis(delay));
                        }
                    }
                },
                Err(e) => {
                    last_error = Some(e);
                    if attempt < MAX_RETRIES - 1 {
                        let delay = INITIAL_DELAY_MS * 2_u64.pow(attempt);
                        log::warn!(
                            "Failed to get database connection (attempt {}), retrying in {}ms: {}",
                            attempt + 1,
                            delay,
                            last_error.as_ref().unwrap()
                        );
                        std::thread::sleep(std::time::Duration::from_millis(delay));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("All retry attempts failed")))
    }

    /// Clean up WAL and SHM files from previous runs
    fn cleanup_wal_files(db_path: &Path) -> Result<()> {
        let wal_path = db_path.with_extension("db-wal");
        let shm_path = db_path.with_extension("db-shm");

        // Remove WAL file if it exists
        if wal_path.exists() {
            std::fs::remove_file(&wal_path)
                .with_context(|| format!("Failed to remove WAL file {}", wal_path.display()))?;
            log::info!("Removed existing WAL file: {}", wal_path.display());
        }

        // Remove SHM file if it exists
        if shm_path.exists() {
            std::fs::remove_file(&shm_path)
                .with_context(|| format!("Failed to remove SHM file {}", shm_path.display()))?;
            log::info!("Removed existing SHM file: {}", shm_path.display());
        }

        Ok(())
    }
}

/// Connection pool health metrics
#[derive(Debug, Clone)]
pub struct ConnectionPoolHealth {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub is_healthy: bool,
    pub max_connections: u32,
}

/// Initialize the database and create necessary tables (no migrations).
pub fn init_db(db_path: &Path) -> Result<Database> {
    // Create database directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    // Clean up any existing WAL/SHM files before creating the database
    Database::cleanup_wal_files(db_path)?;

    // Create the database file if it doesn't exist
    if !db_path.exists() {
        std::fs::File::create(db_path)?;
    }

    // Create the database and connection pool
    let db = Database::new(db_path)?;

    // Get a mutable connection from the pool for table initialization
    let mut conn = db.get_conn()?;

    // Initialize tables using the mutable connection
    init_tables(&mut conn)?;

    // Run initial optimization
    if let Err(e) = optimize_database(&db) {
        warn!("Initial database optimization failed: {e}");
        // Continue anyway as this is not critical for basic functionality
    }

    Ok(db)
}

/// Initialize database tables (no migration system, creates tables idempotently).
pub fn init_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    // Core tables
    tx.execute(
        r#"
        CREATE TABLE IF NOT EXISTS courses (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            raw_titles TEXT NOT NULL,
            videos TEXT,
            structure TEXT
        );
        "#,
        [],
    )?;

    tx.execute(
        r#"
        CREATE TABLE IF NOT EXISTS plans (
            id TEXT PRIMARY KEY,
            course_id TEXT NOT NULL,
            settings TEXT NOT NULL,
            items TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            FOREIGN KEY(course_id) REFERENCES courses(id) ON DELETE CASCADE
        );
        "#,
        [],
    )?;

    // Video progress tracking (previously created via migrations v4)
    tx.execute(
        r#"
        CREATE TABLE IF NOT EXISTS video_progress (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            plan_id TEXT NOT NULL,
            session_index INTEGER NOT NULL,
            video_index INTEGER NOT NULL,
            completed BOOLEAN NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL,
            UNIQUE(plan_id, session_index, video_index)
        );
        "#,
        [],
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_video_progress_plan_session
         ON video_progress(plan_id, session_index);",
        [],
    )?;

    tx.commit()?;

    // Initialize notes table schema and indexes via the notes module
    crate::storage::notes::init_notes_table(conn)
        .with_context(|| "Notes table initialization failed")?;

    // Secondary indexes (idempotent)
    let tx = conn.transaction()?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_plans_course_id ON plans(course_id);",
        [],
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_courses_created_at ON courses(created_at);",
        [],
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_courses_name ON courses(name);",
        [],
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_plans_created_at ON plans(created_at);",
        [],
    )?;
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_plans_course_created ON plans(course_id, created_at);",
        [],
    )?;
    tx.commit()?;

    Ok(())
}

/// Optimize database performance by running maintenance operations (standalone, no migrations).
pub fn optimize_database(db: &Database) -> Result<()> {
    info!("Running database optimization");

    let conn = db.get_conn()?;

    // Update table statistics for query optimizer
    conn.execute("ANALYZE", [])?;
    info!("Updated table statistics");

    // Rebuild indexes if needed (only if database is large)
    let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
    if page_count > 1000 {
        info!("Database is large ({page_count} pages), rebuilding indexes");
        conn.execute("REINDEX", [])?;
    }

    // Optimize database file structure
    conn.execute("PRAGMA optimize", [])?;
    info!("Optimized database file structure");

    // Check for integrity issues
    let integrity_check: String = conn
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .with_context(|| "Failed to run database integrity check")?;
    if integrity_check != "ok" {
        warn!("Database integrity check failed: {integrity_check}");
        return Err(anyhow::anyhow!("Integrity check failed: {integrity_check}"));
    }

    info!("Database optimization completed successfully");
    Ok(())
}

/// Get database performance metrics
pub fn get_database_performance_metrics(db: &Database) -> Result<DatabasePerformanceMetrics> {
    let conn = db.get_conn()?;

    // Get basic database info
    let page_count: i64 = conn.query_row("PRAGMA page_count", [], |row| row.get(0))?;
    let page_size: i64 = conn.query_row("PRAGMA page_size", [], |row| row.get(0))?;
    let freelist_count: i64 = conn.query_row("PRAGMA freelist_count", [], |row| row.get(0))?;

    // Get table row counts
    let courses_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM courses", [], |row| row.get(0))?;
    let plans_count: i64 = conn.query_row("SELECT COUNT(*) FROM plans", [], |row| row.get(0))?;
    let notes_count: i64 = conn.query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))?;

    // Calculate fragmentation
    let fragmentation_ratio = if page_count > 0 {
        freelist_count as f64 / page_count as f64
    } else {
        0.0
    };

    Ok(DatabasePerformanceMetrics {
        total_size_bytes: (page_count * page_size) as usize,
        page_count: page_count as usize,
        page_size: page_size as usize,
        free_pages: freelist_count as usize,
        fragmentation_ratio,
        courses_count: courses_count as usize,
        plans_count: plans_count as usize,
        notes_count: notes_count as usize,
        connection_pool_active: db.pool().state().connections,
        connection_pool_idle: db.pool().state().idle_connections,
    })
}

/// Database performance metrics
#[derive(Debug, Clone)]
pub struct DatabasePerformanceMetrics {
    pub total_size_bytes: usize,
    pub page_count: usize,
    pub page_size: usize,
    pub free_pages: usize,
    pub fragmentation_ratio: f64,
    pub courses_count: usize,
    pub plans_count: usize,
    pub notes_count: usize,
    pub connection_pool_active: u32,
    pub connection_pool_idle: u32,
}

/// Helper to parse UUID from a string to keep compatibility with pre-existing helpers
pub fn parse_uuid_sqlite(s: &str, idx: usize) -> Result<Uuid, rusqlite::Error> {
    Uuid::parse_str(s).map_err(|_| {
        rusqlite::Error::InvalidColumnType(idx, "uuid".to_string(), rusqlite::types::Type::Text)
    })
}

/// Helper to parse JSON from string for row mappers
pub fn parse_json_sqlite<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, rusqlite::Error> {
    serde_json::from_str(s).map_err(|e| {
        rusqlite::Error::InvalidColumnType(0, format!("json: {e}"), rusqlite::types::Type::Text)
    })
}
