//! Database operations for Course Pilot
//!
//! This module provides SQLite-based persistence for courses and study plans
//! using JSON serialization for complex data structures.
//! It uses connection pooling for better performance under load.

use crate::error_handling::DatabaseError;
use crate::types::{Course, Plan};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json;
use std::path::Path;
use std::sync::Arc;

use uuid::Uuid;

/// Helper function to parse UUID from string, returning rusqlite::Error
fn parse_uuid_sqlite(s: &str, idx: usize) -> Result<Uuid, rusqlite::Error> {
    Uuid::parse_str(s).map_err(|_| {
        rusqlite::Error::InvalidColumnType(idx, "uuid".to_string(), rusqlite::types::Type::Text)
    })
}

/// Helper function to parse JSON from string, returning rusqlite::Error
fn parse_json_sqlite<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, rusqlite::Error> {
    serde_json::from_str(s).map_err(|e| {
        rusqlite::Error::InvalidColumnType(0, format!("json: {e}"), rusqlite::types::Type::Text)
    })
}

/// Type alias for the database connection pool
type DbPool = Pool<SqliteConnectionManager>;

/// Database connection manager that holds the connection pool
#[derive(Clone)]
pub struct Database {
    pool: Arc<DbPool>,
}

impl Database {
    /// Initialize a new database connection pool
    pub fn new(db_path: &Path) -> Result<Self> {
        info!("Initializing database at: {}", db_path.display());

        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                info!("Creating database directory: {}", parent.display());
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("Failed to create database directory {}", parent.display()))
                    .map_err(|e| DatabaseError::ConnectionFailed { 
                        message: format!("Could not create database directory: {}", e) 
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
            .map_err(|e| DatabaseError::ConnectionFailed { 
                message: format!("Failed to create connection pool: {}", e) 
            })
            .with_context(|| "Failed to create database connection pool")?;

        // Initialize database schema
        let mut conn = pool.get()
            .map_err(|e| DatabaseError::ConnectionFailed { 
                message: format!("Failed to get connection for initialization: {}", e) 
            })
            .with_context(|| "Failed to get initial database connection")?;
        
        init_tables(&mut conn)
            .with_context(|| "Failed to initialize database tables")?;

        Ok(Database {
            pool: Arc::new(pool),
        })
    }

    /// Get a connection from the pool with error handling
    pub fn get_conn(&self) -> Result<PooledConnection> {
        self.pool.get()
            .map_err(|e| DatabaseError::Pool(e))
            .with_context(|| "Failed to get database connection from pool")
            .map_err(Into::into)
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

    /// Execute a query with automatic retry on connection failures
    pub fn execute_with_retry<F, R>(&self, operation: F) -> Result<R>
    where
        F: Fn(&Connection) -> Result<R> + Send + 'static,
        R: Send + 'static,
    {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_DELAY_MS: u64 = 100;

        let mut last_error = None;
        
        for attempt in 0..MAX_RETRIES {
            match self.get_conn() {
                Ok(conn) => {
                    match operation(&conn) {
                        Ok(result) => return Ok(result),
                        Err(e) => {
                            last_error = Some(e);
                            if attempt < MAX_RETRIES - 1 {
                                let delay = INITIAL_DELAY_MS * 2_u64.pow(attempt);
                                log::warn!("Database operation failed (attempt {}), retrying in {}ms: {}", 
                                          attempt + 1, delay, last_error.as_ref().unwrap());
                                std::thread::sleep(std::time::Duration::from_millis(delay));
                            }
                        }
                    }
                }
                Err(e) => {
                    last_error = Some(e);
                    if attempt < MAX_RETRIES - 1 {
                        let delay = INITIAL_DELAY_MS * 2_u64.pow(attempt);
                        log::warn!("Failed to get database connection (attempt {}), retrying in {}ms: {}", 
                                  attempt + 1, delay, last_error.as_ref().unwrap());
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
                .with_context(|| format!("Failed to remove WAL file {}", wal_path.display()))
                .map_err(|e| {
                    log::warn!("Failed to remove WAL file {}: {}", wal_path.display(), e);
                    e
                })?;
            log::info!("Removed existing WAL file: {}", wal_path.display());
        }

        // Remove SHM file if it exists
        if shm_path.exists() {
            std::fs::remove_file(&shm_path)
                .with_context(|| format!("Failed to remove SHM file {}", shm_path.display()))
                .map_err(|e| {
                    log::warn!("Failed to remove SHM file {}: {}", shm_path.display(), e);
                    e
                })?;
            log::info!("Removed existing SHM file: {}", shm_path.display());
        }

        Ok(())
    }
}

/// Type alias for a pooled connection
type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Connection pool health metrics
#[derive(Debug, Clone)]
pub struct ConnectionPoolHealth {
    pub total_connections: u32,
    pub idle_connections: u32,
    pub active_connections: u32,
    pub is_healthy: bool,
    pub max_connections: u32,
}

/// Helper function to detect if a title looks like a YouTube video
fn is_youtube_video_title(title: &str) -> bool {
    // Simple heuristics to detect YouTube videos
    if title.contains("youtube.com") || title.contains("youtu.be") || title.contains("watch?v=") {
        return true;
    }
    
    // Check if it looks like a typical YouTube video title
    // YouTube titles typically don't have file extensions and are reasonable length
    let has_video_extension = title.to_lowercase().ends_with(".mp4") || 
                             title.to_lowercase().ends_with(".avi") || 
                             title.to_lowercase().ends_with(".mov") || 
                             title.to_lowercase().ends_with(".mkv") || 
                             title.to_lowercase().ends_with(".webm");
    
    // If it doesn't have a video file extension and is reasonable length, assume YouTube
    !has_video_extension && title.len() > 5 && title.len() < 200
}

/// Helper function to extract YouTube playlist ID from URL
fn extract_playlist_id_from_url(url: &str) -> Option<String> {
    // Look for list= parameter in YouTube URLs
    if let Some(start) = url.find("list=") {
        let id_start = start + 5; // length of "list="
        if let Some(end) = url[id_start..].find('&') {
            Some(url[id_start..id_start + end].to_string())
        } else {
            // No more parameters, take the rest
            Some(url[id_start..].to_string())
        }
    } else {
        None
    }
}

/// Helper function to extract YouTube video ID from title if present
fn extract_youtube_video_id_from_title(title: &str) -> Option<String> {
    // Try to extract video ID from URL patterns in title
    if let Some(start) = title.find("watch?v=") {
        let id_start = start + 8; // length of "watch?v="
        if let Some(end) = title[id_start..].find('&') {
            Some(title[id_start..id_start + end].to_string())
        } else if let Some(end) = title[id_start..].find(' ') {
            Some(title[id_start..id_start + end].to_string())
        } else {
            let remaining = &title[id_start..];
            if remaining.len() == 11 { // YouTube video IDs are 11 characters
                Some(remaining.to_string())
            } else {
                None
            }
        }
    } else if let Some(start) = title.find("youtu.be/") {
        let id_start = start + 9; // length of "youtu.be/"
        if let Some(end) = title[id_start..].find('?') {
            Some(title[id_start..id_start + end].to_string())
        } else if let Some(end) = title[id_start..].find(' ') {
            Some(title[id_start..id_start + end].to_string())
        } else {
            let remaining = &title[id_start..];
            if remaining.len() == 11 {
                Some(remaining.to_string())
            } else {
                None
            }
        }
    } else {
        None
    }
}

/// Validate video metadata to ensure YouTube fields are not lost during database operations
fn validate_video_metadata(videos: &[crate::types::VideoMetadata]) -> Result<Vec<crate::types::VideoMetadata>> {
    let mut validated_videos = Vec::new();
    
    for (index, video) in videos.iter().enumerate() {
        // Debug logging to see what we're validating
        info!("Validating video {}: title='{}', video_id={:?}, source_url={:?}, is_local={}", 
              index, video.title, video.video_id, video.source_url, video.is_local);
        
        let mut validated_video = video.clone();
        
        // Validate YouTube videos have required metadata
        if !video.is_local {
            // Check if this is a YouTube video missing critical metadata
            if video.video_id.is_none() && video.source_url.is_none() {
                warn!("YouTube video at index {} missing both video_id and source_url: '{}'", index, video.title);
                
                // Try to extract video_id from title if possible
                if let Some(extracted_id) = extract_youtube_video_id_from_title(&video.title) {
                    info!("Extracted video_id '{}' from title for video at index {}", extracted_id, index);
                    validated_video.video_id = Some(extracted_id.clone());
                    validated_video.source_url = Some(format!("https://www.youtube.com/watch?v={}", extracted_id));
                    // Try to extract playlist_id from source_url if available
                    if let Some(ref url) = validated_video.source_url {
                        validated_video.playlist_id = extract_playlist_id_from_url(url);
                    }
                } else {
                    // Instead of creating placeholders, return an error to prevent corruption
                    return Err(anyhow::anyhow!(
                        "Cannot save YouTube video '{}' at index {}: missing video_id and cannot extract from title. This indicates an import error. Raw video data: video_id={:?}, source_url={:?}, is_local={}",
                        video.title, index, video.video_id, video.source_url, video.is_local
                    ).into());
                }
            } else if video.video_id.is_some() && video.source_url.is_none() {
                // Has video_id but missing source_url, reconstruct it
                if let Some(ref video_id) = video.video_id {
                    let url = if let Some(ref playlist_id) = video.playlist_id {
                        format!("https://www.youtube.com/watch?v={}&list={}", video_id, playlist_id)
                    } else {
                        format!("https://www.youtube.com/watch?v={}", video_id)
                    };
                    validated_video.source_url = Some(url);
                }
            } else if video.video_id.is_none() && video.source_url.is_some() {
                // Has source_url but missing video_id, try to extract it
                if let Some(ref url) = video.source_url {
                    if let Some(extracted_id) = extract_youtube_video_id_from_title(url) {
                        validated_video.video_id = Some(extracted_id);
                    }
                    // Also try to extract playlist_id
                    if validated_video.playlist_id.is_none() {
                        validated_video.playlist_id = extract_playlist_id_from_url(url);
                    }
                }
            } else {
                // YouTube video should have both video_id and source_url - validate them
                if let Some(ref video_id) = video.video_id {
                    if video_id.starts_with("PLACEHOLDER_") {
                        return Err(anyhow::anyhow!(
                            "Cannot save YouTube video '{}' at index {}: contains placeholder video_id '{}'. This indicates corrupted import data.",
                            video.title, index, video_id
                        ).into());
                    }
                }
                
                // Video appears to have valid YouTube metadata, keep as-is
            }
        } else {
            // For local videos, ensure source_url is set to file path
            if video.source_url.is_none() {
                validated_video.source_url = Some(video.title.clone());
            }
        }
        
        validated_videos.push(validated_video);
    }
    
    Ok(validated_videos)
}

/// Run database migrations using the new migration system
fn run_database_migrations(conn: &mut Connection) -> Result<()> {
    use crate::storage::migrations::MigrationManager;
    
    let migration_manager = MigrationManager::new();
    migration_manager.migrate(conn)?;
    
    // Validate database after migrations
    let validation_report = migration_manager.validate_database(conn)?;
    if !validation_report.is_valid {
        error!("Database validation failed after migrations: {:?}", validation_report.issues);
        return Err(anyhow::anyhow!("Database validation failed: {:?}", validation_report.issues));
    }
    
    if !validation_report.warnings.is_empty() {
        warn!("Database validation warnings: {:?}", validation_report.warnings);
    }
    
    Ok(())
}



/// Validate and repair loaded metadata to ensure complete VideoMetadata including video_id, playlist_id
fn validate_and_repair_loaded_metadata(
    parsed_videos: Vec<crate::types::VideoMetadata>, 
    raw_titles: &[String]
) -> Result<Vec<crate::types::VideoMetadata>> {
    let mut repaired_videos = Vec::new();
    
    for (index, video) in parsed_videos.into_iter().enumerate() {
        let mut repaired_video = video.clone();
        
        // Check if this video needs repair
        if !video.is_local && video.video_id.is_none() && video.source_url.is_none() {
            // This is a YouTube video with missing metadata, try to repair it
            log::warn!("Found YouTube video with missing metadata during load, repairing: '{}'", video.title);
            
            if is_youtube_video_title(&video.title) {
                if let Some(video_id) = extract_youtube_video_id_from_title(&video.title) {
                    log::info!("Repaired video_id '{}' from title for video at index {}", video_id, index);
                    repaired_video.video_id = Some(video_id.clone());
                    repaired_video.source_url = Some(format!("https://www.youtube.com/watch?v={}", video_id));
                    // Set original_index if not already set
                    if repaired_video.original_index == 0 && index > 0 {
                        repaired_video.original_index = index;
                    }
                } else {
                    // Create placeholder metadata that will work with title analysis
                    log::warn!("Could not extract video_id from title, creating placeholder for video at index {}", index);
                    repaired_video.video_id = Some(format!("PLACEHOLDER_{}", index));
                    repaired_video.source_url = Some(format!("https://www.youtube.com/watch?v=PLACEHOLDER_{}", index));
                    repaired_video.playlist_id = None;
                    // Set original_index if not already set
                    if repaired_video.original_index == 0 && index > 0 {
                        repaired_video.original_index = index;
                    }
                }
            } else {
                // Assume local video if it doesn't look like YouTube
                log::info!("Converting assumed YouTube video to local video at index {}", index);
                repaired_video.is_local = true;
                repaired_video.source_url = Some(video.title.clone());
            }
        } else if !video.is_local && video.video_id.is_some() && video.source_url.is_none() {
            // Has video_id but missing source_url, reconstruct it
            if let Some(ref video_id) = video.video_id {
                log::info!("Reconstructing source_url for video_id '{}' at index {}", video_id, index);
                let url = if let Some(ref playlist_id) = repaired_video.playlist_id {
                    format!("https://www.youtube.com/watch?v={}&list={}", video_id, playlist_id)
                } else {
                    format!("https://www.youtube.com/watch?v={}", video_id)
                };
                repaired_video.source_url = Some(url);
            }
        } else if !video.is_local && video.video_id.is_none() && video.source_url.is_some() {
            // Has source_url but missing video_id, try to extract it
            if let Some(ref url) = video.source_url {
                if let Some(extracted_id) = extract_youtube_video_id_from_title(url) {
                    log::info!("Extracted video_id '{}' from source_url at index {}", extracted_id, index);
                    repaired_video.video_id = Some(extracted_id);
                }
                // Also try to extract playlist_id
                if repaired_video.playlist_id.is_none() {
                    repaired_video.playlist_id = extract_playlist_id_from_url(url);
                }
            }
        } else if video.is_local && video.source_url.is_none() {
            // Local video missing source_url (file path)
            log::info!("Setting source_url for local video at index {}", index);
            repaired_video.source_url = Some(video.title.clone());
        }
        
        // Ensure original_index is set correctly
        if repaired_video.original_index != index {
            repaired_video.original_index = index;
        }
        
        repaired_videos.push(repaired_video);
    }
    
    // If we have fewer videos than raw_titles, pad with fallback metadata
    if repaired_videos.len() < raw_titles.len() {
        log::warn!("Video metadata count ({}) less than raw_titles count ({}), padding with fallback", 
                   repaired_videos.len(), raw_titles.len());
        
        for i in repaired_videos.len()..raw_titles.len() {
            let fallback_video = if is_youtube_video_title(&raw_titles[i]) {
                if let Some(video_id) = extract_youtube_video_id_from_title(&raw_titles[i]) {
                    crate::types::VideoMetadata::new_youtube_with_playlist(
                        raw_titles[i].clone(),
                        video_id.clone(),
                        format!("https://www.youtube.com/watch?v={}", video_id),
                        None,
                        i
                    )
                } else {
                    crate::types::VideoMetadata {
                        title: raw_titles[i].clone(),
                        source_url: Some(format!("https://www.youtube.com/watch?v=PLACEHOLDER_{}", i)),
                        video_id: Some(format!("PLACEHOLDER_{}", i)),
                        playlist_id: None,
                        original_index: i,
                        duration_seconds: None,
                        thumbnail_url: None,
                        description: None,
                        upload_date: None,
                        author: None,
                        view_count: None,
                        tags: Vec::new(),
                        is_local: false,
                    }
                }
            } else {
                crate::types::VideoMetadata::new_local_with_index(raw_titles[i].clone(), raw_titles[i].clone(), i)
            };
            
            repaired_videos.push(fallback_video);
        }
    }
    
    Ok(repaired_videos)
}

/// Create fallback video metadata from raw titles with intelligent detection
fn create_fallback_video_metadata(raw_titles: &[String]) -> Vec<crate::types::VideoMetadata> {
    raw_titles.iter().enumerate().map(|(index, title)| {
        log::info!("Creating fallback metadata for video {}: '{}'", index, title);
        
        // Try to detect if this is a YouTube video based on title patterns
        if is_youtube_video_title(title) {
            log::info!("Detected as YouTube video: '{}'", title);
            if let Some(video_id) = extract_youtube_video_id_from_title(title) {
                log::info!("Extracted video ID '{}' from title: '{}'", video_id, title);
                crate::types::VideoMetadata::new_youtube_with_playlist(
                    title.clone(),
                    video_id.clone(),
                    format!("https://www.youtube.com/watch?v={}", video_id),
                    None, // No playlist_id available from title alone
                    index
                )
            } else {
                log::warn!("YouTube video detected but could not extract ID from title: '{}'", title);
                // For YouTube videos without extractable ID, create a placeholder that will trigger title analysis
                crate::types::VideoMetadata {
                    title: title.clone(),
                    source_url: Some(format!("https://www.youtube.com/watch?v=PLACEHOLDER_{}", index)),
                    video_id: Some(format!("PLACEHOLDER_{}", index)),
                    playlist_id: None,
                    original_index: index,
                    duration_seconds: None,
                    thumbnail_url: None,
                    description: None,
                    upload_date: None,
                    author: None,
                    view_count: None,
                    tags: Vec::new(),
                    is_local: false,
                }
            }
        } else {
            log::info!("Detected as local video: '{}'", title);
            // Assume local video
            crate::types::VideoMetadata::new_local_with_index(title.clone(), title.clone(), index)
        }
    }).collect()
}

/// Database version for migration tracking (now handled by migrations.rs)
const DATABASE_VERSION: i32 = 3;

/// Initialize database tables
fn init_tables(conn: &mut Connection) -> Result<()> {
    let tx = conn.transaction()?;

    // Create database version table for migration tracking
    tx.execute(
        r#"
        CREATE TABLE IF NOT EXISTS database_version (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );
        "#,
        [],
    )?;

    // Create courses table
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

    // Create plans table
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

    // Commit the transaction first so we can call init_notes_table
    tx.commit()?;

    // Initialize notes table with proper migration logic
    crate::storage::notes::init_notes_table(conn)
        .with_context(|| "Notes table initialization failed")?;

    // Run database migrations
    run_database_migrations(conn)?;

    // Start a new transaction for the remaining operations
    let tx = conn.transaction()?;

    // Create indices for better performance
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_plans_course_id ON plans(course_id);",
        [],
    )?;

    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_courses_created_at ON courses(created_at);",
        [],
    )?;

    // Additional performance indexes
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_courses_name ON courses(name);",
        [],
    )?;

    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_plans_created_at ON plans(created_at);",
        [],
    )?;

    // Composite index for common query patterns
    tx.execute(
        "CREATE INDEX IF NOT EXISTS idx_plans_course_created ON plans(course_id, created_at);",
        [],
    )?;

    tx.commit()?;
    Ok(())
}

/// Optimize database performance by running maintenance operations
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
    let integrity_check: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .with_context(|| "Failed to run database integrity check")?;
    if integrity_check != "ok" {
        warn!("Database integrity check failed: {integrity_check}");
        return Err(DatabaseError::CorruptionDetected { 
            table: "database".to_string(), 
            message: format!("Integrity check failed: {integrity_check}") 
        }.into());
    }

    info!("Database optimization completed successfully");
    Ok(())
}

/// Get database performance metrics
pub fn get_database_performance_metrics(
    db: &Database,
) -> Result<DatabasePerformanceMetrics> {
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

/// Manually run database migrations (useful for troubleshooting)
pub fn run_migrations(db: &Database) -> Result<()> {
    info!("Manually running database migrations");
    let mut conn = db.get_conn()?;
    run_database_migrations(&mut conn)?;
    info!("Manual database migrations completed");
    Ok(())
}

/// Initialize the database and create necessary tables
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

/// Save a course to the database
pub fn save_course(db: &Database, course: &Course) -> Result<()> {
    info!("Saving course: {} (ID: {})", course.name, course.id);

    let raw_titles_json = serde_json::to_string(&course.raw_titles)
        .with_context(|| format!("Failed to serialize raw_titles for course '{}'", course.name))
        .map_err(|e| {
            error!("Failed to serialize raw_titles for course '{}': {}", course.name, e);
            e
        })?;

    // Validate and serialize video metadata with proper YouTube field preservation
    let validated_videos = validate_video_metadata(&course.videos)?;
    let videos_json = serde_json::to_string(&validated_videos)
        .with_context(|| format!("Failed to serialize videos for course '{}'", course.name))
        .map_err(|e| {
            error!("Failed to serialize videos for course '{}': {}", course.name, e);
            e
        })?;
    
    // Debug logging to see what we're saving
    log::info!("Saving course '{}' with {} videos", course.name, validated_videos.len());
    if !validated_videos.is_empty() {
        let first_video = &validated_videos[0];
        log::info!("First video: title='{}', video_id={:?}, source_url={:?}, is_local={}", 
                   first_video.title, first_video.video_id, first_video.source_url, first_video.is_local);
        
        // Validate YouTube metadata is not lost
        if !first_video.is_local && first_video.video_id.is_none() {
            warn!("YouTube video missing video_id: '{}'", first_video.title);
        }
    }

    let structure_json = course
        .structure
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(|e| {
            error!(
                "Failed to serialize structure for course {}: {}",
                course.name, e
            );
            DatabaseError::Serialization(e)
        })?;

    let conn = db.get_conn().map_err(|e| {
        error!(
            "Failed to get database connection for saving course {}: {}",
            course.name, e
        );
        e
    })?;

    conn.execute(
        r#"
        INSERT OR REPLACE INTO courses (id, name, created_at, raw_titles, videos, structure)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        "#,
        params![
            course.id.to_string(),
            course.name,
            course.created_at.timestamp(),
            raw_titles_json,
            videos_json,
            structure_json
        ],
    )
    .map_err(|e| {
        error!(
            "Failed to execute SQL for saving course {}: {}",
            course.name, e
        );
        DatabaseError::Sqlite(e)
    })?;

    info!("Successfully saved course: {}", course.name);
    Ok(())
}

/// Load all courses from the database with optimized query
pub fn load_courses(db: &Database) -> Result<Vec<Course>> {
    info!("Loading all courses from database");

    let conn = db.get_conn().map_err(|e| {
        error!("Failed to get database connection for loading courses: {e}");
        e
    })?;

    // Use index on created_at for efficient ordering
    let mut stmt = conn
        .prepare(
            r#"
        SELECT id, name, created_at, raw_titles, videos, structure
        FROM courses
        ORDER BY created_at DESC
        "#,
        )
        .map_err(|e| {
            error!("Failed to prepare SQL statement for loading courses: {e}");
            DatabaseError::Sqlite(e)
        })?;

    let courses = stmt
        .query_map([], |row| {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let name: String = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            let raw_titles_json: String = row.get(3)?;
            let videos_json: Option<String> = row.get(4)?;
            let structure_json: Option<String> = row.get(5)?;

            let raw_titles: Vec<String> = serde_json::from_str(&raw_titles_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            // Load videos with fallback to raw_titles for backward compatibility
            let videos: Vec<crate::types::VideoMetadata> = if let Some(ref videos_json) = videos_json {
                let parsed_videos: Vec<crate::types::VideoMetadata> = serde_json::from_str(videos_json).unwrap_or_else(|e| {
                    log::warn!("Failed to deserialize video metadata, using intelligent fallback: {}", e);
                    create_fallback_video_metadata(&raw_titles)
                });
                
                // Validate and fix any incomplete metadata during loading
                validate_and_repair_loaded_metadata(parsed_videos, &raw_titles).unwrap_or_else(|e| {
                    log::error!("Failed to repair loaded metadata: {}", e);
                    create_fallback_video_metadata(&raw_titles)
                })
            } else {
                log::info!("No video metadata found, creating from raw_titles");
                create_fallback_video_metadata(&raw_titles)
            };

            let structure = structure_json
                .as_ref()
                .map(|json| {
                    serde_json::from_str(json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            5,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })
                })
                .transpose()?;

            Ok(Course {
                id,
                name,
                created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
                raw_titles,
                videos,
                structure,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(courses)
}

/// Get a specific course by ID
pub fn get_course_by_id(db: &Database, course_id: &Uuid) -> Result<Option<Course>> {
    let conn = db.get_conn()?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, created_at, raw_titles, videos, structure
        FROM courses
        WHERE id = ?1
        "#,
    )?;

    let course = stmt
        .query_row(params![course_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let name: String = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            let raw_titles_json: String = row.get(3)?;
            let videos_json: Option<String> = row.get(4)?;
            let structure_json: Option<String> = row.get(5)?;

            let raw_titles: Vec<String> = serde_json::from_str(&raw_titles_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            // Load videos with fallback to raw_titles for backward compatibility
            let videos: Vec<crate::types::VideoMetadata> = if let Some(ref videos_json) = videos_json {
                let parsed_videos: Vec<crate::types::VideoMetadata> = serde_json::from_str(videos_json).unwrap_or_else(|e| {
                    log::warn!("Failed to deserialize video metadata for course {}, using intelligent fallback: {}", id, e);
                    create_fallback_video_metadata(&raw_titles)
                });
                
                // Validate and fix any incomplete metadata during loading
                validate_and_repair_loaded_metadata(parsed_videos, &raw_titles).unwrap_or_else(|e| {
                    log::error!("Failed to repair loaded metadata: {}", e);
                    create_fallback_video_metadata(&raw_titles)
                })
            } else {
                log::info!("No video metadata found for course {}, creating from raw_titles", id);
                create_fallback_video_metadata(&raw_titles)
            };

            let structure = structure_json
                .as_ref()
                .map(|json| serde_json::from_str(json))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(Course {
                id,
                name,
                created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
                raw_titles,
                videos,
                structure,
            })
        })
        .optional()?;

    Ok(course)
}

/// Save a plan to the database
pub fn save_plan(db: &Database, plan: &Plan) -> Result<()> {
    let settings_json = serde_json::to_string(&plan.settings)?;
    let items_json = serde_json::to_string(&plan.items)?;

    let conn = db.get_conn()?;

    conn.execute(
        r#"
        INSERT OR REPLACE INTO plans (id, course_id, settings, items, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            plan.id.to_string(),
            plan.course_id.to_string(),
            settings_json,
            items_json,
            plan.created_at.timestamp(),
        ],
    )?;

    Ok(())
}

/// Load a plan by course ID
pub fn get_plan_by_course_id(
    db: &Database,
    course_id: &Uuid,
) -> Result<Option<Plan>> {
    let conn = db.get_conn()?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, course_id, settings, items, created_at
        FROM plans
        WHERE course_id = ?1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )?;

    let plan = stmt
        .query_row(params![course_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let course_id_str: String = row.get(1)?;
            let course_id = Uuid::parse_str(&course_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let settings_json: String = row.get(2)?;
            let settings = serde_json::from_str(&settings_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    2,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let items_json: String = row.get(3)?;
            let items = serde_json::from_str(&items_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let created_at: i64 = row.get(4)?;

            Ok(Plan {
                id,
                course_id,
                settings,
                items,
                created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
            })
        })
        .optional()?;

    Ok(plan)
}

/// Load a specific plan by ID
pub fn load_plan(db: &Database, plan_id: &Uuid) -> Result<Option<Plan>> {
    let conn = db.get_conn()?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, course_id, settings, items, created_at
        FROM plans
        WHERE id = ?1
        "#,
    )?;

    let _plan = stmt.query_row(params![plan_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let course_id_str: String = row.get(1)?;
        let course_id = Uuid::parse_str(&course_id_str).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let settings_json: String = row.get(2)?;
        let settings = serde_json::from_str(&settings_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let items_json: String = row.get(3)?;
        let items = serde_json::from_str(&items_json).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(3, rusqlite::types::Type::Text, Box::new(e))
        })?;

        let created_at: i64 = row.get(4)?;

        Ok(Plan {
            id,
            course_id,
            settings,
            items,
            created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
        })
    })?;

    // Helper function to parse UUID from string, returning rusqlite::Error
    fn parse_uuid_sqlite(s: &str, idx: usize) -> Result<Uuid, rusqlite::Error> {
        Uuid::parse_str(s).map_err(|_| {
            rusqlite::Error::InvalidColumnType(idx, "uuid".to_string(), rusqlite::types::Type::Text)
        })
    }

    // Helper function to parse JSON from string, returning rusqlite::Error
    fn parse_json_sqlite<T: serde::de::DeserializeOwned>(s: &str) -> Result<T, rusqlite::Error> {
        serde_json::from_str(s).map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
        })
    }

    // Execute the query and get the first row if it exists
    let result: Result<Option<Plan>, DatabaseError> = {
        // First, try to get the row and handle the Option<Result<...>>
        match stmt.query_row(params![plan_id.to_string()], |row| {
            // Get the ID
            let id_str: String = row.get(0)?;
            let id = parse_uuid_sqlite(&id_str, 0)?;

            // Get the course ID
            let course_id_str: String = row.get(1)?;
            let course_id = parse_uuid_sqlite(&course_id_str, 1)?;

            // Get settings JSON
            let settings_json: String = row.get(2)?;
            let settings = parse_json_sqlite(&settings_json)?;

            // Get items JSON
            let items_json: String = row.get(3)?;
            let items = parse_json_sqlite(&items_json)?;

            // Get created_at timestamp
            let created_at: i64 = row.get(4)?;

            Ok(Plan {
                id,
                course_id,
                settings,
                items,
                created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
            })
        }) {
            Ok(plan) => Ok(Some(plan)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    };

    result.map_err(Into::into)
}

/// Delete a course from the database
pub fn delete_course(db: &Database, course_id: &Uuid) -> Result<()> {
    let mut conn = db.get_conn()?;

    // Start a transaction to ensure atomicity
    let tx = conn.transaction()?;

    // Delete associated plans first (due to foreign key constraint)
    tx.execute(
        "DELETE FROM plans WHERE course_id = ?1",
        params![course_id.to_string()],
    )?;

    // Then delete the course
    tx.execute(
        "DELETE FROM courses WHERE id = ?1",
        params![course_id.to_string()],
    )?;

    // Commit the transaction
    tx.commit()?;

    Ok(())
}

/// Delete a plan from the database
pub fn delete_plan(db: &Database, plan_id: &Uuid) -> Result<()> {
    let conn = db.get_conn()?;
    conn.execute(
        "DELETE FROM plans WHERE id = ?1",
        params![plan_id.to_string()],
    )?;
    Ok(())
}

// ============================================================================
// ENHANCED CLUSTERING INTEGRATION
// ============================================================================

use crate::types::{ClusteringAlgorithm, ClusteringMetadata, ClusteringStrategy};

/// Clustering analytics for dashboard insights
#[derive(Debug, Clone, PartialEq)]
pub struct ClusteringAnalytics {
    pub total_courses: usize,
    pub clustered_courses: usize,
    pub average_quality_score: f32,
    pub algorithm_distribution: std::collections::HashMap<ClusteringAlgorithm, usize>,
    pub strategy_distribution: std::collections::HashMap<ClusteringStrategy, usize>,
    pub quality_distribution: QualityDistribution,
    pub processing_time_stats: ProcessingTimeStats,
}

/// Quality score distribution
#[derive(Debug, Clone, PartialEq)]
pub struct QualityDistribution {
    pub excellent: usize, // 0.8+
    pub good: usize,      // 0.6-0.8
    pub fair: usize,      // 0.4-0.6
    pub poor: usize,      // <0.4
}

/// Processing time statistics
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessingTimeStats {
    pub average_ms: f64,
    pub median_ms: f64,
    pub min_ms: u64,
    pub max_ms: u64,
}

/// Get courses filtered by clustering quality
pub fn get_courses_by_clustering_quality(
    db: &Database,
    min_quality: f32,
) -> Result<Vec<Course>> {
    let conn = db.get_conn()?;

    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, raw_titles, structure 
         FROM courses 
         WHERE structure IS NOT NULL",
    )?;

    let course_iter = stmt.query_map([], |row| {
        let structure_json: String = row.get(4)?;
        let structure: crate::types::CourseStructure = parse_json_sqlite(&structure_json)?;

        // Check if clustering quality meets threshold
        if let Some(clustering_metadata) = &structure.clustering_metadata {
            if clustering_metadata.quality_score >= min_quality {
                let raw_titles: Vec<String> = parse_json_sqlite(&row.get::<_, String>(3)?)?;
                let videos = raw_titles.iter().map(|title| crate::types::VideoMetadata::new_local(title.clone(), "".to_string())).collect();
                return Ok(Some(Course {
                    id: parse_uuid_sqlite(&row.get::<_, String>(0)?, 0)?,
                    name: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|_e| {
                            rusqlite::Error::InvalidColumnType(
                                2,
                                "created_at".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?
                        .with_timezone(&Utc),
                    raw_titles,
                    videos,
                    structure: Some(structure),
                }));
            }
        }
        Ok(None)
    })?;

    let mut courses = Vec::new();
    for course_result in course_iter {
        if let Some(course) = course_result? {
            courses.push(course);
        }
    }

    Ok(courses)
}

/// Get comprehensive clustering analytics
pub fn get_clustering_analytics(db: &Database) -> Result<ClusteringAnalytics> {
    let conn = db.get_conn()?;

    // Get total course count
    let total_courses: usize =
        conn.query_row("SELECT COUNT(*) FROM courses", [], |row| row.get(0))?;

    // Get courses with clustering data
    let mut stmt = conn.prepare("SELECT structure FROM courses WHERE structure IS NOT NULL")?;

    let structure_iter = stmt.query_map([], |row| {
        let structure_json: String = row.get(0)?;
        let structure: crate::types::CourseStructure = parse_json_sqlite(&structure_json)?;
        Ok(structure)
    })?;

    let mut clustered_courses = 0;
    let mut quality_scores = Vec::new();
    let mut algorithm_counts = std::collections::HashMap::new();
    let mut strategy_counts = std::collections::HashMap::new();
    let mut processing_times = Vec::new();

    for structure_result in structure_iter {
        let structure = structure_result?;

        if let Some(clustering_metadata) = structure.clustering_metadata {
            clustered_courses += 1;
            quality_scores.push(clustering_metadata.quality_score);
            processing_times.push(clustering_metadata.processing_time_ms);

            *algorithm_counts
                .entry(clustering_metadata.algorithm_used)
                .or_insert(0) += 1;
            *strategy_counts
                .entry(clustering_metadata.strategy_used)
                .or_insert(0) += 1;
        }
    }

    // Calculate statistics
    let average_quality_score = if !quality_scores.is_empty() {
        quality_scores.iter().sum::<f32>() / quality_scores.len() as f32
    } else {
        0.0
    };

    let quality_distribution = calculate_quality_distribution(&quality_scores);
    let processing_time_stats = calculate_processing_time_stats(&processing_times);

    Ok(ClusteringAnalytics {
        total_courses,
        clustered_courses,
        average_quality_score,
        algorithm_distribution: algorithm_counts,
        strategy_distribution: strategy_counts,
        quality_distribution,
        processing_time_stats,
    })
}

/// Update clustering metadata for an existing course
pub fn update_clustering_metadata(
    db: &Database,
    course_id: Uuid,
    metadata: ClusteringMetadata,
) -> Result<()> {
    let conn = db.get_conn()?;

    // Get current course structure
    let current_structure: crate::types::CourseStructure = conn.query_row(
        "SELECT structure FROM courses WHERE id = ?1",
        params![course_id.to_string()],
        |row| {
            let structure_json: String = row.get(0)?;
            let structure: crate::types::CourseStructure = parse_json_sqlite(&structure_json)?;
            Ok(structure)
        },
    )?;

    // Update clustering metadata
    let updated_structure = crate::types::CourseStructure {
        clustering_metadata: Some(metadata),
        ..current_structure
    };

    // Save updated structure
    let structure_json = serde_json::to_string(&updated_structure)?;
    conn.execute(
        "UPDATE courses SET structure = ?1 WHERE id = ?2",
        params![structure_json, course_id.to_string()],
    )?;

    Ok(())
}

/// Get courses with similar clustering characteristics
pub fn get_similar_courses_by_clustering(
    db: &Database,
    reference_course_id: Uuid,
    similarity_threshold: f32,
) -> Result<Vec<Course>> {
    let conn = db.get_conn()?;

    // Get reference course clustering metadata
    let reference_metadata: ClusteringMetadata = conn.query_row(
        "SELECT structure FROM courses WHERE id = ?1",
        params![reference_course_id.to_string()],
        |row| {
            let structure_json: String = row.get(0)?;
            let structure: crate::types::CourseStructure = parse_json_sqlite(&structure_json)?;
            Ok(structure.clustering_metadata.unwrap_or_default())
        },
    )?;

    // Find similar courses
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, raw_titles, structure 
         FROM courses 
         WHERE id != ?1 AND structure IS NOT NULL",
    )?;

    let course_iter = stmt.query_map(params![reference_course_id.to_string()], |row| {
        let structure_json: String = row.get(4)?;
        let structure: crate::types::CourseStructure = parse_json_sqlite(&structure_json)?;

        if let Some(clustering_metadata) = &structure.clustering_metadata {
            // Calculate similarity based on algorithm, strategy, and quality
            let similarity =
                calculate_clustering_similarity(&reference_metadata, clustering_metadata);

            if similarity >= similarity_threshold {
                let raw_titles: Vec<String> = parse_json_sqlite(&row.get::<_, String>(3)?)?;
                let videos = raw_titles.iter().map(|title| crate::types::VideoMetadata::new_local(title.clone(), "".to_string())).collect();
                return Ok(Some(Course {
                    id: parse_uuid_sqlite(&row.get::<_, String>(0)?, 0)?,
                    name: row.get(1)?,
                    created_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map_err(|_e| {
                            rusqlite::Error::InvalidColumnType(
                                2,
                                "created_at".to_string(),
                                rusqlite::types::Type::Text,
                            )
                        })?
                        .with_timezone(&Utc),
                    raw_titles,
                    videos,
                    structure: Some(structure),
                }));
            }
        }
        Ok(None)
    })?;

    let mut similar_courses = Vec::new();
    for course_result in course_iter {
        if let Some(course) = course_result? {
            similar_courses.push(course);
        }
    }

    Ok(similar_courses)
}

/// Get clustering performance history
pub fn get_clustering_performance_history(
    db: &Database,
    days: i64,
) -> Result<Vec<ClusteringPerformancePoint>> {
    let conn = db.get_conn()?;

    let cutoff_date = Utc::now() - chrono::Duration::days(days);

    let mut stmt = conn.prepare(
        "SELECT created_at, structure FROM courses 
         WHERE structure IS NOT NULL AND created_at >= ?1
         ORDER BY created_at ASC",
    )?;

    let performance_iter = stmt.query_map(params![cutoff_date.to_rfc3339()], |row| {
        let created_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(0)?)
            .map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    0,
                    "created_at".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?
            .with_timezone(&Utc);

        let structure_json: String = row.get(1)?;
        let structure: crate::types::CourseStructure = parse_json_sqlite(&structure_json)?;

        if let Some(clustering_metadata) = structure.clustering_metadata {
            return Ok(Some(ClusteringPerformancePoint {
                timestamp: created_at,
                quality_score: clustering_metadata.quality_score,
                processing_time_ms: clustering_metadata.processing_time_ms,
                algorithm_used: clustering_metadata.algorithm_used,
                strategy_used: clustering_metadata.strategy_used,
            }));
        }
        Ok(None)
    })?;

    let mut performance_points = Vec::new();
    for point_result in performance_iter {
        if let Some(point) = point_result? {
            performance_points.push(point);
        }
    }

    Ok(performance_points)
}

/// Clustering performance data point
#[derive(Debug, Clone)]
pub struct ClusteringPerformancePoint {
    pub timestamp: DateTime<Utc>,
    pub quality_score: f32,
    pub processing_time_ms: u64,
    pub algorithm_used: ClusteringAlgorithm,
    pub strategy_used: ClusteringStrategy,
}

// Helper functions

fn calculate_quality_distribution(quality_scores: &[f32]) -> QualityDistribution {
    let mut excellent = 0;
    let mut good = 0;
    let mut fair = 0;
    let mut poor = 0;

    for &score in quality_scores {
        match score {
            s if s >= 0.8 => excellent += 1,
            s if s >= 0.6 => good += 1,
            s if s >= 0.4 => fair += 1,
            _ => poor += 1,
        }
    }

    QualityDistribution {
        excellent,
        good,
        fair,
        poor,
    }
}

fn calculate_processing_time_stats(processing_times: &[u64]) -> ProcessingTimeStats {
    if processing_times.is_empty() {
        return ProcessingTimeStats {
            average_ms: 0.0,
            median_ms: 0.0,
            min_ms: 0,
            max_ms: 0,
        };
    }

    let mut sorted_times = processing_times.to_vec();
    sorted_times.sort_unstable();

    let average_ms = processing_times.iter().sum::<u64>() as f64 / processing_times.len() as f64;
    let median_ms = if sorted_times.len() % 2 == 0 {
        let mid = sorted_times.len() / 2;
        (sorted_times[mid - 1] + sorted_times[mid]) as f64 / 2.0
    } else {
        sorted_times[sorted_times.len() / 2] as f64
    };

    ProcessingTimeStats {
        average_ms,
        median_ms,
        min_ms: *sorted_times.first().unwrap_or(&0),
        max_ms: *sorted_times.last().unwrap_or(&0),
    }
}

fn calculate_clustering_similarity(
    metadata1: &ClusteringMetadata,
    metadata2: &ClusteringMetadata,
) -> f32 {
    let mut similarity: f32 = 0.0;

    // Algorithm similarity (0.3 weight)
    if metadata1.algorithm_used == metadata2.algorithm_used {
        similarity += 0.3;
    }

    // Strategy similarity (0.2 weight)
    if metadata1.strategy_used == metadata2.strategy_used {
        similarity += 0.2;
    }

    // Quality similarity (0.3 weight)
    let quality_diff = (metadata1.quality_score - metadata2.quality_score).abs();
    let quality_similarity = 1.0 - quality_diff.min(1.0);
    similarity += quality_similarity * 0.3;

    // Cluster count similarity (0.2 weight)
    let cluster_diff =
        (metadata1.cluster_count as i32 - metadata2.cluster_count as i32).abs() as f32;
    let cluster_similarity = 1.0 - (cluster_diff / 10.0).min(1.0); // Normalize by 10
    similarity += cluster_similarity * 0.2;

    similarity.clamp(0.0, 1.0)
}

/// Save video progress tracking information
pub fn save_video_progress(db: &Database, progress: &crate::types::VideoProgressUpdate) -> Result<(), DatabaseError> {
    let conn = db.get_conn()
        .map_err(|e| DatabaseError::ConnectionFailed { message: e.to_string() })?;
    
    conn.execute(
        "INSERT OR REPLACE INTO video_progress (plan_id, session_index, video_index, completed, updated_at) 
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            progress.plan_id.to_string(),
            progress.session_index as i64,
            progress.video_index as i64,
            progress.completed,
            chrono::Utc::now().to_rfc3339()
        ],
    ).map_err(|e| DatabaseError::QueryFailed { 
        query: "INSERT OR REPLACE INTO video_progress".to_string(), 
        message: e.to_string() 
    })?;
    
    Ok(())
}

/// Get video completion status
pub fn get_video_completion_status(db: &Database, plan_id: &uuid::Uuid, session_index: usize, video_index: usize) -> Result<bool, DatabaseError> {
    let conn = db.get_conn()
        .map_err(|e| DatabaseError::ConnectionFailed { message: e.to_string() })?;
    
    let mut stmt = conn.prepare(
        "SELECT completed FROM video_progress WHERE plan_id = ?1 AND session_index = ?2 AND video_index = ?3"
    ).map_err(|e| DatabaseError::QueryFailed { 
        query: "SELECT completed FROM video_progress".to_string(), 
        message: e.to_string() 
    })?;
    
    let result = stmt.query_row(
        params![plan_id.to_string(), session_index as i64, video_index as i64],
        |row| Ok(row.get::<_, bool>(0)?)
    );
    
    match result {
        Ok(completed) => Ok(completed),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false), // Default to not completed
        Err(e) => Err(DatabaseError::QueryFailed { 
            query: "SELECT completed FROM video_progress".to_string(), 
            message: e.to_string() 
        }),
    }
}

/// Get session progress as percentage (0.0 to 1.0)
pub fn get_session_progress(db: &Database, plan_id: &uuid::Uuid, session_index: usize) -> Result<f32, DatabaseError> {
    let conn = db.get_conn()
        .map_err(|e| DatabaseError::ConnectionFailed { message: e.to_string() })?;
    
    // Get total videos in the session and completed count
    let mut stmt = conn.prepare(
        "SELECT 
            COUNT(*) as total_videos,
            SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END) as completed_videos
         FROM video_progress 
         WHERE plan_id = ?1 AND session_index = ?2"
    ).map_err(|e| DatabaseError::QueryFailed { 
        query: "SELECT COUNT(*) FROM video_progress".to_string(), 
        message: e.to_string() 
    })?;
    
    let result = stmt.query_row(
        params![plan_id.to_string(), session_index as i64],
        |row| {
            let total: i64 = row.get(0)?;
            let completed: i64 = row.get(1)?;
            Ok((total, completed))
        }
    );
    
    match result {
        Ok((total, completed)) => {
            if total > 0 {
                Ok(completed as f32 / total as f32)
            } else {
                Ok(0.0)
            }
        }
        Err(e) => Err(DatabaseError::QueryFailed { 
            query: "SELECT COUNT(*) FROM video_progress".to_string(), 
            message: e.to_string() 
        }),
    }
}
