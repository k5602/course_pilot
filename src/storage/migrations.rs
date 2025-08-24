//! Database migration system for Course Pilot
//!
//! This module provides a comprehensive migration framework for managing
//! database schema changes, data transformations, and rollback capabilities.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use rusqlite::{Connection, OptionalExtension, params};
use std::collections::HashMap;

/// Current database schema version
pub const CURRENT_SCHEMA_VERSION: i32 = 4;

/// Migration manager for handling database schema changes
pub struct MigrationManager {
    migrations: HashMap<i32, Box<dyn Migration>>,
}

impl MigrationManager {
    /// Create a new migration manager with all available migrations
    pub fn new() -> Self {
        let mut manager = Self {
            migrations: HashMap::new(),
        };

        // Register all migrations
        manager.register_migration(1, Box::new(InitialVersionTracking));
        manager.register_migration(2, Box::new(VideoMetadataEnhancement));
        manager.register_migration(3, Box::new(PerformanceIndexes));
        manager.register_migration(4, Box::new(VideoProgressTracking));

        manager
    }

    /// Register a migration for a specific version
    fn register_migration(&mut self, version: i32, migration: Box<dyn Migration>) {
        self.migrations.insert(version, migration);
    }

    /// Run all pending migrations
    pub fn migrate(&self, conn: &mut Connection) -> Result<()> {
        // Ensure migration tracking table exists
        self.ensure_migration_table(conn)?;

        let current_version = self.get_current_version(conn)?;
        info!(
            "Current database version: {}, target version: {}",
            current_version, CURRENT_SCHEMA_VERSION
        );

        if current_version >= CURRENT_SCHEMA_VERSION {
            info!("Database is up to date, no migrations needed");
            return Ok(());
        }

        // Run migrations in sequence
        for version in (current_version + 1)..=CURRENT_SCHEMA_VERSION {
            if let Some(migration) = self.migrations.get(&version) {
                info!(
                    "Applying migration to version {}: {}",
                    version,
                    migration.name()
                );

                // Create a savepoint for rollback capability
                let savepoint_name = format!("migration_v{}", version);
                conn.execute(&format!("SAVEPOINT {}", savepoint_name), [])?;

                match self.apply_migration(conn, migration.as_ref(), version) {
                    Ok(_) => {
                        // Migration successful, release savepoint
                        conn.execute(&format!("RELEASE {}", savepoint_name), [])?;
                        self.record_migration(conn, version, migration.name())?;
                        info!("Successfully applied migration to version {}", version);
                    }
                    Err(e) => {
                        // Migration failed, rollback to savepoint
                        error!("Migration to version {} failed: {}", version, e);
                        conn.execute(&format!("ROLLBACK TO {}", savepoint_name), [])?;
                        conn.execute(&format!("RELEASE {}", savepoint_name), [])?;
                        return Err(e).with_context(|| {
                            format!("Failed to apply migration to version {}", version)
                        });
                    }
                }
            } else {
                warn!("No migration found for version {}", version);
            }
        }

        info!("All database migrations completed successfully");
        Ok(())
    }

    /// Rollback to a specific version (if supported)
    pub fn rollback_to_version(&self, conn: &mut Connection, target_version: i32) -> Result<()> {
        let current_version = self.get_current_version(conn)?;

        if target_version >= current_version {
            return Err(anyhow::anyhow!(
                "Cannot rollback to version {} from version {}",
                target_version,
                current_version
            ));
        }

        info!(
            "Rolling back from version {} to version {}",
            current_version, target_version
        );

        // Rollback migrations in reverse order
        for version in ((target_version + 1)..=current_version).rev() {
            if let Some(migration) = self.migrations.get(&version) {
                if migration.supports_rollback() {
                    info!(
                        "Rolling back migration version {}: {}",
                        version,
                        migration.name()
                    );

                    let savepoint_name = format!("rollback_v{}", version);
                    conn.execute(&format!("SAVEPOINT {}", savepoint_name), [])?;

                    match migration.rollback(conn) {
                        Ok(_) => {
                            conn.execute(&format!("RELEASE {}", savepoint_name), [])?;
                            self.remove_migration_record(conn, version)?;
                            info!("Successfully rolled back migration version {}", version);
                        }
                        Err(e) => {
                            error!("Rollback of version {} failed: {}", version, e);
                            conn.execute(&format!("ROLLBACK TO {}", savepoint_name), [])?;
                            conn.execute(&format!("RELEASE {}", savepoint_name), [])?;
                            return Err(e).with_context(|| {
                                format!("Failed to rollback migration version {}", version)
                            });
                        }
                    }
                } else {
                    warn!("Migration version {} does not support rollback", version);
                    return Err(anyhow::anyhow!(
                        "Migration version {} does not support rollback",
                        version
                    ));
                }
            }
        }

        info!(
            "Rollback to version {} completed successfully",
            target_version
        );
        Ok(())
    }

    /// Get list of applied migrations
    pub fn get_migration_history(&self, conn: &Connection) -> Result<Vec<MigrationRecord>> {
        let mut stmt = conn.prepare(
            "SELECT version, name, applied_at, checksum FROM migration_history ORDER BY version",
        )?;

        let migration_iter = stmt.query_map([], |row| {
            Ok(MigrationRecord {
                version: row.get(0)?,
                name: row.get(1)?,
                applied_at: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .map_err(|e| {
                        rusqlite::Error::FromSqlConversionFailure(
                            2,
                            rusqlite::types::Type::Text,
                            Box::new(e),
                        )
                    })?
                    .with_timezone(&Utc),
                checksum: row.get(3)?,
            })
        })?;

        let mut records = Vec::new();
        for record in migration_iter {
            records.push(record?);
        }

        Ok(records)
    }

    /// Validate database integrity after migrations
    pub fn validate_database(&self, conn: &Connection) -> Result<ValidationReport> {
        let mut report = ValidationReport {
            is_valid: true,
            issues: Vec::new(),
            warnings: Vec::new(),
        };

        // Check database integrity
        let integrity_check: String =
            conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity_check != "ok" {
            report.is_valid = false;
            report.issues.push(format!(
                "Database integrity check failed: {}",
                integrity_check
            ));
        }

        // Check foreign key constraints
        let fk_check: Vec<String> = {
            let mut stmt = conn.prepare("PRAGMA foreign_key_check")?;
            let rows = stmt.query_map([], |row| {
                Ok(format!(
                    "Foreign key violation in table {}: {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(3)?
                ))
            })?;

            let mut violations = Vec::new();
            for row in rows {
                violations.push(row?);
            }
            violations
        };

        if !fk_check.is_empty() {
            report.is_valid = false;
            report.issues.extend(fk_check);
        }

        // Check for missing indexes on foreign keys
        let missing_indexes = self.check_missing_foreign_key_indexes(conn)?;
        if !missing_indexes.is_empty() {
            report.warnings.extend(missing_indexes);
        }

        // Check table statistics
        let stats_warnings = self.check_table_statistics(conn)?;
        report.warnings.extend(stats_warnings);

        Ok(report)
    }

    /// Ensure the migration tracking table exists
    fn ensure_migration_table(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            r#"
            CREATE TABLE IF NOT EXISTS migration_history (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at TEXT NOT NULL,
                checksum TEXT NOT NULL
            );
            "#,
            [],
        )?;
        Ok(())
    }

    /// Get the current database version
    fn get_current_version(&self, conn: &Connection) -> Result<i32> {
        let version = conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM migration_history",
                [],
                |row| row.get::<_, i32>(0),
            )
            .unwrap_or(0);

        Ok(version)
    }

    /// Apply a single migration
    fn apply_migration(
        &self,
        conn: &Connection,
        migration: &dyn Migration,
        _version: i32,
    ) -> Result<()> {
        // Validate migration before applying
        migration.validate(conn)?;

        // Apply the migration
        migration.apply(conn)?;

        // Verify migration was applied correctly
        migration.verify(conn)?;

        Ok(())
    }

    /// Record a successful migration
    fn record_migration(&self, conn: &Connection, version: i32, name: &str) -> Result<()> {
        let checksum = self.calculate_migration_checksum(version, name);
        conn.execute(
            "INSERT OR REPLACE INTO migration_history (version, name, applied_at, checksum) VALUES (?1, ?2, ?3, ?4)",
            params![version, name, Utc::now().to_rfc3339(), checksum],
        )?;
        Ok(())
    }

    /// Remove a migration record (for rollbacks)
    fn remove_migration_record(&self, conn: &Connection, version: i32) -> Result<()> {
        conn.execute(
            "DELETE FROM migration_history WHERE version = ?1",
            params![version],
        )?;
        Ok(())
    }

    /// Calculate a checksum for migration verification
    fn calculate_migration_checksum(&self, version: i32, name: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        version.hash(&mut hasher);
        name.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Check for missing indexes on foreign key columns
    fn check_missing_foreign_key_indexes(&self, conn: &Connection) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // This is a simplified check - in a real implementation, you'd parse
        // the schema to find all foreign key relationships
        let foreign_keys = vec![
            ("plans", "course_id"),
            ("notes", "course_id"),
            ("notes", "video_id"),
        ];

        for (table, column) in foreign_keys {
            let index_exists: bool = conn.query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'index' AND tbl_name = ?1 AND sql LIKE ?2",
                params![table, format!("%{}%", column)],
                |row| row.get(0),
            ).unwrap_or(false);

            if !index_exists {
                warnings.push(format!("Missing index on foreign key {}.{}", table, column));
            }
        }

        Ok(warnings)
    }

    /// Check table statistics and suggest optimizations
    fn check_table_statistics(&self, conn: &Connection) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check if ANALYZE has been run recently
        let analyze_info: Option<String> = conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE name = 'sqlite_stat1'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if analyze_info.is_none() {
            warnings.push(
                "Database statistics are missing - run ANALYZE for better query performance"
                    .to_string(),
            );
        }

        // Check for large tables without proper indexes
        let large_tables: Vec<(String, i64)> = {
            let mut stmt = conn.prepare(
                "SELECT name, (SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND tbl_name = m.name) as index_count
                 FROM sqlite_master m WHERE type = 'table' AND name NOT LIKE 'sqlite_%'"
            )?;

            let rows = stmt.query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?;

            let mut tables = Vec::new();
            for row in rows {
                tables.push(row?);
            }
            tables
        };

        for (table_name, index_count) in large_tables {
            if index_count < 2 {
                // Assuming at least primary key + one other index
                warnings.push(format!(
                    "Table '{}' has only {} indexes - consider adding more for better performance",
                    table_name, index_count
                ));
            }
        }

        Ok(warnings)
    }
}

/// Trait for database migrations
pub trait Migration: Send + Sync {
    /// Get the migration name
    fn name(&self) -> &str;

    /// Validate that the migration can be applied
    fn validate(&self, conn: &Connection) -> Result<()>;

    /// Apply the migration
    fn apply(&self, conn: &Connection) -> Result<()>;

    /// Verify the migration was applied correctly
    fn verify(&self, conn: &Connection) -> Result<()>;

    /// Whether this migration supports rollback
    fn supports_rollback(&self) -> bool {
        false
    }

    /// Rollback the migration (if supported)
    fn rollback(&self, _conn: &Connection) -> Result<()> {
        Err(anyhow::anyhow!("Rollback not supported for this migration"))
    }
}

/// Migration record for tracking applied migrations
#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub version: i32,
    pub name: String,
    pub applied_at: DateTime<Utc>,
    pub checksum: String,
}

/// Database validation report
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub is_valid: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

// Concrete migration implementations

/// Migration 1: Initial version tracking setup
struct InitialVersionTracking;

impl Migration for InitialVersionTracking {
    fn name(&self) -> &str {
        "Initial version tracking setup"
    }

    fn validate(&self, _conn: &Connection) -> Result<()> {
        // Always valid - this is the initial migration
        Ok(())
    }

    fn apply(&self, _conn: &Connection) -> Result<()> {
        // This migration just establishes version tracking
        info!("Setting up initial version tracking");
        Ok(())
    }

    fn verify(&self, conn: &Connection) -> Result<()> {
        // Verify migration_history table exists
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'migration_history'",
            [],
            |row| row.get(0),
        )?;

        if !table_exists {
            return Err(anyhow::anyhow!("migration_history table was not created"));
        }

        // Ensure baseline schema exists for subsequent migrations on a fresh DB
        // Create minimal tables required by later migrations and tests.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS courses (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                raw_titles TEXT NOT NULL,
                videos TEXT,
                structure TEXT
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS plans (
                id TEXT PRIMARY KEY,
                course_id TEXT NOT NULL,
                settings TEXT NOT NULL,
                items TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                course_id TEXT NOT NULL,
                video_id INTEGER,
                content TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(())
    }
}

/// Migration 2: VideoMetadata enhancement with playlist_id and original_index
struct VideoMetadataEnhancement;

impl Migration for VideoMetadataEnhancement {
    fn name(&self) -> &str {
        "VideoMetadata enhancement with playlist_id and original_index"
    }

    fn validate(&self, conn: &Connection) -> Result<()> {
        // Check if courses table exists
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type = 'table' AND name = 'courses'",
            [],
            |row| row.get(0),
        )?;

        if !table_exists {
            return Err(anyhow::anyhow!("courses table does not exist"));
        }

        Ok(())
    }

    fn apply(&self, conn: &Connection) -> Result<()> {
        info!("Updating VideoMetadata to include playlist_id and original_index");

        // Get all courses that need migration
        let mut stmt =
            conn.prepare("SELECT id, name, videos FROM courses WHERE videos IS NOT NULL")?;

        let courses_to_migrate: Vec<(String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?, // id
                    row.get::<_, String>(1)?, // name
                    row.get::<_, String>(2)?, // videos JSON
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        info!("Found {} courses to migrate", courses_to_migrate.len());

        for (course_id, course_name, videos_json) in courses_to_migrate {
            info!("Migrating course: {} ({})", course_name, course_id);

            // Parse existing video metadata
            let existing_videos: Vec<serde_json::Value> = match serde_json::from_str(&videos_json) {
                Ok(videos) => videos,
                Err(e) => {
                    warn!(
                        "Failed to parse video metadata for course {}: {}",
                        course_name, e
                    );
                    continue;
                }
            };

            // Migrate each video metadata
            let mut migrated_videos = Vec::new();
            for (index, mut video) in existing_videos.into_iter().enumerate() {
                // Add playlist_id field if missing
                if !video.as_object().unwrap().contains_key("playlist_id") {
                    video["playlist_id"] = serde_json::Value::Null;
                }

                // Add original_index field if missing
                if !video.as_object().unwrap().contains_key("original_index") {
                    video["original_index"] = serde_json::Value::from(index);
                }

                migrated_videos.push(video);
            }

            // Serialize updated video metadata
            let updated_videos_json = serde_json::to_string(&migrated_videos)?;

            // Update the course in the database
            conn.execute(
                "UPDATE courses SET videos = ?1 WHERE id = ?2",
                params![updated_videos_json, course_id],
            )?;

            info!("Successfully migrated course: {}", course_name);
        }

        Ok(())
    }

    fn verify(&self, conn: &Connection) -> Result<()> {
        // Verify that at least one course has the new fields
        let sample_course: Option<String> = conn
            .query_row(
                "SELECT videos FROM courses WHERE videos IS NOT NULL LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(videos_json) = sample_course {
            let videos: Vec<serde_json::Value> = serde_json::from_str(&videos_json)?;
            if let Some(first_video) = videos.first() {
                if !first_video
                    .as_object()
                    .unwrap()
                    .contains_key("original_index")
                {
                    return Err(anyhow::anyhow!(
                        "Migration verification failed: original_index field missing"
                    ));
                }
            }
        }

        Ok(())
    }
}

/// Migration 3: Add performance indexes
struct PerformanceIndexes;

impl Migration for PerformanceIndexes {
    fn name(&self) -> &str {
        "Add performance indexes for frequently queried columns"
    }

    fn validate(&self, _conn: &Connection) -> Result<()> {
        // Always valid - adding indexes is safe
        Ok(())
    }

    fn apply(&self, conn: &Connection) -> Result<()> {
        info!("Adding performance indexes");

        // Add indexes for better search performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_courses_name_search ON courses(name COLLATE NOCASE);",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_notes_content_search_improved ON notes(content) WHERE length(content) > 3;",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_plans_course_created_covering ON plans(course_id, created_at, settings);",
            [],
        )?;

        info!("Performance indexes added successfully");
        Ok(())
    }

    fn verify(&self, conn: &Connection) -> Result<()> {
        // Verify that the indexes were created
        let index_count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_%_search%'",
            [],
            |row| row.get(0),
        )?;

        if index_count < 2 {
            return Err(anyhow::anyhow!(
                "Migration verification failed: expected indexes not found"
            ));
        }

        Ok(())
    }

    fn supports_rollback(&self) -> bool {
        true
    }

    fn rollback(&self, conn: &Connection) -> Result<()> {
        info!("Rolling back performance indexes");

        conn.execute("DROP INDEX IF EXISTS idx_courses_name_search;", [])?;
        conn.execute(
            "DROP INDEX IF EXISTS idx_notes_content_search_improved;",
            [],
        )?;
        conn.execute(
            "DROP INDEX IF EXISTS idx_plans_course_created_covering;",
            [],
        )?;

        info!("Performance indexes rolled back successfully");
        Ok(())
    }
}

/// Migration 4: Add video progress tracking
struct VideoProgressTracking;

impl Migration for VideoProgressTracking {
    fn name(&self) -> &str {
        "video_progress_tracking"
    }

    fn validate(&self, _conn: &Connection) -> Result<()> {
        // No specific validation needed for this migration
        Ok(())
    }

    fn apply(&self, conn: &Connection) -> Result<()> {
        info!("Creating video progress tracking table...");

        // Create video_progress table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS video_progress (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                plan_id TEXT NOT NULL,
                session_index INTEGER NOT NULL,
                video_index INTEGER NOT NULL,
                completed BOOLEAN NOT NULL DEFAULT 0,
                updated_at TEXT NOT NULL,
                UNIQUE(plan_id, session_index, video_index)
            )",
            [],
        )
        .context("Failed to create video_progress table")?;

        // Create index for efficient lookups
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_video_progress_plan_session
             ON video_progress(plan_id, session_index)",
            [],
        )
        .context("Failed to create video progress index")?;

        info!("Video progress tracking migration completed successfully");
        Ok(())
    }

    fn verify(&self, conn: &Connection) -> Result<()> {
        // Verify table exists
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='table' AND name='video_progress'",
        )?;

        let table_exists = stmt.query_row([], |_| Ok(())).is_ok();

        if !table_exists {
            return Err(anyhow::anyhow!("video_progress table was not created"));
        }

        // Verify index exists
        let mut stmt = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_video_progress_plan_session'"
        )?;

        let index_exists = stmt.query_row([], |_| Ok(())).is_ok();

        if !index_exists {
            return Err(anyhow::anyhow!("video_progress index was not created"));
        }

        info!("Video progress tracking migration verification completed successfully");
        Ok(())
    }

    fn supports_rollback(&self) -> bool {
        true
    }

    fn rollback(&self, conn: &Connection) -> Result<()> {
        info!("Rolling back video progress tracking");
        // Drop index first, then table
        conn.execute("DROP INDEX IF EXISTS idx_video_progress_plan_session", [])?;
        conn.execute("DROP TABLE IF EXISTS video_progress", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_migration_manager() {
        let mut conn = Connection::open_in_memory().unwrap();
        let manager = MigrationManager::new();

        // Test migration
        manager.migrate(&mut conn).unwrap();

        // Verify current version
        let version = manager.get_current_version(&conn).unwrap();
        assert_eq!(version, CURRENT_SCHEMA_VERSION);

        // Test migration history
        let history = manager.get_migration_history(&conn).unwrap();
        assert_eq!(history.len(), CURRENT_SCHEMA_VERSION as usize);

        // Test validation
        let report = manager.validate_database(&conn).unwrap();
        assert!(report.is_valid);
    }

    #[test]
    fn test_rollback() {
        let mut conn = Connection::open_in_memory().unwrap();
        let manager = MigrationManager::new();

        // Apply all migrations
        manager.migrate(&mut conn).unwrap();

        // Test rollback (only version 3 supports rollback)
        manager.rollback_to_version(&mut conn, 2).unwrap();

        let version = manager.get_current_version(&conn).unwrap();
        assert_eq!(version, 2);
    }
}
