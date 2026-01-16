//! SQLite Tag Repository implementation.

use std::str::FromStr;
use std::sync::Arc;

use diesel::prelude::*;

use crate::domain::entities::Tag;
use crate::domain::ports::{RepositoryError, TagRepository};
use crate::domain::value_objects::{CourseId, TagId};
use crate::infrastructure::persistence::DbPool;
use crate::infrastructure::persistence::models::{CourseTagRow, NewTag, TagRow};
use crate::schema::{course_tags, tags};

/// SQLite implementation of TagRepository.
pub struct SqliteTagRepository {
    pool: Arc<DbPool>,
}

impl SqliteTagRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn row_to_entity(row: TagRow) -> Result<Tag, RepositoryError> {
        let id = TagId::from_str(&row.id)
            .map_err(|e| RepositoryError::Database(format!("Invalid tag ID: {}", e)))?;
        Ok(Tag::with_color(id, row.name, row.color))
    }
}

impl TagRepository for SqliteTagRepository {
    fn save(&self, tag: &Tag) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let new_tag =
            NewTag { id: &tag.id().as_uuid().to_string(), name: tag.name(), color: tag.color() };

        diesel::insert_into(tags::table)
            .values(&new_tag)
            .on_conflict(tags::id)
            .do_update()
            .set((tags::name.eq(tag.name()), tags::color.eq(tag.color())))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_all(&self) -> Result<Vec<Tag>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let rows: Vec<TagRow> = tags::table
            .order(tags::name.asc())
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Tag>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let rows: Vec<TagRow> = tags::table
            .inner_join(course_tags::table.on(course_tags::tag_id.eq(tags::id)))
            .filter(course_tags::course_id.eq(course_id.as_uuid().to_string()))
            .select(TagRow::as_select())
            .order(tags::name.asc())
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    fn add_to_course(&self, course_id: &CourseId, tag_id: &TagId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let link = CourseTagRow {
            course_id: course_id.as_uuid().to_string(),
            tag_id: tag_id.as_uuid().to_string(),
        };

        diesel::insert_into(course_tags::table)
            .values(&link)
            .on_conflict_do_nothing()
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn remove_from_course(
        &self,
        course_id: &CourseId,
        tag_id: &TagId,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::delete(
            course_tags::table
                .filter(course_tags::course_id.eq(course_id.as_uuid().to_string()))
                .filter(course_tags::tag_id.eq(tag_id.as_uuid().to_string())),
        )
        .execute(&mut conn)
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, tag_id: &TagId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        // Course tags will be deleted via CASCADE
        diesel::delete(tags::table.filter(tags::id.eq(tag_id.as_uuid().to_string())))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}
