//! Database operations for Course Pilot
//!
//! This module provides SQLite-based persistence for courses and study plans
//! using JSON serialization for complex data structures.
//! It uses connection pooling for better performance under load.

use crate::DatabaseError;
use crate::types::{Course, Plan};
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
    pub fn new(db_path: &Path) -> Result<Self, DatabaseError> {
        info!("Initializing database at: {}", db_path.display());

        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                info!("Creating database directory: {}", parent.display());
                std::fs::create_dir_all(parent).map_err(|e| {
                    error!(
                        "Failed to create database directory {}: {}",
                        parent.display(),
                        e
                    );
                    DatabaseError::Io(e)
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
            .build(manager)?;

        // Initialize database schema
        let mut conn = pool.get()?;
        init_tables(&mut conn)?;

        Ok(Database {
            pool: Arc::new(pool),
        })
    }

    /// Get a connection from the pool
    pub fn get_conn(&self) -> Result<PooledConnection, DatabaseError> {
        self.pool.get().map_err(Into::into)
    }

    /// Get a reference to the underlying pool
    pub fn pool(&self) -> &DbPool {
        &self.pool
    }

    /// Clean up WAL and SHM files from previous runs
    fn cleanup_wal_files(db_path: &Path) -> Result<(), DatabaseError> {
        let wal_path = db_path.with_extension("db-wal");
        let shm_path = db_path.with_extension("db-shm");

        // Remove WAL file if it exists
        if wal_path.exists() {
            std::fs::remove_file(&wal_path).map_err(|e| {
                log::warn!("Failed to remove WAL file {}: {}", wal_path.display(), e);
                DatabaseError::Io(e)
            })?;
            log::info!("Removed existing WAL file: {}", wal_path.display());
        }

        // Remove SHM file if it exists
        if shm_path.exists() {
            std::fs::remove_file(&shm_path).map_err(|e| {
                log::warn!("Failed to remove SHM file {}: {}", shm_path.display(), e);
                DatabaseError::Io(e)
            })?;
            log::info!("Removed existing SHM file: {}", shm_path.display());
        }

        Ok(())
    }
}

/// Type alias for a pooled connection
type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

/// Initialize database tables
fn init_tables(conn: &mut Connection) -> Result<(), DatabaseError> {
    let tx = conn.transaction()?;

    // Create courses table
    tx.execute(
        r#"
        CREATE TABLE IF NOT EXISTS courses (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            raw_titles TEXT NOT NULL,
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
    crate::storage::notes::init_notes_table(conn).map_err(|e| {
        DatabaseError::Io(std::io::Error::other(format!(
            "Notes table initialization failed: {e}"
        )))
    })?;

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
pub fn optimize_database(db: &Database) -> Result<(), DatabaseError> {
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
    let integrity_check: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
    if integrity_check != "ok" {
        warn!("Database integrity check failed: {integrity_check}");
        return Err(DatabaseError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Database integrity check failed: {integrity_check}"),
        )));
    }

    info!("Database optimization completed successfully");
    Ok(())
}

/// Get database performance metrics
pub fn get_database_performance_metrics(
    db: &Database,
) -> Result<DatabasePerformanceMetrics, DatabaseError> {
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

/// Initialize the database and create necessary tables
pub fn init_db(db_path: &Path) -> Result<Database, DatabaseError> {
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
pub fn save_course(db: &Database, course: &Course) -> Result<(), DatabaseError> {
    info!("Saving course: {} (ID: {})", course.name, course.id);

    let raw_titles_json = serde_json::to_string(&course.raw_titles).map_err(|e| {
        error!(
            "Failed to serialize raw_titles for course {}: {}",
            course.name, e
        );
        DatabaseError::Serialization(e)
    })?;

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
        INSERT OR REPLACE INTO courses (id, name, created_at, raw_titles, structure)
        VALUES (?1, ?2, ?3, ?4, ?5)
        "#,
        params![
            course.id.to_string(),
            course.name,
            course.created_at.timestamp(),
            raw_titles_json,
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
pub fn load_courses(db: &Database) -> Result<Vec<Course>, DatabaseError> {
    info!("Loading all courses from database");

    let conn = db.get_conn().map_err(|e| {
        error!("Failed to get database connection for loading courses: {e}");
        e
    })?;

    // Use index on created_at for efficient ordering
    let mut stmt = conn
        .prepare(
            r#"
        SELECT id, name, created_at, raw_titles, structure
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
            let structure_json: Option<String> = row.get(4)?;

            let raw_titles: Vec<String> = serde_json::from_str(&raw_titles_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let structure = structure_json
                .map(|json| {
                    serde_json::from_str(&json).map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            4,
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
                structure,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(courses)
}

/// Get a specific course by ID
pub fn get_course_by_id(db: &Database, course_id: &Uuid) -> Result<Option<Course>, DatabaseError> {
    let conn = db.get_conn()?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, created_at, raw_titles, structure
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
            let structure_json: Option<String> = row.get(4)?;

            let raw_titles = serde_json::from_str(&raw_titles_json).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let structure = structure_json
                .map(|json| serde_json::from_str(&json))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(Course {
                id,
                name,
                created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
                raw_titles,
                structure,
            })
        })
        .optional()?;

    Ok(course)
}

/// Save a plan to the database
pub fn save_plan(db: &Database, plan: &Plan) -> Result<(), DatabaseError> {
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
) -> Result<Option<Plan>, DatabaseError> {
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
pub fn load_plan(db: &Database, plan_id: &Uuid) -> Result<Option<Plan>, DatabaseError> {
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

    result
}

/// Delete a course from the database
pub fn delete_course(db: &Database, course_id: &Uuid) -> Result<(), DatabaseError> {
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
pub fn delete_plan(db: &Database, plan_id: &Uuid) -> Result<(), DatabaseError> {
    let conn = db.get_conn()?;
    conn.execute(
        "DELETE FROM plans WHERE id = ?1",
        params![plan_id.to_string()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{CourseStructure, Module, Section, StructureMetadata};
    use chrono::Utc;
    use std::time::Duration;

    #[test]
    fn test_database_operations() {
        // Create and initialize an in-memory database for testing
        let conn = init_db(Path::new(":memory:")).unwrap();

        // Create test course
        let course = Course {
            id: Uuid::new_v4(),
            name: "Test Course".to_string(),
            created_at: Utc::now(),
            raw_titles: vec!["Video 1".to_string(), "Video 2".to_string()],
            structure: Some(CourseStructure::new_basic(
                vec![Module::new_basic(
                    "Module 1".to_string(),
                    vec![Section {
                        title: "Section 1".to_string(),
                        video_index: 0,
                        duration: Duration::from_secs(3600),
                    }],
                )],
                StructureMetadata {
                    total_videos: 2,
                    total_duration: Duration::from_secs(3600),
                    estimated_duration_hours: Some(2.0),
                    difficulty_level: Some("Beginner".to_string()),
                    structure_quality_score: None,
                    content_coherence_score: None,
                },
            )),
        };

        // Test save and load course
        save_course(&conn, &course).unwrap();
        let loaded_courses = load_courses(&conn).unwrap();
        assert_eq!(loaded_courses.len(), 1);
        assert_eq!(loaded_courses[0].name, course.name);
        assert_eq!(loaded_courses[0].raw_titles, course.raw_titles);
    }
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
) -> Result<Vec<Course>, DatabaseError> {
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
                    raw_titles: parse_json_sqlite(&row.get::<_, String>(3)?)?,
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
pub fn get_clustering_analytics(db: &Database) -> Result<ClusteringAnalytics, DatabaseError> {
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
) -> Result<(), DatabaseError> {
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
) -> Result<Vec<Course>, DatabaseError> {
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
                    raw_titles: parse_json_sqlite(&row.get::<_, String>(3)?)?,
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
) -> Result<Vec<ClusteringPerformancePoint>, DatabaseError> {
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
