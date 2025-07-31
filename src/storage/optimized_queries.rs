//! Optimized database queries for frequently used operations
//!
//! This module contains performance-optimized queries that use prepared statements,
//! proper indexing, and efficient query patterns for common database operations.

use crate::DatabaseError;
use crate::storage::Database;
use crate::types::{Course, Note, Plan};
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::params;
use std::collections::HashMap;
use uuid::Uuid;

/// Optimized query manager for frequently used database operations
pub struct OptimizedQueries {
    db: Database,
}

impl OptimizedQueries {
    /// Create a new optimized queries instance
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    /// Get courses with their latest plans in a single optimized query
    pub fn get_courses_with_latest_plans(
        &self,
    ) -> Result<Vec<(Course, Option<Plan>)>, DatabaseError> {
        let conn = self.db.get_conn()?;

        // Use a LEFT JOIN to get courses with their most recent plans
        let mut stmt = conn.prepare(
            r#"
            SELECT 
                c.id, c.name, c.created_at, c.raw_titles, c.structure,
                p.id as plan_id, p.settings, p.items, p.created_at as plan_created_at
            FROM courses c
            LEFT JOIN (
                SELECT p1.* FROM plans p1
                INNER JOIN (
                    SELECT course_id, MAX(created_at) as max_created_at
                    FROM plans
                    GROUP BY course_id
                ) p2 ON p1.course_id = p2.course_id AND p1.created_at = p2.max_created_at
            ) p ON c.id = p.course_id
            ORDER BY c.created_at DESC
            "#,
        )?;

        let results = stmt.query_map([], |row| {
            // Parse course data
            let course_id_str: String = row.get(0)?;
            let course_id = Uuid::parse_str(&course_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let course_name: String = row.get(1)?;
            let course_created_at: i64 = row.get(2)?;
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

            let course = Course {
                id: course_id,
                name: course_name,
                created_at: DateTime::from_timestamp(course_created_at, 0).unwrap_or_else(Utc::now),
                raw_titles,
                structure,
            };

            // Parse plan data if present
            let plan = if let Some(plan_id_str) = row.get::<_, Option<String>>(5)? {
                let plan_id = Uuid::parse_str(&plan_id_str).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let settings_json: String = row.get(6)?;
                let items_json: String = row.get(7)?;
                let plan_created_at: i64 = row.get(8)?;

                let settings = serde_json::from_str(&settings_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        6,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                let items = serde_json::from_str(&items_json).map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        7,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

                Some(Plan {
                    id: plan_id,
                    course_id,
                    settings,
                    items,
                    created_at: DateTime::from_timestamp(plan_created_at, 0)
                        .unwrap_or_else(Utc::now),
                })
            } else {
                None
            };

            Ok((course, plan))
        })?;

        let mut course_plan_pairs = Vec::new();
        for result in results {
            course_plan_pairs.push(result?);
        }

        Ok(course_plan_pairs)
    }

    /// Get course statistics in a single optimized query
    pub fn get_course_statistics(&self) -> Result<CourseStatistics, DatabaseError> {
        let conn = self.db.get_conn()?;

        // Get comprehensive statistics in a single query
        let stats = conn.query_row(
            r#"
            SELECT 
                COUNT(*) as total_courses,
                COUNT(CASE WHEN structure IS NOT NULL THEN 1 END) as structured_courses,
                COUNT(CASE WHEN p.course_id IS NOT NULL THEN 1 END) as courses_with_plans,
                AVG(LENGTH(raw_titles)) as avg_raw_titles_length
            FROM courses c
            LEFT JOIN (SELECT DISTINCT course_id FROM plans) p ON c.id = p.course_id
            "#,
            [],
            |row| {
                Ok(CourseStatistics {
                    total_courses: row.get(0)?,
                    structured_courses: row.get(1)?,
                    courses_with_plans: row.get(2)?,
                    avg_raw_titles_length: row.get(3)?,
                })
            },
        )?;

        Ok(stats)
    }

    /// Get notes count by course in a single optimized query
    pub fn get_notes_count_by_course(&self) -> Result<HashMap<Uuid, usize>, DatabaseError> {
        let conn = self.db.get_conn()?;

        let mut stmt =
            conn.prepare("SELECT course_id, COUNT(*) as note_count FROM notes GROUP BY course_id")?;

        let results = stmt.query_map([], |row| {
            let course_id_str: String = row.get(0)?;
            let course_id = Uuid::parse_str(&course_id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let count: usize = row.get(1)?;
            Ok((course_id, count))
        })?;

        let mut counts = HashMap::new();
        for result in results {
            let (course_id, count) = result?;
            counts.insert(course_id, count);
        }

        Ok(counts)
    }

    /// Batch insert notes for better performance
    pub fn batch_insert_notes(&self, notes: &[Note]) -> Result<(), DatabaseError> {
        if notes.is_empty() {
            return Ok(());
        }

        let mut conn = self.db.get_conn()?;
        let tx = conn.transaction()?;

        {
            // Prepare statement once for all inserts
            let mut stmt = tx.prepare(
                r#"
                INSERT INTO notes (id, course_id, video_id, video_index, content, timestamp, created_at, updated_at, tags)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                "#,
            )?;

            for note in notes {
                stmt.execute(params![
                    note.id.to_string(),
                    note.course_id.to_string(),
                    note.video_id.as_ref().map(|v| v.to_string()),
                    note.video_index.map(|i| i as i64),
                    note.content,
                    note.timestamp.map(|t| t as i64),
                    note.created_at.to_rfc3339(),
                    note.updated_at.to_rfc3339(),
                    serde_json::to_string(&note.tags)
                        .map_err(|e| { rusqlite::Error::ToSqlConversionFailure(Box::new(e)) })?,
                ])?;
            }
        } // stmt is dropped here

        tx.commit()?;
        Ok(())
    }

    /// Get recent activity across all entities
    pub fn get_recent_activity(&self, limit: usize) -> Result<Vec<ActivityItem>, DatabaseError> {
        let conn = self.db.get_conn()?;

        let mut stmt = conn.prepare(
            r#"
            SELECT 'course' as type, id, name as title, created_at, NULL as course_id
            FROM courses
            UNION ALL
            SELECT 'plan' as type, id, 'Study Plan' as title, created_at, course_id
            FROM plans
            UNION ALL
            SELECT 'note' as type, id, SUBSTR(content, 1, 50) as title, created_at, course_id
            FROM notes
            ORDER BY created_at DESC
            LIMIT ?1
            "#,
        )?;

        let results = stmt.query_map([limit], |row| {
            let activity_type: String = row.get(0)?;
            let id_str: String = row.get(1)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let title: String = row.get(2)?;
            let created_at_str: String = row.get(3)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        3,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);

            let course_id = row
                .get::<_, Option<String>>(4)?
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(ActivityItem {
                activity_type: match activity_type.as_str() {
                    "course" => ActivityType::Course,
                    "plan" => ActivityType::Plan,
                    "note" => ActivityType::Note,
                    _ => ActivityType::Course, // fallback
                },
                id,
                title,
                created_at,
                course_id,
            })
        })?;

        let mut activities = Vec::new();
        for result in results {
            activities.push(result?);
        }

        Ok(activities)
    }

    /// Optimized search across all content types
    pub fn search_all_content(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, DatabaseError> {
        let conn = self.db.get_conn()?;
        let search_pattern = format!("%{query}%");

        let mut stmt = conn.prepare(
            r#"
            SELECT 'course' as type, id, name as title, name as content, created_at, NULL as course_id
            FROM courses
            WHERE name LIKE ?1 COLLATE NOCASE
            UNION ALL
            SELECT 'note' as type, id, SUBSTR(content, 1, 100) as title, content, created_at, course_id
            FROM notes
            WHERE content LIKE ?1 COLLATE NOCASE
            ORDER BY created_at DESC
            LIMIT ?2
            "#,
        )?;

        let results = stmt.query_map([&search_pattern, &limit.to_string()], |row| {
            let result_type: String = row.get(0)?;
            let id_str: String = row.get(1)?;
            let id = Uuid::parse_str(&id_str).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;
            let title: String = row.get(2)?;
            let content: String = row.get(3)?;
            let created_at_str: String = row.get(4)?;
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        4,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?
                .with_timezone(&Utc);

            let course_id = row
                .get::<_, Option<String>>(5)?
                .map(|s| Uuid::parse_str(&s))
                .transpose()
                .map_err(|e| {
                    rusqlite::Error::FromSqlConversionFailure(
                        5,
                        rusqlite::types::Type::Text,
                        Box::new(e),
                    )
                })?;

            Ok(SearchResult {
                result_type: match result_type.as_str() {
                    "course" => SearchResultType::Course,
                    "note" => SearchResultType::Note,
                    _ => SearchResultType::Course, // fallback
                },
                id,
                title,
                content,
                created_at,
                course_id,
            })
        })?;

        let mut search_results = Vec::new();
        for result in results {
            search_results.push(result?);
        }

        Ok(search_results)
    }
}

/// Course statistics for dashboard display
#[derive(Debug, Clone)]
pub struct CourseStatistics {
    pub total_courses: usize,
    pub structured_courses: usize,
    pub courses_with_plans: usize,
    pub avg_raw_titles_length: f64,
}

/// Activity item for recent activity display
#[derive(Debug, Clone)]
pub struct ActivityItem {
    pub activity_type: ActivityType,
    pub id: Uuid,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub course_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub enum ActivityType {
    Course,
    Plan,
    Note,
}

/// Search result for unified search
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub result_type: SearchResultType,
    pub id: Uuid,
    pub title: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub course_id: Option<Uuid>,
}

#[derive(Debug, Clone)]
pub enum SearchResultType {
    Course,
    Note,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::init_db;
    use std::path::Path;

    #[test]
    fn test_optimized_queries() {
        let db = init_db(Path::new(":memory:")).unwrap();
        let queries = OptimizedQueries::new(db);

        // Test course statistics
        let stats = queries.get_course_statistics().unwrap();
        assert_eq!(stats.total_courses, 0);

        // Test recent activity
        let activity = queries.get_recent_activity(10).unwrap();
        assert!(activity.is_empty());

        // Test search
        let results = queries.search_all_content("test", 10).unwrap();
        assert!(results.is_empty());
    }
}
