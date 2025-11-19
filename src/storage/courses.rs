use crate::storage::core::Database;
use crate::types::{
    ClusteringMetadata, Course, CourseStructure, DifficultyLevel, Module, Section,
    StructureMetadata, VideoMetadata,
};
use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::de::DeserializeOwned;
use serde_json;
use std::time::Duration;
use uuid::Uuid;

fn parse_uuid_sqlite(s: &str, idx: usize) -> Result<Uuid, rusqlite::Error> {
    Uuid::parse_str(s).map_err(|_| {
        rusqlite::Error::InvalidColumnType(idx, "uuid".to_string(), rusqlite::types::Type::Text)
    })
}

fn parse_json_sqlite_at<T: DeserializeOwned>(s: &str, idx: usize) -> Result<T, rusqlite::Error> {
    serde_json::from_str(s).map_err(|e| {
        rusqlite::Error::InvalidColumnType(idx, format!("json: {e}"), rusqlite::types::Type::Text)
    })
}

fn parse_json_sqlite<T: DeserializeOwned>(s: &str) -> Result<T, rusqlite::Error> {
    parse_json_sqlite_at(s, 0)
}

fn validate_video_metadata(videos: &[VideoMetadata]) -> Result<Vec<VideoMetadata>> {
    let mut validated_videos = Vec::with_capacity(videos.len());

    for (index, video) in videos.iter().enumerate() {
        info!(
            "Validating video {}: title='{}', video_id={:?}, source_url={:?}, is_local={}",
            index, video.title, video.video_id, video.source_url, video.is_local
        );

        let mut validated_video = video.clone();

        if !video.is_local {
            if video.video_id.is_none() && video.source_url.is_none() {
                warn!(
                    "YouTube video at index {} missing both video_id and source_url: '{}'",
                    index, video.title
                );
                return Err(anyhow!(
                    "Cannot save YouTube video '{}' at index {}: missing video_id and source_url",
                    video.title,
                    index
                ));
            }

            if video.video_id.is_some() && video.source_url.is_none() {
                if let Some(ref video_id) = video.video_id {
                    let url = if let Some(ref playlist_id) = video.playlist_id {
                        format!("https://www.youtube.com/watch?v={}&list={}", video_id, playlist_id)
                    } else {
                        format!("https://www.youtube.com/watch?v={}", video_id)
                    };
                    validated_video.source_url = Some(url);
                }
            } else if video.video_id.is_none() && video.source_url.is_some() {
                let url = video.source_url.clone().unwrap_or_default();
                if url.is_empty() {
                    return Err(anyhow!(
                        "YouTube video '{}' at index {} has empty source_url",
                        video.title,
                        index
                    ));
                }
            } else if let Some(ref video_id) = video.video_id {
                if video_id.starts_with("PLACEHOLDER_") {
                    return Err(anyhow!(
                        "Cannot save YouTube video '{}' at index {}: placeholder video_id",
                        video.title,
                        index
                    ));
                }
            }
        } else if video.source_url.is_none() {
            validated_video.source_url = Some(video.title.clone());
        }

        validated_videos.push(validated_video);
    }

    Ok(validated_videos)
}

fn persist_course_videos(
    tx: &rusqlite::Transaction<'_>,
    course_id: &Uuid,
    videos: &[VideoMetadata],
) -> Result<()> {
    tx.execute("DELETE FROM course_videos WHERE course_id = ?1", params![course_id.to_string()])?;

    for (video_index, video) in videos.iter().enumerate() {
        tx.execute(
            r#"
            INSERT INTO course_videos (
                course_id, video_index, title, source_url, video_id, playlist_id,
                original_index, duration_seconds, thumbnail_url, description, upload_date,
                author, view_count, tags, is_local
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)
            "#,
            params![
                course_id.to_string(),
                video_index as i64,
                video.title,
                video.source_url,
                video.video_id,
                video.playlist_id,
                video.original_index as i64,
                video.duration_seconds,
                video.thumbnail_url,
                video.description,
                video.upload_date.map(|ts| ts.timestamp()),
                video.author,
                video.view_count.map(|v| v as i64),
                serde_json::to_string(&video.tags)?,
                if video.is_local { 1 } else { 0 },
            ],
        )?;
    }

    Ok(())
}

fn persist_course_structure(
    tx: &rusqlite::Transaction<'_>,
    course_id: &Uuid,
    structure: Option<&CourseStructure>,
) -> Result<()> {
    tx.execute(
        "DELETE FROM course_structures WHERE course_id = ?1",
        params![course_id.to_string()],
    )?;
    tx.execute("DELETE FROM course_modules WHERE course_id = ?1", params![course_id.to_string()])?;

    if let Some(structure) = structure {
        tx.execute(
            r#"
            INSERT INTO course_structures (course_id, metadata, clustering_metadata)
            VALUES (?1, ?2, ?3)
            "#,
            params![
                course_id.to_string(),
                serde_json::to_string(&structure.metadata)?,
                structure
                    .clustering_metadata
                    .as_ref()
                    .map(|meta| serde_json::to_string(meta))
                    .transpose()?
            ],
        )?;

        for (module_index, module) in structure.modules.iter().enumerate() {
            tx.execute(
                r#"
                INSERT INTO course_modules (
                    course_id, module_index, title, total_duration, similarity_score,
                    topic_keywords, difficulty_level
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    course_id.to_string(),
                    module_index as i64,
                    module.title,
                    module.total_duration.as_secs() as i64,
                    module.similarity_score,
                    serde_json::to_string(&module.topic_keywords)?,
                    module
                        .difficulty_level
                        .as_ref()
                        .map(|level| serde_json::to_string(level))
                        .transpose()?
                ],
            )?;

            let module_id = tx.last_insert_rowid();
            for (section_index, section) in module.sections.iter().enumerate() {
                tx.execute(
                    r#"
                    INSERT INTO module_sections (
                        module_id, section_index, title, video_index, duration
                    ) VALUES (?1, ?2, ?3, ?4, ?5)
                    "#,
                    params![
                        module_id,
                        section_index as i64,
                        section.title,
                        section.video_index as i64,
                        section.duration.as_secs() as i64,
                    ],
                )?;
            }
        }
    }

    Ok(())
}

fn load_course_videos(conn: &Connection, course_id: &Uuid) -> Result<Vec<VideoMetadata>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT video_index, title, source_url, video_id, playlist_id, original_index,
               duration_seconds, thumbnail_url, description, upload_date, author,
               view_count, tags, is_local
        FROM course_videos
        WHERE course_id = ?1
        ORDER BY video_index ASC
        "#,
    )?;

    let mut rows = stmt.query(params![course_id.to_string()])?;
    let mut videos = Vec::new();

    while let Some(row) = rows.next()? {
        let duration_seconds: Option<f64> = row.get(6)?;
        let upload_date: Option<i64> = row.get(9)?;
        let view_count: Option<i64> = row.get(11)?;
        let tags_json: String = row.get(12).unwrap_or_else(|_| "[]".to_string());
        let is_local: i64 = row.get(13)?;

        let video = VideoMetadata {
            title: row.get(1)?,
            source_url: row.get::<_, Option<String>>(2)?,
            video_id: row.get::<_, Option<String>>(3)?,
            playlist_id: row.get::<_, Option<String>>(4)?,
            original_index: row.get::<_, i64>(5)? as usize,
            duration_seconds,
            thumbnail_url: row.get::<_, Option<String>>(7)?,
            description: row.get::<_, Option<String>>(8)?,
            upload_date: upload_date
                .and_then(|ts| DateTime::from_timestamp(ts, 0))
                .map(|dt| dt.with_timezone(&Utc)),
            author: row.get::<_, Option<String>>(10)?,
            view_count: view_count.map(|v| v as u64),
            tags: serde_json::from_str(&tags_json).unwrap_or_default(),
            is_local: is_local != 0,
        };

        videos.push(video);
    }

    Ok(videos)
}

fn load_course_modules(conn: &Connection, course_id: &Uuid) -> Result<Vec<Module>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT id, title, module_index, total_duration, similarity_score,
               topic_keywords, difficulty_level
        FROM course_modules
        WHERE course_id = ?1
        ORDER BY module_index ASC
        "#,
    )?;

    let mut rows = stmt.query(params![course_id.to_string()])?;
    let mut modules = Vec::new();

    while let Some(row) = rows.next()? {
        let module_id: i64 = row.get(0)?;
        let similarity_score: Option<f64> = row.get(4).ok();
        let topic_keywords_json: String = row.get(5).unwrap_or_else(|_| "[]".to_string());
        let difficulty_json: Option<String> = row.get(6).ok();

        let mut section_stmt = conn.prepare(
            r#"
            SELECT title, video_index, duration
            FROM module_sections
            WHERE module_id = ?1
            ORDER BY section_index ASC
            "#,
        )?;

        let mut section_rows = section_stmt.query(params![module_id])?;
        let mut sections = Vec::new();
        while let Some(section_row) = section_rows.next()? {
            sections.push(Section {
                title: section_row.get(0)?,
                video_index: section_row.get::<_, i64>(1)? as usize,
                duration: Duration::from_secs(section_row.get::<_, i64>(2)? as u64),
            });
        }

        let module = Module {
            title: row.get(1)?,
            sections,
            total_duration: Duration::from_secs(row.get::<_, i64>(3)? as u64),
            similarity_score,
            topic_keywords: serde_json::from_str(&topic_keywords_json).unwrap_or_default(),
            difficulty_level: difficulty_json
                .map(|json| serde_json::from_str(&json))
                .transpose()?
                .map(|level: DifficultyLevel| level),
        };

        modules.push(module);
    }

    Ok(modules)
}

fn load_course_structure(conn: &Connection, course_id: &Uuid) -> Result<Option<CourseStructure>> {
    let row = conn
        .query_row(
            "SELECT metadata, clustering_metadata FROM course_structures WHERE course_id = ?1",
            params![course_id.to_string()],
            |row| {
                let metadata: String = row.get(0)?;
                let clustering: Option<String> = row.get(1).ok();
                Ok((metadata, clustering))
            },
        )
        .optional()?;

    let Some((metadata_json, clustering_json)) = row else {
        return Ok(None);
    };

    let metadata: StructureMetadata = serde_json::from_str(&metadata_json)?;
    let clustering_metadata: Option<ClusteringMetadata> =
        clustering_json.map(|json| serde_json::from_str(&json)).transpose()?;

    let modules = load_course_modules(conn, course_id)?;
    Ok(Some(CourseStructure { modules, metadata, clustering_metadata }))
}

fn legacy_course_from_json(
    id: Uuid,
    name: String,
    created_at: i64,
    raw_titles_json: String,
    videos_json: Option<String>,
    structure_json: Option<String>,
) -> Result<Course> {
    let raw_titles: Vec<String> =
        parse_json_sqlite(&raw_titles_json).map_err(anyhow::Error::new)?;

    let videos: Vec<VideoMetadata> = if let Some(ref videos_json) = videos_json {
        let parsed_videos: Vec<VideoMetadata> =
            serde_json::from_str(videos_json).unwrap_or_else(|e| {
                warn!("Failed to deserialize video metadata, using fallback: {}", e);
                create_fallback_video_metadata(&raw_titles)
            });

        validate_and_repair_loaded_metadata(parsed_videos, &raw_titles).unwrap_or_else(|e| {
            error!("Failed to repair loaded metadata: {}", e);
            create_fallback_video_metadata(&raw_titles)
        })
    } else {
        info!("No video metadata found, creating from raw_titles");
        create_fallback_video_metadata(&raw_titles)
    };

    let structure = structure_json
        .as_ref()
        .map(|json| parse_json_sqlite::<CourseStructure>(json))
        .transpose()
        .map_err(anyhow::Error::new)?;

    Ok(Course {
        id,
        name,
        created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
        raw_titles,
        videos,
        structure,
    })
}

fn load_course_row(conn: &Connection, row: &Row<'_>) -> Result<Course> {
    let id_str: String = row.get(0)?;
    let id = parse_uuid_sqlite(&id_str, 0).map_err(anyhow::Error::new)?;
    let name: String = row.get(1)?;
    let created_at: i64 = row.get(2)?;
    let raw_titles_json: String = row.get(3)?;
    let videos_json: Option<String> = row.get(4)?;
    let structure_json: Option<String> = row.get(5)?;

    let videos = load_course_videos(conn, &id)?;
    if !videos.is_empty() {
        let raw_titles = videos.iter().map(|v| v.title.clone()).collect();
        let structure = load_course_structure(conn, &id)?;
        return Ok(Course {
            id,
            name,
            created_at: DateTime::from_timestamp(created_at, 0).unwrap_or_else(Utc::now),
            raw_titles,
            videos,
            structure,
        });
    }

    legacy_course_from_json(id, name, created_at, raw_titles_json, videos_json, structure_json)
}

fn validate_and_repair_loaded_metadata(
    parsed_videos: Vec<VideoMetadata>,
    raw_titles: &[String],
) -> Result<Vec<VideoMetadata>> {
    let mut repaired_videos = Vec::with_capacity(parsed_videos.len().max(raw_titles.len()));

    for (index, video) in parsed_videos.into_iter().enumerate() {
        let mut repaired_video = video.clone();

        if !video.is_local && video.video_id.is_none() && video.source_url.is_none() {
            warn!(
                "Found YouTube video with missing metadata during load, repairing: '{}'",
                video.title
            );

            repaired_video.video_id = Some(format!("PLACEHOLDER_{}", index));
            repaired_video.source_url =
                Some(format!("https://www.youtube.com/watch?v=PLACEHOLDER_{}", index));
            repaired_video.playlist_id = None;
            if repaired_video.original_index == 0 && index > 0 {
                repaired_video.original_index = index;
            }
        } else if !video.is_local && video.video_id.is_some() && video.source_url.is_none() {
            if let Some(ref video_id) = video.video_id {
                let url = if let Some(ref playlist_id) = repaired_video.playlist_id {
                    format!("https://www.youtube.com/watch?v={}&list={}", video_id, playlist_id)
                } else {
                    format!("https://www.youtube.com/watch?v={}", video_id)
                };
                repaired_video.source_url = Some(url);
            }
        } else if video.is_local && video.source_url.is_none() {
            repaired_video.source_url = Some(video.title.clone());
        }

        if repaired_video.original_index != index {
            repaired_video.original_index = index;
        }

        repaired_videos.push(repaired_video);
    }

    if repaired_videos.len() < raw_titles.len() {
        warn!(
            "Video metadata count ({}) less than raw_titles count ({}), padding with fallback",
            repaired_videos.len(),
            raw_titles.len()
        );

        for i in repaired_videos.len()..raw_titles.len() {
            repaired_videos.push(VideoMetadata::new_local_with_index(
                raw_titles[i].clone(),
                raw_titles[i].clone(),
                i,
            ));
        }
    }

    Ok(repaired_videos)
}

fn create_fallback_video_metadata(raw_titles: &[String]) -> Vec<VideoMetadata> {
    raw_titles
        .iter()
        .enumerate()
        .map(|(index, title)| {
            VideoMetadata::new_local_with_index(title.clone(), title.clone(), index)
        })
        .collect()
}

pub fn save_course(db: &Database, course: &Course) -> Result<()> {
    info!("Saving course: {} (ID: {})", course.name, course.id);

    let raw_titles_json = serde_json::to_string(&course.raw_titles)
        .with_context(|| format!("Failed to serialize raw_titles for '{}'", course.name))?;

    let validated_videos = validate_video_metadata(&course.videos)?;
    let videos_json = serde_json::to_string(&validated_videos)
        .with_context(|| format!("Failed to serialize videos for '{}'", course.name))?;

    let structure_json =
        course.structure.as_ref().map(serde_json::to_string).transpose().map_err(|e| {
            error!("Failed to serialize structure for course {}: {}", course.name, e);
            e
        })?;

    let conn = db.get_conn().with_context(|| {
        format!("Failed to get database connection for saving course {}", course.name)
    })?;

    let tx = conn.transaction()?;
    tx.execute(
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
    )?;

    persist_course_videos(&tx, &course.id, &validated_videos)?;
    persist_course_structure(&tx, &course.id, course.structure.as_ref())?;

    tx.commit()?;
    info!("Successfully saved course: {}", course.name);
    Ok(())
}

pub fn load_courses(db: &Database) -> Result<Vec<Course>> {
    info!("Loading all courses from database");
    let conn =
        db.get_conn().with_context(|| "Failed to get database connection for loading courses")?;

    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, created_at, raw_titles, videos, structure
        FROM courses
        ORDER BY created_at DESC
        "#,
    )?;

    let mut rows = stmt.query([])?;
    let mut courses = Vec::new();
    while let Some(row) = rows.next()? {
        courses.push(load_course_row(&conn, row)?);
    }

    Ok(courses)
}

pub fn get_course_by_id(db: &Database, course_id: &Uuid) -> Result<Option<Course>> {
    let conn = db.get_conn().with_context(|| "Failed to get DB connection")?;
    let mut stmt = conn.prepare(
        r#"
        SELECT id, name, created_at, raw_titles, videos, structure
        FROM courses
        WHERE id = ?1
        "#,
    )?;

    let course = stmt
        .query_row(params![course_id.to_string()], |row| load_course_row(&conn, row))
        .optional()?;

    Ok(course)
}

pub fn delete_course(db: &Database, course_id: &Uuid) -> Result<()> {
    let mut conn = db.get_conn().with_context(|| "Failed to get DB connection")?;
    let tx = conn.transaction()?;
    tx.execute("DELETE FROM plans WHERE course_id = ?1", params![course_id.to_string()])?;
    tx.execute("DELETE FROM courses WHERE id = ?1", params![course_id.to_string()])?;
    tx.commit()?;
    Ok(())
}
