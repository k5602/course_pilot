//! SQLite Search Repository implementation using FTS5.

use std::str::FromStr;
use std::sync::Arc;

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;

use crate::domain::entities::{SearchResult, SearchResultType};
use crate::domain::ports::{RepositoryError, SearchRepository};
use crate::domain::value_objects::CourseId;
use crate::infrastructure::persistence::DbPool;

/// SQLite FTS5 implementation of SearchRepository.
pub struct SqliteSearchRepository {
    pool: Arc<DbPool>,
}

impl SqliteSearchRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

#[derive(QueryableByName, Debug)]
struct SearchRow {
    #[diesel(sql_type = Text)]
    entity_type: String,
    #[diesel(sql_type = Text)]
    entity_id: String,
    #[diesel(sql_type = Text)]
    title: String,
    #[diesel(sql_type = Text)]
    content: String,
    #[diesel(sql_type = Text)]
    course_id: String,
}

impl SearchRepository for SqliteSearchRepository {
    fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        // FTS5 match query with snippet extraction
        let search_query = format!("{}*", query.replace('"', ""));

        let rows: Vec<SearchRow> = sql_query(
            r#"
            SELECT entity_type, entity_id, title, 
                   snippet(search_index, 3, '<b>', '</b>', '...', 20) as content,
                   course_id
            FROM search_index
            WHERE search_index MATCH ?
            ORDER BY rank
            LIMIT ?
            "#,
        )
        .bind::<Text, _>(&search_query)
        .bind::<diesel::sql_types::Integer, _>(limit as i32)
        .load(&mut conn)
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                let entity_type = match row.entity_type.as_str() {
                    "course" => SearchResultType::Course,
                    "video" => SearchResultType::Video,
                    "note" => SearchResultType::Note,
                    _ => SearchResultType::Video,
                };

                let course_id = CourseId::from_str(&row.course_id)
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;

                Ok(SearchResult::new(entity_type, row.entity_id, row.title, row.content, course_id))
            })
            .collect()
    }

    fn index_course(
        &self,
        course_id: &CourseId,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = course_id.as_uuid().to_string();
        let desc = description.unwrap_or("");

        sql_query(
            "INSERT INTO search_index (entity_type, entity_id, title, content, course_id) VALUES ('course', ?, ?, ?, ?)"
        )
        .bind::<Text, _>(&id_str)
        .bind::<Text, _>(name)
        .bind::<Text, _>(desc)
        .bind::<Text, _>(&id_str)
        .execute(&mut conn)
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn index_video(
        &self,
        video_id: &str,
        title: &str,
        description: Option<&str>,
        course_id: &CourseId,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let desc = description.unwrap_or("");
        let course_id_str = course_id.as_uuid().to_string();

        sql_query(
            "INSERT INTO search_index (entity_type, entity_id, title, content, course_id) VALUES ('video', ?, ?, ?, ?)"
        )
        .bind::<Text, _>(video_id)
        .bind::<Text, _>(title)
        .bind::<Text, _>(desc)
        .bind::<Text, _>(&course_id_str)
        .execute(&mut conn)
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn index_note(
        &self,
        note_id: &str,
        video_title: &str,
        content: &str,
        course_id: &CourseId,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let course_id_str = course_id.as_uuid().to_string();

        sql_query(
            "INSERT INTO search_index (entity_type, entity_id, title, content, course_id) VALUES ('note', ?, ?, ?, ?)"
        )
        .bind::<Text, _>(note_id)
        .bind::<Text, _>(video_title)
        .bind::<Text, _>(content)
        .bind::<Text, _>(&course_id_str)
        .execute(&mut conn)
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn remove_from_index(&self, entity_id: &str) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        sql_query("DELETE FROM search_index WHERE entity_id = ?")
            .bind::<Text, _>(entity_id)
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}
