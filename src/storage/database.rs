//! Database operations for Course Pilot
//!
//! This module provides SQLite-based persistence for courses and study plans
//! using JSON serialization for complex data structures.
//! It uses connection pooling for better performance under load.

use crate::DatabaseError;
use crate::types::{Course, Plan};
use chrono::{DateTime, Utc};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json;
use std::path::Path;
use std::sync::Arc;
use uuid::Uuid;

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
        // Ensure the parent directory exists
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        // Clean up any existing WAL/SHM files from previous runs
        Self::cleanup_wal_files(db_path)?;

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
        DatabaseError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Notes table initialization failed: {}", e),
        ))
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

    tx.commit()?;
    Ok(())
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

    Ok(db)
}

/// Save a course to the database
pub fn save_course(db: &Database, course: &Course) -> Result<(), DatabaseError> {
    let raw_titles_json = serde_json::to_string(&course.raw_titles)?;
    let structure_json = course
        .structure
        .as_ref()
        .map(serde_json::to_string)
        .transpose()?;

    let conn = db.get_conn()?;

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
    )?;

    Ok(())
}

/// Load all courses from the database
pub fn load_courses(db: &Database) -> Result<Vec<Course>, DatabaseError> {
    let conn = db.get_conn()?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, created_at, raw_titles, structure
        FROM courses
        ORDER BY created_at DESC
        "#,
    )?;

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
            structure: Some(CourseStructure {
                modules: vec![Module {
                    title: "Module 1".to_string(),
                    sections: vec![Section {
                        title: "Section 1".to_string(),
                        video_index: 0,
                        duration: Duration::from_secs(3600),
                    }],
                    total_duration: Duration::from_secs(3600),
                }],
                metadata: StructureMetadata {
                    total_videos: 2,
                    total_duration: Duration::from_secs(3600),
                    estimated_duration_hours: Some(2.0),
                    difficulty_level: Some("Beginner".to_string()),
                },
            }),
        };

        // Test save and load course
        save_course(&conn, &course).unwrap();
        let loaded_courses = load_courses(&conn).unwrap();
        assert_eq!(loaded_courses.len(), 1);
        assert_eq!(loaded_courses[0].name, course.name);
        assert_eq!(loaded_courses[0].raw_titles, course.raw_titles);
    }
}
