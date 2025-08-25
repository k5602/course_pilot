#![allow(dead_code)]
//! Video progress persistence
//!
//! This module provides CRUD helpers for saving and querying per-video completion
//! progress for a study plan session. It expects the `video_progress` table to
//! exist with the following schema (created by storage core init):
//!
//! CREATE TABLE IF NOT EXISTS video_progress (
//!     id INTEGER PRIMARY KEY AUTOINCREMENT,
//!     plan_id TEXT NOT NULL,
//!     session_index INTEGER NOT NULL,
//!     video_index INTEGER NOT NULL,
//!     completed BOOLEAN NOT NULL DEFAULT 0,
//!     updated_at TEXT NOT NULL,
//!     UNIQUE(plan_id, session_index, video_index)
//! );
//! CREATE INDEX IF NOT EXISTS idx_video_progress_plan_session
//!   ON video_progress(plan_id, session_index);

use crate::error_handling::DatabaseError;
use crate::storage::core::Database;
use rusqlite::params;

/// Save video progress tracking information (insert or replace).
///
/// - Overwrites the existing row for (plan_id, session_index, video_index)
/// - Uses RFC3339 for updated_at timestamp
pub fn save_video_progress(
    db: &Database,
    progress: &crate::types::VideoProgressUpdate,
) -> Result<(), DatabaseError> {
    let conn =
        db.get_conn().map_err(|e| DatabaseError::ConnectionFailed { message: e.to_string() })?;

    conn.execute(
        "INSERT OR REPLACE INTO video_progress (
            plan_id, session_index, video_index, completed, updated_at
         ) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            progress.plan_id.to_string(),
            progress.session_index as i64,
            progress.video_index as i64,
            progress.completed,
            chrono::Utc::now().to_rfc3339()
        ],
    )
    .map_err(|e| DatabaseError::QueryFailed {
        query: "INSERT OR REPLACE INTO video_progress".to_string(),
        message: e.to_string(),
    })?;

    Ok(())
}

/// Get video completion status for a specific (plan_id, session_index, video_index)
///
/// Returns:
/// - Ok(true) if completed == 1
/// - Ok(false) if row not found or completed == 0
pub fn get_video_completion_status(
    db: &Database,
    plan_id: &uuid::Uuid,
    session_index: usize,
    video_index: usize,
) -> Result<bool, DatabaseError> {
    let conn =
        db.get_conn().map_err(|e| DatabaseError::ConnectionFailed { message: e.to_string() })?;

    let mut stmt = conn
        .prepare(
            "SELECT completed
             FROM video_progress
             WHERE plan_id = ?1 AND session_index = ?2 AND video_index = ?3",
        )
        .map_err(|e| DatabaseError::QueryFailed {
            query: "SELECT completed FROM video_progress".to_string(),
            message: e.to_string(),
        })?;

    let result = stmt
        .query_row(params![plan_id.to_string(), session_index as i64, video_index as i64], |row| {
            Ok(row.get::<_, bool>(0)?)
        });

    match result {
        Ok(completed) => Ok(completed),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(false), // Default to not completed
        Err(e) => Err(DatabaseError::QueryFailed {
            query: "SELECT completed FROM video_progress".to_string(),
            message: e.to_string(),
        }),
    }
}

/// Get session progress as percentage (0.0 to 1.0)
///
/// Computes:
/// - total_videos = COUNT(*)
/// - completed_videos = SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END)
/// Returns 0.0 if no rows exist for the given (plan_id, session_index)
pub fn get_session_progress(
    db: &Database,
    plan_id: &uuid::Uuid,
    session_index: usize,
) -> Result<f32, DatabaseError> {
    let conn =
        db.get_conn().map_err(|e| DatabaseError::ConnectionFailed { message: e.to_string() })?;

    // Use COALESCE to avoid NULL for SUM on empty sets
    let mut stmt = conn
        .prepare(
            "SELECT
                COUNT(*) as total_videos,
                COALESCE(SUM(CASE WHEN completed = 1 THEN 1 ELSE 0 END), 0) as completed_videos
             FROM video_progress
             WHERE plan_id = ?1 AND session_index = ?2",
        )
        .map_err(|e| DatabaseError::QueryFailed {
            query: "SELECT COUNT/SUM FROM video_progress".to_string(),
            message: e.to_string(),
        })?;

    let (total, completed) = stmt
        .query_row(params![plan_id.to_string(), session_index as i64], |row| {
            let total: i64 = row.get(0)?;
            let completed: i64 = row.get(1)?;
            Ok((total, completed))
        })
        .map_err(|e| DatabaseError::QueryFailed {
            query: "SELECT COUNT/SUM FROM video_progress".to_string(),
            message: e.to_string(),
        })?;

    if total > 0 { Ok(completed as f32 / total as f32) } else { Ok(0.0) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::core::{Database, init_db};
    use tempfile::tempdir;
    use uuid::Uuid;

    #[test]
    fn test_progress_flow() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db: Database = init_db(&db_path).unwrap();

        let plan_id = Uuid::new_v4();
        let session_index = 0usize;
        let video_index = 0usize;

        // Initially no rows -> not completed, progress 0.0
        assert!(!get_video_completion_status(&db, &plan_id, session_index, video_index).unwrap());
        assert_eq!(get_session_progress(&db, &plan_id, session_index).unwrap(), 0.0);

        // Mark as not completed creates a row
        let upd =
            crate::types::VideoProgressUpdate::new(plan_id, session_index, video_index, false);
        save_video_progress(&db, &upd).unwrap();
        assert!(!get_video_completion_status(&db, &plan_id, session_index, video_index).unwrap());
        // One row with completed=0 -> total=1, completed=0
        assert_eq!(get_session_progress(&db, &plan_id, session_index).unwrap(), 0.0);

        // Mark as completed
        let upd2 =
            crate::types::VideoProgressUpdate::new(plan_id, session_index, video_index, true);
        save_video_progress(&db, &upd2).unwrap();
        assert!(get_video_completion_status(&db, &plan_id, session_index, video_index).unwrap());
        assert_eq!(get_session_progress(&db, &plan_id, session_index).unwrap(), 1.0);
    }
}
