use crate::storage::core::Database;
use crate::types::Plan;
use anyhow::Result;
use chrono::{DateTime, Utc};
use rusqlite::{OptionalExtension, params};

/// Save a plan to the database (insert or replace).
///
/// - Serializes `settings` and `items` as JSON.
/// - Uses INTEGER epoch seconds for `created_at` for performance.
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

/// Load the most recent plan for a given course_id.
pub fn get_plan_by_course_id(db: &Database, course_id: &uuid::Uuid) -> Result<Option<Plan>> {
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
            let id = uuid::Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let course_id = uuid::Uuid::parse_str(&row.get::<_, String>(1)?).map_err(|e| {
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

/// Load a specific plan by its id.
pub fn load_plan(db: &Database, plan_id: &uuid::Uuid) -> Result<Option<Plan>> {
    let conn = db.get_conn()?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, course_id, settings, items, created_at
        FROM plans
        WHERE id = ?1
        "#,
    )?;

    let plan = stmt
        .query_row(params![plan_id.to_string()], |row| {
            let id = uuid::Uuid::parse_str(&row.get::<_, String>(0)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    0,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?;

            let course_id = uuid::Uuid::parse_str(&row.get::<_, String>(1)?).map_err(|e| {
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

/// Delete a plan by id.
pub fn delete_plan(db: &Database, plan_id: &uuid::Uuid) -> Result<()> {
    let conn = db.get_conn()?;
    conn.execute("DELETE FROM plans WHERE id = ?1", params![plan_id.to_string()])?;
    Ok(())
}
