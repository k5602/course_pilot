//! Repository implementations using Diesel.

use diesel::prelude::*;
use std::sync::Arc;

use super::connection::DbPool;
use super::models::{
    CourseRow, ExamRow, ModuleRow, NewCourse, NewExam, NewModule, NewVideo, VideoRow,
};
use crate::domain::{
    entities::{Course, Exam, Module, Video},
    ports::{CourseRepository, ExamRepository, ModuleRepository, RepositoryError, VideoRepository},
    value_objects::{CourseId, ExamId, ModuleId, PlaylistUrl, VideoId, YouTubeVideoId},
};
use crate::schema::{courses, exams, modules, videos};

/// SQLite-backed course repository.
pub struct SqliteCourseRepository {
    pool: Arc<DbPool>,
}

impl SqliteCourseRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl CourseRepository for SqliteCourseRepository {
    fn save(&self, course: &Course) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let new_course = NewCourse {
            id: &course.id().as_uuid().to_string(),
            name: course.name(),
            source_url: course.source_url().raw(),
            playlist_id: course.playlist_id(),
            description: course.description(),
        };

        diesel::insert_into(courses::table)
            .values(&new_course)
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_by_id(&self, id: &CourseId) -> Result<Option<Course>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let result = courses::table
            .find(id.as_uuid().to_string())
            .first::<CourseRow>(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match result {
            Some(row) => {
                let course_id = CourseId::from_uuid(
                    uuid::Uuid::parse_str(&row.id)
                        .map_err(|e| RepositoryError::Database(e.to_string()))?,
                );
                let playlist_url = PlaylistUrl::new(&row.source_url)
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;

                Ok(Some(Course::new(
                    course_id,
                    row.name,
                    playlist_url,
                    row.playlist_id,
                    row.description,
                )))
            },
            None => Ok(None),
        }
    }

    fn find_all(&self) -> Result<Vec<Course>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let rows: Vec<CourseRow> =
            courses::table.load(&mut conn).map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                let course_id = CourseId::from_uuid(
                    uuid::Uuid::parse_str(&row.id)
                        .map_err(|e| RepositoryError::Database(e.to_string()))?,
                );
                let playlist_url = PlaylistUrl::new(&row.source_url)
                    .map_err(|e| RepositoryError::Database(e.to_string()))?;

                Ok(Course::new(course_id, row.name, playlist_url, row.playlist_id, row.description))
            })
            .collect()
    }

    fn delete(&self, id: &CourseId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::delete(courses::table.find(id.as_uuid().to_string()))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

/// SQLite-backed module repository.
pub struct SqliteModuleRepository {
    pool: Arc<DbPool>,
}

impl SqliteModuleRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl ModuleRepository for SqliteModuleRepository {
    fn save(&self, module: &Module) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let new_module = NewModule {
            id: &module.id().as_uuid().to_string(),
            course_id: &module.course_id().as_uuid().to_string(),
            title: module.title(),
            sort_order: module.sort_order() as i32,
        };

        diesel::insert_into(modules::table)
            .values(&new_module)
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_by_id(&self, id: &ModuleId) -> Result<Option<Module>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let result = modules::table
            .find(id.as_uuid().to_string())
            .first::<ModuleRow>(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match result {
            Some(row) => {
                let module_id = ModuleId::from_uuid(
                    uuid::Uuid::parse_str(&row.id)
                        .map_err(|e| RepositoryError::Database(e.to_string()))?,
                );
                let course_id = CourseId::from_uuid(
                    uuid::Uuid::parse_str(&row.course_id)
                        .map_err(|e| RepositoryError::Database(e.to_string()))?,
                );
                Ok(Some(Module::new(module_id, course_id, row.title, row.sort_order as u32)))
            },
            None => Ok(None),
        }
    }

    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Module>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let rows: Vec<ModuleRow> = modules::table
            .filter(modules::course_id.eq(course_id.as_uuid().to_string()))
            .order(modules::sort_order.asc())
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter()
            .map(|row| {
                let module_id = ModuleId::from_uuid(
                    uuid::Uuid::parse_str(&row.id)
                        .map_err(|e| RepositoryError::Database(e.to_string()))?,
                );
                let cid = CourseId::from_uuid(
                    uuid::Uuid::parse_str(&row.course_id)
                        .map_err(|e| RepositoryError::Database(e.to_string()))?,
                );
                Ok(Module::new(module_id, cid, row.title, row.sort_order as u32))
            })
            .collect()
    }

    fn delete(&self, id: &ModuleId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::delete(modules::table.find(id.as_uuid().to_string()))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

/// SQLite-backed video repository.
pub struct SqliteVideoRepository {
    pool: Arc<DbPool>,
}

impl SqliteVideoRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl VideoRepository for SqliteVideoRepository {
    fn save(&self, video: &Video) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let new_video = NewVideo {
            id: &video.id().as_uuid().to_string(),
            module_id: &video.module_id().as_uuid().to_string(),
            youtube_id: video.youtube_id().as_str(),
            title: video.title(),
            duration_secs: video.duration_secs() as i32,
            is_completed: video.is_completed(),
            sort_order: video.sort_order() as i32,
        };

        diesel::insert_into(videos::table)
            .values(&new_video)
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_by_id(&self, id: &VideoId) -> Result<Option<Video>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let result = videos::table
            .find(id.as_uuid().to_string())
            .first::<VideoRow>(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match result {
            Some(row) => row_to_video(row).map(Some),
            None => Ok(None),
        }
    }

    fn find_by_module(&self, module_id: &ModuleId) -> Result<Vec<Video>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let rows: Vec<VideoRow> = videos::table
            .filter(videos::module_id.eq(module_id.as_uuid().to_string()))
            .order(videos::sort_order.asc())
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_video).collect()
    }

    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Video>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        // Join modules to get videos for a course
        let rows: Vec<VideoRow> = videos::table
            .inner_join(modules::table)
            .filter(modules::course_id.eq(course_id.as_uuid().to_string()))
            .select(VideoRow::as_select())
            .order((modules::sort_order.asc(), videos::sort_order.asc()))
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_video).collect()
    }

    fn update_completion(&self, id: &VideoId, completed: bool) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::update(videos::table.find(id.as_uuid().to_string()))
            .set(videos::is_completed.eq(completed))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: &VideoId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::delete(videos::table.find(id.as_uuid().to_string()))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

fn row_to_video(row: VideoRow) -> Result<Video, RepositoryError> {
    let video_id = VideoId::from_uuid(
        uuid::Uuid::parse_str(&row.id).map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let module_id = ModuleId::from_uuid(
        uuid::Uuid::parse_str(&row.module_id)
            .map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let youtube_id = YouTubeVideoId::new(&row.youtube_id)
        .map_err(|e| RepositoryError::Database(e.to_string()))?;

    let mut video = Video::new(
        video_id,
        module_id,
        youtube_id,
        row.title,
        row.duration_secs as u32,
        row.sort_order as u32,
    );
    if row.is_completed {
        video.mark_completed();
    }
    Ok(video)
}

/// SQLite-backed exam repository.
pub struct SqliteExamRepository {
    pool: Arc<DbPool>,
}

impl SqliteExamRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl ExamRepository for SqliteExamRepository {
    fn save(&self, exam: &Exam) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let new_exam = NewExam {
            id: &exam.id().as_uuid().to_string(),
            video_id: &exam.video_id().as_uuid().to_string(),
            question_json: exam.question_json(),
        };

        diesel::insert_into(exams::table)
            .values(&new_exam)
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_by_id(&self, id: &ExamId) -> Result<Option<Exam>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let result = exams::table
            .find(id.as_uuid().to_string())
            .first::<ExamRow>(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match result {
            Some(row) => row_to_exam(row).map(Some),
            None => Ok(None),
        }
    }

    fn find_by_video(&self, video_id: &VideoId) -> Result<Vec<Exam>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let rows: Vec<ExamRow> = exams::table
            .filter(exams::video_id.eq(video_id.as_uuid().to_string()))
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_exam).collect()
    }

    fn update_result(&self, id: &ExamId, score: f32, passed: bool) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        diesel::update(exams::table.find(id.as_uuid().to_string()))
            .set((exams::score.eq(score), exams::passed.eq(passed)))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

fn row_to_exam(row: ExamRow) -> Result<Exam, RepositoryError> {
    let exam_id = ExamId::from_uuid(
        uuid::Uuid::parse_str(&row.id).map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let video_id = VideoId::from_uuid(
        uuid::Uuid::parse_str(&row.video_id)
            .map_err(|e| RepositoryError::Database(e.to_string()))?,
    );

    let mut exam = Exam::new(exam_id, video_id, row.question_json);
    if let Some(score) = row.score {
        exam.record_result(score);
    }
    Ok(exam)
}
