//! Database operations for Course Pilot
//!
//! This module provides SQLite-based persistence for courses and study plans
//! using JSON serialization for complex data structures.

use crate::DatabaseError;
use crate::types::{Course, Plan};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, params};
use serde_json;
use std::path::Path;
use uuid::Uuid;

/// Initialize the database and create necessary tables
pub fn init_db(db_path: &Path) -> Result<Connection, DatabaseError> {
    let conn = Connection::open(db_path)?;

    // Create courses table
    conn.execute(
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
    conn.execute(
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

    // Create indices for better performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_plans_course_id ON plans(course_id);",
        [],
    )?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_courses_created_at ON courses(created_at);",
        [],
    )?;

    Ok(conn)
}

/// Save a course to the database
pub fn save_course(conn: &Connection, course: &Course) -> Result<(), DatabaseError> {
    let raw_titles_json = serde_json::to_string(&course.raw_titles)?;
    let structure_json = course
        .structure
        .as_ref()
        .map(|s| serde_json::to_string(s))
        .transpose()?;

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
pub fn load_courses(conn: &Connection) -> Result<Vec<Course>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, created_at, raw_titles, structure FROM courses ORDER BY created_at DESC",
    )?;

    let course_iter = stmt.query_map([], |row| {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text)
        })?;

        let name: String = row.get(1)?;
        let timestamp: i64 = row.get(2)?;
        let created_at = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);

        let raw_titles_json: String = row.get(3)?;
        let raw_titles: Vec<String> = serde_json::from_str(&raw_titles_json).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(
                3,
                "raw_titles".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

        let structure_json: Option<String> = row.get(4)?;
        let structure = structure_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    "structure".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

        Ok(Course {
            id,
            name,
            created_at,
            raw_titles,
            structure,
        })
    })?;

    let mut courses = Vec::new();
    for course in course_iter {
        courses.push(course?);
    }

    Ok(courses)
}

/// Get a specific course by ID
pub fn get_course_by_id(
    conn: &Connection,
    course_id: &Uuid,
) -> Result<Option<Course>, DatabaseError> {
    let mut stmt = conn
        .prepare("SELECT id, name, created_at, raw_titles, structure FROM courses WHERE id = ?1")?;

    let mut course_iter = stmt.query_map([course_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text)
        })?;

        let name: String = row.get(1)?;
        let timestamp: i64 = row.get(2)?;
        let created_at = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);

        let raw_titles_json: String = row.get(3)?;
        let raw_titles: Vec<String> = serde_json::from_str(&raw_titles_json).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(
                3,
                "raw_titles".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

        let structure_json: Option<String> = row.get(4)?;
        let structure = structure_json
            .map(|json| serde_json::from_str(&json))
            .transpose()
            .map_err(|_e| {
                rusqlite::Error::InvalidColumnType(
                    4,
                    "structure".to_string(),
                    rusqlite::types::Type::Text,
                )
            })?;

        Ok(Course {
            id,
            name,
            created_at,
            raw_titles,
            structure,
        })
    })?;

    match course_iter.next() {
        Some(course) => Ok(Some(course?)),
        None => Ok(None),
    }
}

/// Save a plan to the database
pub fn save_plan(conn: &Connection, plan: &Plan) -> Result<(), DatabaseError> {
    let settings_json = serde_json::to_string(&plan.settings)?;
    let items_json = serde_json::to_string(&plan.items)?;

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
            plan.created_at.timestamp()
        ],
    )?;

    Ok(())
}

/// Load a plan by course ID
pub fn get_plan_by_course_id(
    conn: &Connection,
    course_id: &Uuid,
) -> Result<Option<Plan>, DatabaseError> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, settings, items, created_at FROM plans WHERE course_id = ?1 ORDER BY created_at DESC LIMIT 1",
    )?;

    let mut plan_iter = stmt.query_map([course_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text)
        })?;

        let course_id_str: String = row.get(1)?;
        let course_id = Uuid::parse_str(&course_id_str).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(
                1,
                "course_id".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

        let settings_json: String = row.get(2)?;
        let settings = serde_json::from_str(&settings_json).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(
                2,
                "settings".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

        let items_json: String = row.get(3)?;
        let items = serde_json::from_str(&items_json).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(3, "items".to_string(), rusqlite::types::Type::Text)
        })?;

        let timestamp: i64 = row.get(4)?;
        let created_at = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);

        Ok(Plan {
            id,
            course_id,
            settings,
            items,
            created_at,
        })
    })?;

    match plan_iter.next() {
        Some(plan) => Ok(Some(plan?)),
        None => Ok(None),
    }
}

/// Load a specific plan by ID
pub fn load_plan(conn: &Connection, plan_id: &Uuid) -> Result<Option<Plan>, DatabaseError> {
    let mut stmt =
        conn.prepare("SELECT id, course_id, settings, items, created_at FROM plans WHERE id = ?1")?;

    let mut plan_iter = stmt.query_map([plan_id.to_string()], |row| {
        let id_str: String = row.get(0)?;
        let id = Uuid::parse_str(&id_str).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(0, "id".to_string(), rusqlite::types::Type::Text)
        })?;

        let course_id_str: String = row.get(1)?;
        let course_id = Uuid::parse_str(&course_id_str).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(
                1,
                "course_id".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

        let settings_json: String = row.get(2)?;
        let settings = serde_json::from_str(&settings_json).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(
                2,
                "settings".to_string(),
                rusqlite::types::Type::Text,
            )
        })?;

        let items_json: String = row.get(3)?;
        let items = serde_json::from_str(&items_json).map_err(|_e| {
            rusqlite::Error::InvalidColumnType(3, "items".to_string(), rusqlite::types::Type::Text)
        })?;

        let timestamp: i64 = row.get(4)?;
        let created_at = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(Utc::now);

        Ok(Plan {
            id,
            course_id,
            settings,
            items,
            created_at,
        })
    })?;

    match plan_iter.next() {
        Some(plan) => Ok(Some(plan?)),
        None => Ok(None),
    }
}

/// Delete a course from the database
pub fn delete_course(conn: &Connection, course_id: &Uuid) -> Result<(), DatabaseError> {
    // Delete associated plans first (due to foreign key constraint)
    conn.execute(
        "DELETE FROM plans WHERE course_id = ?1",
        [course_id.to_string()],
    )?;

    // Delete the course
    conn.execute("DELETE FROM courses WHERE id = ?1", [course_id.to_string()])?;

    Ok(())
}

/// Delete a plan from the database
pub fn delete_plan(conn: &Connection, plan_id: &Uuid) -> Result<(), DatabaseError> {
    conn.execute("DELETE FROM plans WHERE id = ?1", [plan_id.to_string()])?;

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
