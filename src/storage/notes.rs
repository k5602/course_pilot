use crate::types::Note;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, Row, ToSql, params};
use serde_json;
use uuid::Uuid;

/// Initialize the notes table if it doesn't exist, and perform migrations for course_id and video_id.
pub fn init_notes_table(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id          TEXT PRIMARY KEY,
            course_id   TEXT NOT NULL,
            video_id    TEXT,
            content     TEXT NOT NULL,
            timestamp   INTEGER,
            created_at  TEXT NOT NULL,
            updated_at  TEXT NOT NULL,
            tags        TEXT DEFAULT '[]'
        );
        CREATE INDEX IF NOT EXISTS idx_notes_course_id ON notes(course_id);
        CREATE INDEX IF NOT EXISTS idx_notes_video_id ON notes(video_id);
        "#,
    )
    .context("Failed to create notes table")?;

    // Migration: add missing columns if needed
    let mut stmt = conn.prepare("PRAGMA table_info(notes);")?;
    let columns: Vec<String> = stmt
        .query_map([], |row| row.get(1))?
        .collect::<std::result::Result<Vec<String>, _>>()?;

    if !columns.iter().any(|c| c == "tags") {
        conn.execute("ALTER TABLE notes ADD COLUMN tags TEXT DEFAULT '[]';", [])?;
    }
    if !columns.iter().any(|c| c == "course_id") {
        // If course_id is missing, add it as nullable, then update to NOT NULL if possible.
        conn.execute("ALTER TABLE notes ADD COLUMN course_id TEXT;", [])?;
        // You may want to run a migration script to populate course_id for existing notes.
    }
    if !columns.iter().any(|c| c == "video_id") {
        conn.execute("ALTER TABLE notes ADD COLUMN video_id TEXT;", [])?;
    }
    Ok(())
}

/// Insert a new note into the database.
pub fn create_note(conn: &Connection, note: &Note) -> Result<()> {
    conn.execute(
        r#"
        INSERT INTO notes (id, course_id, video_id, content, timestamp, created_at, updated_at, tags)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            note.id.to_string(),
            note.course_id.to_string(),
            note.video_id.as_ref().map(|v| v.to_string()),
            note.content,
            note.timestamp.map(|t| t as i64),
            note.created_at.to_rfc3339(),
            note.updated_at.to_rfc3339(),
            serde_json::to_string(&note.tags)?,
        ],
    )
    .context("Failed to insert note")?;
    Ok(())
}

/// Update an existing note by id.
pub fn update_note(conn: &Connection, note: &Note) -> Result<()> {
    conn.execute(
        r#"
        UPDATE notes
        SET course_id = ?1, video_id = ?2, content = ?3, timestamp = ?4, updated_at = ?5, tags = ?6
        WHERE id = ?7
        "#,
        params![
            note.course_id.to_string(),
            note.video_id.as_ref().map(|v| v.to_string()),
            note.content,
            note.timestamp.map(|t| t as i64),
            note.updated_at.to_rfc3339(),
            serde_json::to_string(&note.tags)?,
            note.id.to_string(),
        ],
    )
    .context("Failed to update note")?;
    Ok(())
}

/// Delete a note by id.
pub fn delete_note(conn: &Connection, note_id: Uuid) -> Result<()> {
    conn.execute(
        "DELETE FROM notes WHERE id = ?1",
        params![note_id.to_string()],
    )
    .context("Failed to delete note")?;
    Ok(())
}

/// Get all notes across all courses.
pub fn get_all_notes(conn: &Connection) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, video_id, content, timestamp, created_at, updated_at, tags FROM notes ORDER BY updated_at DESC",
    )?;
    let notes = stmt
        .query_map([], note_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

/// Get all notes for a given course (both course-level and video-level).
pub fn get_notes_by_course(conn: &Connection, course_id: Uuid) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, video_id, content, timestamp, created_at, updated_at, tags FROM notes WHERE course_id = ?1 ORDER BY created_at ASC",
    )?;
    let notes = stmt
        .query_map(params![course_id.to_string()], note_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

/// Get all notes for a given video (video-level notes only).
pub fn get_notes_by_video(conn: &Connection, video_id: Uuid) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, video_id, content, timestamp, created_at, updated_at, tags FROM notes WHERE video_id = ?1 ORDER BY created_at ASC",
    )?;
    let notes = stmt
        .query_map(params![video_id.to_string()], note_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

/// Get all course-level notes (notes not tied to a specific video) for a course.
pub fn get_course_level_notes(conn: &Connection, course_id: Uuid) -> Result<Vec<Note>> {
    let mut stmt = conn.prepare(
        "SELECT id, course_id, video_id, content, timestamp, created_at, updated_at, tags FROM notes WHERE course_id = ?1 AND video_id IS NULL ORDER BY created_at ASC",
    )?;
    let notes = stmt
        .query_map(params![course_id.to_string()], note_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

/// Get a single note by id.
pub fn get_note_by_id(conn: &Connection, note_id: Uuid) -> Result<Option<Note>> {
    conn.query_row(
        "SELECT id, course_id, video_id, content, timestamp, created_at, updated_at, tags FROM notes WHERE id = ?1",
        params![note_id.to_string()],
        note_from_row,
    )
    .optional()
    .context("Failed to fetch note by id")
}

/// Search notes by content (case-insensitive LIKE).
pub fn search_notes(conn: &Connection, query: &str) -> Result<Vec<Note>> {
    let pattern = format!("%{query}%");
    let mut stmt = conn.prepare(
        "SELECT id, course_id, video_id, content, timestamp, created_at, updated_at, tags FROM notes WHERE content LIKE ?1 COLLATE NOCASE ORDER BY updated_at DESC",
    )?;
    let notes = stmt
        .query_map(params![pattern], note_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

/// Advanced search filters for notes.
pub struct NoteSearchFilters<'a> {
    pub course_id: Option<Uuid>,
    pub video_id: Option<Option<Uuid>>, // Some(Some) = filter by video, Some(None) = course-level, None = ignore
    pub content: Option<&'a str>,
    pub tags: Option<&'a [&'a str]>, // match any tag in list
    pub timestamp_min: Option<u32>,
    pub timestamp_max: Option<u32>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub updated_after: Option<DateTime<Utc>>,
    pub updated_before: Option<DateTime<Utc>>,
}

/// Advanced search for notes with flexible filters.
/// All filters are optional and can be combined.
pub fn search_notes_advanced(conn: &Connection, filters: NoteSearchFilters) -> Result<Vec<Note>> {
    let mut sql = String::from(
        "SELECT id, course_id, video_id, content, timestamp, created_at, updated_at, tags FROM notes WHERE 1=1",
    );
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    if let Some(course_id) = filters.course_id {
        sql.push_str(" AND course_id = ? ");
        params.push(Box::new(course_id.to_string()));
    }
    if let Some(video_id_opt) = filters.video_id {
        match video_id_opt {
            Some(video_id) => {
                sql.push_str(" AND video_id = ? ");
                params.push(Box::new(video_id.to_string()));
            }
            None => {
                sql.push_str(" AND video_id IS NULL ");
            }
        }
    }
    if let Some(content) = filters.content {
        sql.push_str(" AND content LIKE ? ");
        params.push(Box::new(format!("%{content}%")));
    }
    if let Some(tags) = filters.tags {
        // Match any tag in the array (simple LIKE for MVP, can be improved with JSON1 extension)
        for tag in tags {
            sql.push_str(" AND tags LIKE ? ");
            params.push(Box::new(format!("%{tag}%")));
        }
    }
    if let Some(ts_min) = filters.timestamp_min {
        sql.push_str(" AND timestamp >= ? ");
        params.push(Box::new(ts_min as i64));
    }
    if let Some(ts_max) = filters.timestamp_max {
        sql.push_str(" AND timestamp <= ? ");
        params.push(Box::new(ts_max as i64));
    }
    if let Some(created_after) = filters.created_after {
        sql.push_str(" AND created_at >= ? ");
        params.push(Box::new(created_after.to_rfc3339()));
    }
    if let Some(created_before) = filters.created_before {
        sql.push_str(" AND created_at <= ? ");
        params.push(Box::new(created_before.to_rfc3339()));
    }
    if let Some(updated_after) = filters.updated_after {
        sql.push_str(" AND updated_at >= ? ");
        params.push(Box::new(updated_after.to_rfc3339()));
    }
    if let Some(updated_before) = filters.updated_before {
        sql.push_str(" AND updated_at <= ? ");
        params.push(Box::new(updated_before.to_rfc3339()));
    }
    sql.push_str(" ORDER BY updated_at DESC");

    let mut stmt = conn.prepare(&sql)?;
    let notes = stmt
        .query_map(
            rusqlite::params_from_iter(params.iter().map(|b| &**b)),
            note_from_row,
        )?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(notes)
}

/// Export all notes for a course as a markdown string (includes both course-level and video-level notes).
pub fn export_notes_markdown_by_course(conn: &Connection, course_id: Uuid) -> Result<String> {
    let notes = get_notes_by_course(conn, course_id)?;
    let mut md = String::new();
    for note in notes {
        let ts = note
            .timestamp
            .map(|t| format!(" at {t}s"))
            .unwrap_or_default();
        let video_info = match note.video_id {
            Some(_) => format!("Video Note{ts}"),
            None => "Course Note".to_string(),
        };
        md.push_str(&format!(
            "### {} ({})\n{}\n\n---\n\n",
            video_info,
            note.created_at.format("%Y-%m-%d %H:%M"),
            note.content
        ));
    }
    Ok(md)
}

/// Export all notes for a video as a markdown string (video-level notes only).
pub fn export_notes_markdown_by_video(conn: &Connection, video_id: Uuid) -> Result<String> {
    let notes = get_notes_by_video(conn, video_id)?;
    let mut md = String::new();
    for note in notes {
        let ts = note
            .timestamp
            .map(|t| format!(" at {t}s"))
            .unwrap_or_default();
        md.push_str(&format!(
            "### Video Note{} ({})\n{}\n\n---\n\n",
            ts,
            note.created_at.format("%Y-%m-%d %H:%M"),
            note.content
        ));
    }
    Ok(md)
}

/// Render a note's markdown content to HTML using markdown-rs.
pub fn render_note_html(note: &Note) -> String {
    markdown::to_html(&note.content)
}

/// Helper: Map rusqlite row to Note struct.
fn note_from_row(row: &Row) -> std::result::Result<Note, rusqlite::Error> {
    use rusqlite::Error as SqlError;

    let id = Uuid::parse_str(row.get::<_, String>(0)?.as_str())
        .map_err(|e| SqlError::ToSqlConversionFailure(Box::new(e)))?;
    let course_id = Uuid::parse_str(row.get::<_, String>(1)?.as_str())
        .map_err(|e| SqlError::ToSqlConversionFailure(Box::new(e)))?;
    let video_id = match row.get::<_, Option<String>>(2)? {
        Some(s) => {
            Some(Uuid::parse_str(&s).map_err(|e| SqlError::ToSqlConversionFailure(Box::new(e)))?)
        }
        None => None,
    };
    let content = row.get(3)?;
    let timestamp = row.get::<_, Option<i64>>(4)?.map(|t| t as u32);
    let created_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(5)?)
        .map_err(|e| SqlError::ToSqlConversionFailure(Box::new(e)))?
        .with_timezone(&Utc);
    let updated_at = DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
        .map_err(|e| SqlError::ToSqlConversionFailure(Box::new(e)))?
        .with_timezone(&Utc);
    let tags = match row.get::<_, Option<String>>(7)? {
        Some(json) => serde_json::from_str(&json).unwrap_or_default(),
        None => Vec::new(),
    };

    Ok(Note {
        id,
        course_id,
        video_id,
        content,
        timestamp,
        created_at,
        updated_at,
        tags,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn setup_conn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_notes_table(&conn).unwrap();
        conn
    }

    fn sample_note(course_id: Uuid, video_id: Option<Uuid>) -> Note {
        let now = Utc::now();
        Note {
            id: Uuid::new_v4(),
            course_id,
            video_id,
            content: "This is a **test** note.".to_string(),
            timestamp: Some(42),
            created_at: now,
            updated_at: now,
            tags: vec!["rust".to_string(), "sqlite".to_string()],
        }
    }

    #[test]
    fn test_create_and_get_note() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let note = sample_note(course_id, video_id);
        create_note(&conn, &note).unwrap();

        let fetched = get_note_by_id(&conn, note.id).unwrap().unwrap();
        assert_eq!(fetched.content, note.content);
        assert_eq!(fetched.timestamp, note.timestamp);
        assert_eq!(fetched.course_id, course_id);
        assert_eq!(fetched.video_id, video_id);
    }

    #[test]
    fn test_create_and_get_course_level_note() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let note = sample_note(course_id, None);
        create_note(&conn, &note).unwrap();

        let fetched = get_note_by_id(&conn, note.id).unwrap().unwrap();
        assert_eq!(fetched.content, note.content);
        assert_eq!(fetched.video_id, None);
        assert_eq!(fetched.course_id, course_id);
    }

    #[test]
    fn test_update_note() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let mut note = sample_note(course_id, video_id);
        create_note(&conn, &note).unwrap();

        note.content = "Updated content".to_string();
        note.updated_at = Utc::now();
        update_note(&conn, &note).unwrap();

        let fetched = get_note_by_id(&conn, note.id).unwrap().unwrap();
        assert_eq!(fetched.content, "Updated content");
    }

    #[test]
    fn test_delete_note() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let note = sample_note(course_id, video_id);
        create_note(&conn, &note).unwrap();

        delete_note(&conn, note.id).unwrap();
        let fetched = get_note_by_id(&conn, note.id).unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_get_notes_by_course() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let note1 = sample_note(course_id, video_id);
        let note2 = sample_note(course_id, None);
        create_note(&conn, &note1).unwrap();
        create_note(&conn, &note2).unwrap();

        let notes = get_notes_by_course(&conn, course_id).unwrap();
        assert_eq!(notes.len(), 2);
        assert!(notes.iter().any(|n| n.video_id == video_id));
        assert!(notes.iter().any(|n| n.video_id.is_none()));
    }

    #[test]
    fn test_get_notes_by_video() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let note1 = sample_note(course_id, video_id);
        let note2 = sample_note(course_id, None);
        create_note(&conn, &note1).unwrap();
        create_note(&conn, &note2).unwrap();

        let notes = get_notes_by_video(&conn, video_id.unwrap()).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].video_id, video_id);
    }

    #[test]
    fn test_get_course_level_notes() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let note1 = sample_note(course_id, None);
        let note2 = sample_note(course_id, Some(Uuid::new_v4()));
        create_note(&conn, &note1).unwrap();
        create_note(&conn, &note2).unwrap();

        let notes = get_course_level_notes(&conn, course_id).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].video_id, None);
    }

    #[test]
    fn test_search_notes() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let note1 = Note {
            content: "Rust is great".to_string(),
            tags: vec!["rust".to_string(), "lang".to_string()],
            ..sample_note(course_id, video_id)
        };
        let note2 = Note {
            content: "Learning SQLite".to_string(),
            tags: vec!["sqlite".to_string()],
            ..sample_note(course_id, None)
        };
        create_note(&conn, &note1).unwrap();
        create_note(&conn, &note2).unwrap();

        let results = search_notes(&conn, "rust").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Rust is great");
    }

    #[test]
    fn test_search_notes_advanced() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let note1 = Note {
            content: "Rust is great".to_string(),
            tags: vec!["rust".to_string(), "lang".to_string()],
            timestamp: Some(10),
            ..sample_note(course_id, video_id)
        };
        let note2 = Note {
            content: "Learning SQLite".to_string(),
            tags: vec!["sqlite".to_string()],
            timestamp: Some(50),
            ..sample_note(course_id, None)
        };
        create_note(&conn, &note1).unwrap();
        create_note(&conn, &note2).unwrap();

        // Search by tag
        let filters = super::NoteSearchFilters {
            course_id: Some(course_id),
            video_id: None,
            content: None,
            tags: Some(&["rust"]),
            timestamp_min: None,
            timestamp_max: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
        };
        let results = super::search_notes_advanced(&conn, filters).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Rust is great"); // Only course-level notes

        // Search by video_id
        let filters = super::NoteSearchFilters {
            course_id: Some(course_id),
            video_id: Some(video_id),
            content: None,
            tags: Some(&["rust"]),
            timestamp_min: None,
            timestamp_max: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
        };
        let results = super::search_notes_advanced(&conn, filters).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Rust is great");

        // Search by timestamp range
        let filters = super::NoteSearchFilters {
            course_id: Some(course_id),
            video_id: None,
            content: None,
            tags: None,
            timestamp_min: Some(40),
            timestamp_max: Some(60),
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
        };
        let results = super::search_notes_advanced(&conn, filters).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Learning SQLite");

        // Search by content and tag
        let filters = super::NoteSearchFilters {
            course_id: Some(course_id),
            video_id: Some(video_id),
            content: Some("great"),
            tags: Some(&["lang"]),
            timestamp_min: None,
            timestamp_max: None,
            created_after: None,
            created_before: None,
            updated_after: None,
            updated_before: None,
        };
        let results = super::search_notes_advanced(&conn, filters).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].content, "Rust is great");
    }

    #[test]
    fn test_export_notes_markdown_by_course() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let note1 = sample_note(course_id, Some(Uuid::new_v4()));
        let note2 = sample_note(course_id, None);
        create_note(&conn, &note1).unwrap();
        create_note(&conn, &note2).unwrap();

        let md = export_notes_markdown_by_course(&conn, course_id).unwrap();
        assert!(md.contains("Course Note"));
        assert!(md.contains("Video Note"));
    }

    #[test]
    fn test_export_notes_markdown_by_video() {
        let conn = setup_conn();
        let course_id = Uuid::new_v4();
        let video_id = Some(Uuid::new_v4());
        let note = sample_note(course_id, video_id);
        create_note(&conn, &note).unwrap();

        let md = export_notes_markdown_by_video(&conn, video_id.unwrap()).unwrap();
        assert!(md.contains("Video Note"));
    }

    #[test]
    fn test_render_note_html() {
        let note = Note {
            content: "# Title\nSome *markdown*.".to_string(),
            ..sample_note(Uuid::new_v4(), None)
        };
        let html = render_note_html(&note);
        assert!(html.contains("<h1>Title</h1>"));
        assert!(html.contains("<em>markdown</em>"));
    }
}
