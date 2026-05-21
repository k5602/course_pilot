//! implementation of domain repositories using Diesel.

use std::sync::Arc;

use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::prelude::*;

use crate::domain::{
    entities::{Course, Exam, Module, Note, NoteId, Video},
    ports::{
        CourseRepository, ExamRepository, ModuleRepository, NoteRepository, RepositoryError,
        VideoRepository,
    },
    value_objects::{
        CourseId, ExamId, ModuleId, PlaylistUrl, VideoId, VideoSource, YouTubeVideoId,
    },
};
use crate::infrastructure::persistence::connection::DbPool;
use crate::infrastructure::persistence::models::*;
use crate::schema::{courses, exams, modules, notes, videos};

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

        let id_str = course.id().as_uuid().to_string();
        let new_course = NewCourse {
            id: &id_str,
            name: course.name(),
            source_url: course.source_url().raw(),
            playlist_id: course.playlist_id(),
            description: course.description(),
            source_hash: course.source_hash(),
        };

        diesel::insert_into(courses::table)
            .values(&new_course)
            .on_conflict(courses::id)
            .do_update()
            .set((
                courses::name.eq(new_course.name),
                courses::description.eq(new_course.description),
            ))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn save_batch(&self, courses: &[Course]) -> Result<(), RepositoryError> {
        if courses.is_empty() {
            return Ok(());
        }
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;
        conn.transaction::<_, diesel::result::Error, _>(|tx| {
            for course in courses {
                let id_str = course.id().as_uuid().to_string();
                let new_course = NewCourse {
                    id: &id_str,
                    name: course.name(),
                    source_url: course.source_url().raw(),
                    playlist_id: course.playlist_id(),
                    description: course.description(),
                    source_hash: course.source_hash(),
                };
                diesel::insert_into(courses::table)
                    .values(&new_course)
                    .on_conflict(courses::id)
                    .do_update()
                    .set((
                        courses::name.eq(new_course.name),
                        courses::description.eq(new_course.description),
                    ))
                    .execute(tx)?;
            }
            Ok(())
        })
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    fn find_by_id(&self, id: &CourseId) -> Result<Option<Course>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        let row: Option<CourseRow> = courses::table
            .find(&id_str)
            .first(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_course(r)?)),
            None => Ok(None),
        }
    }

    fn find_by_source_hash(&self, hash: &str) -> Result<Option<Course>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let row: Option<CourseRow> = courses::table
            .filter(courses::source_hash.eq(Some(hash)))
            .first(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_course(r)?)),
            None => Ok(None),
        }
    }

    fn find_all(&self) -> Result<Vec<Course>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let rows: Vec<CourseRow> =
            courses::table.load(&mut conn).map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_course).collect()
    }

    fn update_metadata(
        &self,
        id: &CourseId,
        name: &str,
        description: Option<&str>,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::update(courses::table.find(&id_str))
            .set((courses::name.eq(name), courses::description.eq(description)))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: &CourseId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::delete(courses::table.find(&id_str))
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

        let id_str = module.id().as_uuid().to_string();
        let course_id_str = module.course_id().as_uuid().to_string();
        let new_module = NewModule {
            id: &id_str,
            course_id: &course_id_str,
            title: module.title(),
            sort_order: module.sort_order() as i32,
        };

        diesel::insert_into(modules::table)
            .values(&new_module)
            .on_conflict(modules::id)
            .do_update()
            .set((
                modules::title.eq(new_module.title),
                modules::sort_order.eq(new_module.sort_order),
            ))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn save_batch(&self, modules: &[Module]) -> Result<(), RepositoryError> {
        if modules.is_empty() {
            return Ok(());
        }
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;
        conn.transaction::<_, diesel::result::Error, _>(|tx| {
            for module in modules {
                let id_str = module.id().as_uuid().to_string();
                let course_id_str = module.course_id().as_uuid().to_string();
                let new_module = NewModule {
                    id: &id_str,
                    course_id: &course_id_str,
                    title: module.title(),
                    sort_order: module.sort_order() as i32,
                };
                diesel::insert_into(modules::table)
                    .values(&new_module)
                    .on_conflict(modules::id)
                    .do_update()
                    .set((
                        modules::title.eq(new_module.title),
                        modules::sort_order.eq(new_module.sort_order),
                    ))
                    .execute(tx)?;
            }
            Ok(())
        })
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    fn find_by_id(&self, id: &ModuleId) -> Result<Option<Module>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        let row: Option<ModuleRow> = modules::table
            .find(&id_str)
            .first(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_module(r)?)),
            None => Ok(None),
        }
    }

    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Module>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let course_id_str = course_id.as_uuid().to_string();
        let rows: Vec<ModuleRow> = modules::table
            .filter(modules::course_id.eq(&course_id_str))
            .order(modules::sort_order.asc())
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_module).collect()
    }

    fn update_title(&self, id: &ModuleId, title: &str) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::update(modules::table.find(&id_str))
            .set(modules::title.eq(title))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn delete(&self, id: &ModuleId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::delete(modules::table.find(&id_str))
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

        let (source_type, source_ref, youtube_id) = match video.source() {
            VideoSource::YouTube(id) => ("youtube", id.as_str().to_string(), Some(id.as_str())),
            VideoSource::LocalPath(path) => ("local", path.clone(), None),
        };

        let video_id_str = video.id().as_uuid().to_string();
        let module_id_str = video.module_id().as_uuid().to_string();
        let new_video = NewVideo {
            id: &video_id_str,
            module_id: &module_id_str,
            youtube_id,
            title: video.title(),
            duration_secs: video.duration_secs() as i32,
            is_completed: video.is_completed(),
            sort_order: video.sort_order() as i32,
            description: video.description(),
            transcript: video.transcript(),
            summary: video.summary(),
            source_type,
            source_ref: &source_ref,
            key_points: None,
            key_terms: None,
        };

        diesel::insert_into(videos::table)
            .values(&new_video)
            .on_conflict(videos::id)
            .do_update()
            .set((
                videos::title.eq(new_video.title),
                videos::duration_secs.eq(new_video.duration_secs),
                videos::is_completed.eq(new_video.is_completed),
                videos::sort_order.eq(new_video.sort_order),
                videos::description.eq(new_video.description),
                videos::transcript.eq(new_video.transcript),
                videos::summary.eq(new_video.summary),
                videos::module_id.eq(&module_id_str),
            ))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn save_batch(&self, videos: &[Video]) -> Result<(), RepositoryError> {
        if videos.is_empty() {
            return Ok(());
        }
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;
        conn.transaction::<_, diesel::result::Error, _>(|tx| {
            for video in videos {
                let (source_type, source_ref, youtube_id) = match video.source() {
                    VideoSource::YouTube(id) => {
                        ("youtube", id.as_str().to_string(), Some(id.as_str()))
                    },
                    VideoSource::LocalPath(path) => ("local", path.clone(), None),
                };

                let video_id_str = video.id().as_uuid().to_string();
                let module_id_str = video.module_id().as_uuid().to_string();
                let new_video = NewVideo {
                    id: &video_id_str,
                    module_id: &module_id_str,
                    youtube_id,
                    title: video.title(),
                    duration_secs: video.duration_secs() as i32,
                    is_completed: video.is_completed(),
                    sort_order: video.sort_order() as i32,
                    description: video.description(),
                    transcript: video.transcript(),
                    summary: video.summary(),
                    source_type,
                    source_ref: &source_ref,
                    key_points: None,
                    key_terms: None,
                };

                diesel::insert_into(videos::table)
                    .values(&new_video)
                    .on_conflict(videos::id)
                    .do_update()
                    .set((
                        videos::title.eq(new_video.title),
                        videos::duration_secs.eq(new_video.duration_secs),
                        videos::is_completed.eq(new_video.is_completed),
                        videos::sort_order.eq(new_video.sort_order),
                        videos::description.eq(new_video.description),
                        videos::transcript.eq(new_video.transcript),
                        videos::summary.eq(new_video.summary),
                        videos::module_id.eq(&module_id_str),
                    ))
                    .execute(tx)?;
            }
            Ok(())
        })
        .map_err(|e| RepositoryError::Database(e.to_string()))
    }

    fn find_by_id(&self, id: &VideoId) -> Result<Option<Video>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        let row: Option<VideoRow> = videos::table
            .find(&id_str)
            .first(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_video(r)?)),
            None => Ok(None),
        }
    }

    fn find_by_module(&self, module_id: &ModuleId) -> Result<Vec<Video>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let module_id_str = module_id.as_uuid().to_string();
        let rows: Vec<VideoRow> = videos::table
            .filter(videos::module_id.eq(&module_id_str))
            .order(videos::sort_order.asc())
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_video).collect()
    }

    fn find_by_course(&self, course_id: &CourseId) -> Result<Vec<Video>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let course_id_str = course_id.as_uuid().to_string();
        let rows: Vec<VideoRow> = videos::table
            .inner_join(modules::table)
            .filter(modules::course_id.eq(&course_id_str))
            .select(VideoRow::as_select())
            .order((modules::sort_order.asc(), videos::sort_order.asc()))
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_video).collect()
    }

    fn update_completion(&self, id: &VideoId, completed: bool) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::update(videos::table.find(&id_str))
            .set(videos::is_completed.eq(completed))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn update_transcript(
        &self,
        id: &VideoId,
        transcript: Option<&str>,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::update(videos::table.find(&id_str))
            .set(videos::transcript.eq(transcript))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn update_summary(&self, id: &VideoId, summary: Option<&str>) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::update(videos::table.find(&id_str))
            .set(videos::summary.eq(summary))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn update_module(
        &self,
        id: &VideoId,
        module_id: &ModuleId,
        sort_order: u32,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        let module_id_str = module_id.as_uuid().to_string();
        diesel::update(videos::table.find(&id_str))
            .set((videos::module_id.eq(&module_id_str), videos::sort_order.eq(sort_order as i32)))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn swap_video_orders(
        &self,
        video_a_id: &VideoId,
        video_b_id: &VideoId,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let a_str = video_a_id.as_uuid().to_string();
        let b_str = video_b_id.as_uuid().to_string();

        conn.transaction(|conn| {
            let a_row: (i32, String) = videos::table
                .find(&a_str)
                .select((videos::sort_order, videos::module_id))
                .first(conn)?;

            let b_row: (i32, String) = videos::table
                .find(&b_str)
                .select((videos::sort_order, videos::module_id))
                .first(conn)?;

            diesel::update(videos::table.find(&a_str))
                .set(videos::sort_order.eq(b_row.0))
                .execute(conn)?;

            diesel::update(videos::table.find(&b_str))
                .set(videos::sort_order.eq(a_row.0))
                .execute(conn)?;

            Ok(())
        })
        .map_err(|e: diesel::result::Error| RepositoryError::Database(e.to_string()))
    }

    fn delete(&self, id: &VideoId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::delete(videos::table.find(&id_str))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
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

        let id_str = exam.id().as_uuid().to_string();
        let video_id_str = exam.video_id().as_uuid().to_string();
        let new_exam = NewExam {
            id: &id_str,
            video_id: &video_id_str,
            question_json: exam.question_json(),
            user_answers_json: exam.user_answers_json(),
        };

        diesel::insert_into(exams::table)
            .values(&new_exam)
            .on_conflict(exams::id)
            .do_update()
            .set((
                exams::question_json.eq(new_exam.question_json),
                exams::user_answers_json.eq(new_exam.user_answers_json),
            ))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_by_id(&self, id: &ExamId) -> Result<Option<Exam>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        let row: Option<ExamRow> = exams::table
            .find(&id_str)
            .first(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_exam(r)?)),
            None => Ok(None),
        }
    }

    fn find_all(&self) -> Result<Vec<Exam>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;
        let rows: Vec<ExamRow> =
            exams::table.load(&mut conn).map_err(|e| RepositoryError::Database(e.to_string()))?;
        rows.into_iter().map(row_to_exam).collect()
    }

    fn find_by_video(&self, video_id: &VideoId) -> Result<Vec<Exam>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let video_id_str = video_id.as_uuid().to_string();
        let rows: Vec<ExamRow> = exams::table
            .filter(exams::video_id.eq(&video_id_str))
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(row_to_exam).collect()
    }

    fn update_result(
        &self,
        id: &ExamId,
        score: f32,
        passed: bool,
        user_answers_json: Option<String>,
    ) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = id.as_uuid().to_string();
        diesel::update(exams::table.find(&id_str))
            .set((
                exams::score.eq(score),
                exams::passed.eq(passed),
                exams::user_answers_json.eq(user_answers_json),
            ))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

/// SQLite-backed note repository.
pub struct SqliteNoteRepository {
    pool: Arc<DbPool>,
}

impl SqliteNoteRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }
}

impl NoteRepository for SqliteNoteRepository {
    fn save(&self, note: &Note) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let id_str = note.id().as_uuid().to_string();
        let video_id_str = note.video_id().as_uuid().to_string();
        let new_note = NewNote { id: &id_str, video_id: &video_id_str, content: note.content() };

        diesel::insert_into(notes::table)
            .values(&new_note)
            .on_conflict(notes::id)
            .do_update()
            .set(notes::content.eq(new_note.content))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_by_video(&self, video_id: &VideoId) -> Result<Option<Note>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let video_id_str = video_id.as_uuid().to_string();
        let row: Option<NoteRow> = notes::table
            .filter(notes::video_id.eq(&video_id_str))
            .first(&mut conn)
            .optional()
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        match row {
            Some(r) => Ok(Some(row_to_note(r)?)),
            None => Ok(None),
        }
    }

    fn delete(&self, video_id: &VideoId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;

        let video_id_str = video_id.as_uuid().to_string();
        diesel::delete(notes::table.filter(notes::video_id.eq(&video_id_str)))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }
}

// --- Mappings ---

fn row_to_course(row: CourseRow) -> Result<Course, RepositoryError> {
    let course_id = CourseId::from_uuid(
        uuid::Uuid::parse_str(&row.id).map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let playlist_url =
        PlaylistUrl::new(&row.source_url).map_err(|e| RepositoryError::Database(e.to_string()))?;

    let created_at = parse_sqlite_timestamp(&row.created_at)?;

    Ok(Course::new_with_created_at(
        course_id,
        row.name,
        playlist_url,
        row.playlist_id,
        row.description,
        row.source_hash,
        created_at,
    ))
}

fn row_to_module(row: ModuleRow) -> Result<Module, RepositoryError> {
    let module_id = ModuleId::from_uuid(
        uuid::Uuid::parse_str(&row.id).map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let course_id = CourseId::from_uuid(
        uuid::Uuid::parse_str(&row.course_id)
            .map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let sort_order = i32_to_u32(row.sort_order, "sort_order")?;

    Ok(Module::new(module_id, course_id, row.title, sort_order))
}

fn row_to_video(row: VideoRow) -> Result<Video, RepositoryError> {
    let video_id = VideoId::from_uuid(
        uuid::Uuid::parse_str(&row.id).map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let module_id = ModuleId::from_uuid(
        uuid::Uuid::parse_str(&row.module_id)
            .map_err(|e| RepositoryError::Database(e.to_string()))?,
    );

    let source = match row.source_type.as_str() {
        "youtube" => {
            let youtube_id = YouTubeVideoId::new(&row.source_ref)
                .map_err(|e| RepositoryError::Database(e.to_string()))?;
            VideoSource::YouTube(youtube_id)
        },
        "local" => VideoSource::LocalPath(row.source_ref.clone()),
        other => {
            return Err(RepositoryError::Database(format!("Invalid video source type: {other}")));
        },
    };

    let duration_secs = i32_to_u32(row.duration_secs, "duration_secs")?;
    let sort_order = i32_to_u32(row.sort_order, "sort_order")?;

    let mut video = Video::with_description(
        video_id,
        module_id,
        source,
        row.title,
        row.description,
        duration_secs,
        sort_order,
    );
    video.update_transcript(row.transcript);
    video.update_summary(row.summary);
    if row.is_completed {
        video.mark_completed();
    }
    Ok(video)
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
        exam.record_result(score, row.user_answers_json);
    }
    Ok(exam)
}

fn row_to_note(row: NoteRow) -> Result<Note, RepositoryError> {
    let note_id = NoteId::from_uuid(
        uuid::Uuid::parse_str(&row.id).map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    let video_id = VideoId::from_uuid(
        uuid::Uuid::parse_str(&row.video_id)
            .map_err(|e| RepositoryError::Database(e.to_string()))?,
    );
    Ok(Note::new(note_id, video_id, row.content))
}

// --- Internal Helpers ---

fn i32_to_u32(value: i32, field: &str) -> Result<u32, RepositoryError> {
    u32::try_from(value)
        .map_err(|_| RepositoryError::Database(format!("Invalid value for {field}: {value}")))
}

fn parse_sqlite_timestamp(ts: &str) -> Result<DateTime<Utc>, RepositoryError> {
    // SQLite timestamps in Diesel are typically "YYYY-MM-DD HH:MM:SS"
    NaiveDateTime::parse_from_str(ts, "%Y-%m-%d %H:%M:%S")
        .map(|dt| DateTime::<Utc>::from_naive_utc_and_offset(dt, Utc))
        .or_else(|_| {
            // Fallback for full ISO strings if any
            ts.parse::<DateTime<Utc>>()
        })
        .map_err(|e| RepositoryError::Database(format!("Failed to parse timestamp {ts}: {e}")))
}
