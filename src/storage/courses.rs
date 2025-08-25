use crate::storage::core::Database;
use crate::types::{Course, CourseStructure, VideoMetadata};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use log::{error, info, warn};
use rusqlite::{OptionalExtension, params};
use serde::de::DeserializeOwned;
use serde_json;
use uuid::Uuid;

/// ==============================
/// SQLite helpers (local)
/// ==============================
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

/// ==============================
/// YouTube/local video helpers
/// ==============================
fn is_youtube_video_title(title: &str) -> bool {
    if title.contains("youtube.com") || title.contains("youtu.be") || title.contains("watch?v=") {
        return true;
    }

    let t = title.to_lowercase();
    let has_video_extension = t.ends_with(".mp4")
        || t.ends_with(".avi")
        || t.ends_with(".mov")
        || t.ends_with(".mkv")
        || t.ends_with(".webm");

    !has_video_extension && title.len() > 5 && title.len() < 200
}

fn extract_playlist_id_from_url(url: &str) -> Option<String> {
    if let Some(start) = url.find("list=") {
        let id_start = start + 5;
        if let Some(end) = url[id_start..].find('&') {
            Some(url[id_start..id_start + end].to_string())
        } else {
            Some(url[id_start..].to_string())
        }
    } else {
        None
    }
}

fn extract_youtube_video_id_from_title(title: &str) -> Option<String> {
    if let Some(start) = title.find("watch?v=") {
        let id_start = start + 8;
        if let Some(end) = title[id_start..].find('&') {
            Some(title[id_start..id_start + end].to_string())
        } else if let Some(end) = title[id_start..].find(' ') {
            Some(title[id_start..id_start + end].to_string())
        } else {
            let remaining = &title[id_start..];
            if remaining.len() == 11 { Some(remaining.to_string()) } else { None }
        }
    } else if let Some(start) = title.find("youtu.be/") {
        let id_start = start + 9;
        if let Some(end) = title[id_start..].find('?') {
            Some(title[id_start..id_start + end].to_string())
        } else if let Some(end) = title[id_start..].find(' ') {
            Some(title[id_start..id_start + end].to_string())
        } else {
            let remaining = &title[id_start..];
            if remaining.len() == 11 { Some(remaining.to_string()) } else { None }
        }
    } else {
        None
    }
}

/// ==============================
/// Metadata validation/repair
/// ==============================
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

                if let Some(extracted_id) = extract_youtube_video_id_from_title(&video.title) {
                    info!(
                        "Extracted video_id '{}' from title for video at index {}",
                        extracted_id, index
                    );
                    validated_video.video_id = Some(extracted_id.clone());
                    validated_video.source_url =
                        Some(format!("https://www.youtube.com/watch?v={}", extracted_id));
                    if let Some(ref url) = validated_video.source_url {
                        validated_video.playlist_id = extract_playlist_id_from_url(url);
                    }
                } else {
                    return Err(anyhow::anyhow!(
                        "Cannot save YouTube video '{}' at index {}: missing video_id and cannot extract from title. Raw: video_id={:?}, source_url={:?}, is_local={}",
                        video.title,
                        index,
                        video.video_id,
                        video.source_url,
                        video.is_local
                    ));
                }
            } else if video.video_id.is_some() && video.source_url.is_none() {
                if let Some(ref video_id) = video.video_id {
                    let url = if let Some(ref playlist_id) = video.playlist_id {
                        format!("https://www.youtube.com/watch?v={}&list={}", video_id, playlist_id)
                    } else {
                        format!("https://www.youtube.com/watch?v={}", video_id)
                    };
                    validated_video.source_url = Some(url);
                }
            } else if video.video_id.is_none() && video.source_url.is_some() {
                if let Some(ref url) = video.source_url {
                    if let Some(extracted_id) = extract_youtube_video_id_from_title(url) {
                        validated_video.video_id = Some(extracted_id);
                    }
                    if validated_video.playlist_id.is_none() {
                        validated_video.playlist_id = extract_playlist_id_from_url(url);
                    }
                }
            } else {
                if let Some(ref video_id) = video.video_id {
                    if video_id.starts_with("PLACEHOLDER_") {
                        return Err(anyhow::anyhow!(
                            "Cannot save YouTube video '{}' at index {}: contains placeholder video_id '{}'",
                            video.title,
                            index,
                            video_id
                        ));
                    }
                }
            }
        } else {
            if video.source_url.is_none() {
                validated_video.source_url = Some(video.title.clone());
            }
        }

        validated_videos.push(validated_video);
    }

    Ok(validated_videos)
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

            if is_youtube_video_title(&video.title) {
                if let Some(video_id) = extract_youtube_video_id_from_title(&video.title) {
                    info!(
                        "Repaired video_id '{}' from title for video at index {}",
                        video_id, index
                    );
                    repaired_video.video_id = Some(video_id.clone());
                    repaired_video.source_url =
                        Some(format!("https://www.youtube.com/watch?v={}", video_id));
                    if repaired_video.original_index == 0 && index > 0 {
                        repaired_video.original_index = index;
                    }
                } else {
                    warn!(
                        "Could not extract video_id from title, creating placeholder for video at index {}",
                        index
                    );
                    repaired_video.video_id = Some(format!("PLACEHOLDER_{}", index));
                    repaired_video.source_url =
                        Some(format!("https://www.youtube.com/watch?v=PLACEHOLDER_{}", index));
                    repaired_video.playlist_id = None;
                    if repaired_video.original_index == 0 && index > 0 {
                        repaired_video.original_index = index;
                    }
                }
            } else {
                info!("Converting assumed YouTube video to local video at index {}", index);
                repaired_video.is_local = true;
                repaired_video.source_url = Some(video.title.clone());
            }
        } else if !video.is_local && video.video_id.is_some() && video.source_url.is_none() {
            if let Some(ref video_id) = video.video_id {
                info!("Reconstructing source_url for video_id '{}' at index {}", video_id, index);
                let url = if let Some(ref playlist_id) = repaired_video.playlist_id {
                    format!("https://www.youtube.com/watch?v={}&list={}", video_id, playlist_id)
                } else {
                    format!("https://www.youtube.com/watch?v={}", video_id)
                };
                repaired_video.source_url = Some(url);
            }
        } else if !video.is_local && video.video_id.is_none() && video.source_url.is_some() {
            if let Some(ref url) = video.source_url {
                if let Some(extracted_id) = extract_youtube_video_id_from_title(url) {
                    info!(
                        "Extracted video_id '{}' from source_url at index {}",
                        extracted_id, index
                    );
                    repaired_video.video_id = Some(extracted_id);
                }
                if repaired_video.playlist_id.is_none() {
                    repaired_video.playlist_id = extract_playlist_id_from_url(url);
                }
            }
        } else if video.is_local && video.source_url.is_none() {
            info!("Setting source_url for local video at index {}", index);
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
            let fallback_video = if is_youtube_video_title(&raw_titles[i]) {
                if let Some(video_id) = extract_youtube_video_id_from_title(&raw_titles[i]) {
                    VideoMetadata::new_youtube_with_playlist(
                        raw_titles[i].clone(),
                        video_id.clone(),
                        format!("https://www.youtube.com/watch?v={}", video_id),
                        None,
                        i,
                    )
                } else {
                    VideoMetadata {
                        title: raw_titles[i].clone(),
                        source_url: Some(format!(
                            "https://www.youtube.com/watch?v=PLACEHOLDER_{}",
                            i
                        )),
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
                VideoMetadata::new_local_with_index(raw_titles[i].clone(), raw_titles[i].clone(), i)
            };

            repaired_videos.push(fallback_video);
        }
    }

    Ok(repaired_videos)
}

fn create_fallback_video_metadata(raw_titles: &[String]) -> Vec<VideoMetadata> {
    raw_titles
        .iter()
        .enumerate()
        .map(|(index, title)| {
            info!("Creating fallback metadata for video {}: '{}'", index, title);

            if is_youtube_video_title(title) {
                info!("Detected as YouTube video: '{}'", title);
                if let Some(video_id) = extract_youtube_video_id_from_title(title) {
                    info!("Extracted video ID '{}' from title: '{}'", video_id, title);
                    VideoMetadata::new_youtube_with_playlist(
                        title.clone(),
                        video_id.clone(),
                        format!("https://www.youtube.com/watch?v={}", video_id),
                        None,
                        index,
                    )
                } else {
                    warn!(
                        "YouTube video detected but could not extract ID from title: '{}'",
                        title
                    );
                    VideoMetadata {
                        title: title.clone(),
                        source_url: Some(format!(
                            "https://www.youtube.com/watch?v=PLACEHOLDER_{}",
                            index
                        )),
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
                info!("Detected as local video: '{}'", title);
                VideoMetadata::new_local_with_index(title.clone(), title.clone(), index)
            }
        })
        .collect()
}

/// ==============================
/// Public API: Courses CRUD
/// ==============================
pub fn save_course(db: &Database, course: &Course) -> Result<()> {
    info!("Saving course: {} (ID: {})", course.name, course.id);

    let raw_titles_json = serde_json::to_string(&course.raw_titles)
        .with_context(|| format!("Failed to serialize raw_titles for '{}'", course.name))?;

    let validated_videos = validate_video_metadata(&course.videos)?;
    let videos_json = serde_json::to_string(&validated_videos)
        .with_context(|| format!("Failed to serialize videos for '{}'", course.name))?;

    info!("Saving course '{}' with {} videos", course.name, validated_videos.len());
    if !validated_videos.is_empty() {
        let first_video = &validated_videos[0];
        info!(
            "First video: title='{}', video_id={:?}, source_url={:?}, is_local={}",
            first_video.title, first_video.video_id, first_video.source_url, first_video.is_local
        );
        if !first_video.is_local && first_video.video_id.is_none() {
            warn!("YouTube video missing video_id: '{}'", first_video.title);
        }
    }

    let structure_json =
        course.structure.as_ref().map(serde_json::to_string).transpose().map_err(|e| {
            error!("Failed to serialize structure for course {}: {}", course.name, e);
            e
        })?;

    let conn = db.get_conn().with_context(|| {
        format!("Failed to get database connection for saving course {}", course.name)
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
    .with_context(|| format!("Failed to execute SQL for saving course {}", course.name))?;

    info!("Successfully saved course: {}", course.name);
    Ok(())
}

pub fn load_courses(db: &Database) -> Result<Vec<Course>> {
    info!("Loading all courses from database");

    let conn =
        db.get_conn().with_context(|| "Failed to get database connection for loading courses")?;

    let mut stmt = conn
        .prepare(
            r#"
        SELECT id, name, created_at, raw_titles, videos, structure
        FROM courses
        ORDER BY created_at DESC
        "#,
        )
        .with_context(|| "Failed to prepare SQL statement for loading courses")?;

    let courses = stmt
        .query_map([], |row| {
            let id_str: String = row.get(0)?;
            let id = parse_uuid_sqlite(&id_str, 0)?;

            let name: String = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            let raw_titles_json: String = row.get(3)?;
            let videos_json: Option<String> = row.get(4)?;
            let structure_json: Option<String> = row.get(5)?;

            let raw_titles: Vec<String> = parse_json_sqlite_at(&raw_titles_json, 3)?;

            let videos: Vec<VideoMetadata> = if let Some(ref videos_json) = videos_json {
                let parsed_videos: Vec<VideoMetadata> = serde_json::from_str(videos_json)
                    .unwrap_or_else(|e| {
                        warn!("Failed to deserialize video metadata, using fallback: {}", e);
                        create_fallback_video_metadata(&raw_titles)
                    });

                validate_and_repair_loaded_metadata(parsed_videos, &raw_titles).unwrap_or_else(
                    |e| {
                        error!("Failed to repair loaded metadata: {}", e);
                        create_fallback_video_metadata(&raw_titles)
                    },
                )
            } else {
                info!("No video metadata found, creating from raw_titles");
                create_fallback_video_metadata(&raw_titles)
            };

            let structure = structure_json
                .as_ref()
                .map(|json| parse_json_sqlite::<CourseStructure>(json))
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
        .query_row(params![course_id.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let id = parse_uuid_sqlite(&id_str, 0)?;

            let name: String = row.get(1)?;
            let created_at: i64 = row.get(2)?;
            let raw_titles_json: String = row.get(3)?;
            let videos_json: Option<String> = row.get(4)?;
            let structure_json: Option<String> = row.get(5)?;

            let raw_titles: Vec<String> = parse_json_sqlite_at(&raw_titles_json, 3)?;

            let videos: Vec<VideoMetadata> = if let Some(ref videos_json) = videos_json {
                let parsed_videos: Vec<VideoMetadata> =
                    serde_json::from_str(videos_json).unwrap_or_else(|e| {
                        warn!(
                            "Failed to deserialize video metadata for course {}, using fallback: {}",
                            id, e
                        );
                        create_fallback_video_metadata(&raw_titles)
                    });

                validate_and_repair_loaded_metadata(parsed_videos, &raw_titles).unwrap_or_else(
                    |e| {
                        error!("Failed to repair loaded metadata: {}", e);
                        create_fallback_video_metadata(&raw_titles)
                    },
                )
            } else {
                info!(
                    "No video metadata found for course {}, creating from raw_titles",
                    id
                );
                create_fallback_video_metadata(&raw_titles)
            };

            let structure = structure_json
                .as_ref()
                .map(|json| serde_json::from_str::<CourseStructure>(json))
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

pub fn delete_course(db: &Database, course_id: &Uuid) -> Result<()> {
    let mut conn = db.get_conn().with_context(|| "Failed to get DB connection")?;

    let tx = conn.transaction()?;
    tx.execute("DELETE FROM plans WHERE course_id = ?1", params![course_id.to_string()])?;
    tx.execute("DELETE FROM courses WHERE id = ?1", params![course_id.to_string()])?;
    tx.commit()?;

    Ok(())
}
