//! Data loading hooks for courses and videos

use std::sync::Arc;

use dioxus::prelude::*;

use crate::application::AppContext;
use crate::domain::entities::{Course, Exam, Module, Video};
use crate::domain::ports::{CourseRepository, ExamRepository, ModuleRepository, VideoRepository};
use crate::domain::value_objects::{CourseId, ExamId, ModuleId, VideoId};

/// Load all courses from the database.
pub fn use_load_courses(backend: Option<Arc<AppContext>>) -> Signal<Vec<Course>> {
    let mut courses = use_signal(Vec::new);

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.course_repo.find_all() {
                Ok(loaded) => courses.set(loaded),
                Err(e) => log::error!("Failed to load courses: {}", e),
            }
        }
    });

    courses
}

/// Load modules for a specific course.
pub fn use_load_modules(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> Signal<Vec<Module>> {
    let mut modules = use_signal(Vec::new);
    let course_id = course_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.module_repo.find_by_course(&course_id) {
                Ok(loaded) => modules.set(loaded),
                Err(e) => log::error!("Failed to load modules: {}", e),
            }
        }
    });

    modules
}

/// Load videos for a specific module.
pub fn use_load_videos(
    backend: Option<Arc<AppContext>>,
    module_id: &ModuleId,
) -> Signal<Vec<Video>> {
    let mut videos = use_signal(Vec::new);
    let module_id = module_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.video_repo.find_by_module(&module_id) {
                Ok(loaded) => videos.set(loaded),
                Err(e) => log::error!("Failed to load videos: {}", e),
            }
        }
    });

    videos
}

/// Load a single course by ID.
pub fn use_load_course(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> Signal<Option<Course>> {
    let mut course = use_signal(|| None);
    let course_id = course_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.course_repo.find_by_id(&course_id) {
                Ok(loaded) => course.set(loaded),
                Err(e) => log::error!("Failed to load course: {}", e),
            }
        }
    });

    course
}

/// Load a single video by ID.
pub fn use_load_video(
    backend: Option<Arc<AppContext>>,
    video_id: &VideoId,
) -> Signal<Option<Video>> {
    let mut video = use_signal(|| None);
    let video_id = video_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.video_repo.find_by_id(&video_id) {
                Ok(loaded) => video.set(loaded),
                Err(e) => log::error!("Failed to load video: {}", e),
            }
        }
    });

    video
}

/// Load a single exam by ID.
pub fn use_load_exam(backend: Option<Arc<AppContext>>, exam_id: &ExamId) -> Signal<Option<Exam>> {
    let mut exam = use_signal(|| None);
    let exam_id = exam_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.exam_repo.find_by_id(&exam_id) {
                Ok(loaded) => exam.set(loaded),
                Err(e) => log::error!("Failed to load exam: {}", e),
            }
        }
    });

    exam
}

/// Load exams for a specific video.
pub fn use_load_exams(backend: Option<Arc<AppContext>>, video_id: &VideoId) -> Signal<Vec<Exam>> {
    let mut exams = use_signal(Vec::new);
    let video_id = video_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.exam_repo.find_by_video(&video_id) {
                Ok(loaded) => exams.set(loaded),
                Err(e) => log::error!("Failed to load exams: {}", e),
            }
        }
    });

    exams
}

/// Load all exams from the database.
pub fn use_load_all_exams(backend: Option<Arc<AppContext>>) -> Signal<Vec<Exam>> {
    let mut exams = use_signal(Vec::new);

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.exam_repo.find_all() {
                Ok(loaded) => exams.set(loaded),
                Err(e) => log::error!("Failed to load exams: {}", e),
            }
        }
    });

    exams
}
