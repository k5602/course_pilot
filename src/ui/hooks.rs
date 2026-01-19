//! Data loading hooks for courses and videos

use std::sync::Arc;

use dioxus::prelude::*;

use crate::application::{AppContext, ServiceFactory};
use crate::domain::entities::{AppAnalytics, Course, Exam, Module, Video};
use crate::domain::ports::{
    Activity, CourseRepository, ExamRepository, ModuleRepository, VideoRepository,
};
use crate::domain::value_objects::{CourseId, ExamId, ModuleId, VideoId};
use crate::ui::routes::Route;
use crate::ui::state::AppState;

/// Load state for data hooks.
#[derive(Clone)]
pub struct LoadState {
    pub is_loading: Signal<bool>,
    pub error: Signal<Option<String>>,
}

/// Initialize loading and error signals for a hook.
pub fn use_load_state() -> LoadState {
    LoadState { is_loading: use_signal(|| false), error: use_signal(|| None) }
}

/// Load dashboard analytics with loading and error state.
pub fn use_load_dashboard_analytics(
    backend: Option<Arc<AppContext>>,
) -> (Signal<Option<AppAnalytics>>, LoadState) {
    let mut analytics = use_signal(|| None);
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;

    use_effect(move || {
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => {
                let use_case = ServiceFactory::dashboard(ctx);
                match use_case.execute() {
                    Ok(snapshot) => analytics.set(Some(snapshot)),
                    Err(e) => error.set(Some(format!("Failed to load analytics: {}", e))),
                }
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (analytics, load_state)
}

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

/// Load all courses with loading and error state.
pub fn use_load_courses_state(
    backend: Option<Arc<AppContext>>,
) -> (Signal<Vec<Course>>, LoadState) {
    let mut courses = use_signal(Vec::new);
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;

    use_effect(move || {
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => match ctx.course_repo.find_all() {
                Ok(loaded) => courses.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load courses: {}", e))),
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (courses, load_state)
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

/// Load modules with loading and error state.
pub fn use_load_modules_state(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> (Signal<Vec<Module>>, LoadState) {
    let mut modules = use_signal(Vec::new);
    let course_id = course_id.clone();
    let mut course_id_signal = use_signal(|| course_id.clone());
    if *course_id_signal.read() != course_id {
        course_id_signal.set(course_id.clone());
    }
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;
    let mut last_course_id = use_signal(|| course_id.clone());
    if *last_course_id.read() != course_id {
        last_course_id.set(course_id.clone());
        modules.set(Vec::new());
        error.set(None);
        is_loading.set(true);
    }

    use_effect(move || {
        let course_id = course_id_signal.read().clone();
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => match ctx.module_repo.find_by_course(&course_id) {
                Ok(loaded) => modules.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load modules: {}", e))),
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (modules, load_state)
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

/// Load a single course with loading and error state.
pub fn use_load_course_state(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> (Signal<Option<Course>>, LoadState) {
    let mut course = use_signal(|| None);
    let course_id = course_id.clone();
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;

    use_effect(move || {
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => match ctx.course_repo.find_by_id(&course_id) {
                Ok(loaded) => course.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load course: {}", e))),
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (course, load_state)
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

/// Load a single video with loading and error state.
pub fn use_load_video_state(
    backend: Option<Arc<AppContext>>,
    video_id: &VideoId,
) -> (Signal<Option<Video>>, LoadState) {
    let mut video = use_signal(|| None);
    let video_id = video_id.clone();
    let mut video_id_signal = use_signal(|| video_id.clone());
    if *video_id_signal.read() != video_id {
        video_id_signal.set(video_id.clone());
    }
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;
    let mut last_video_id = use_signal(|| video_id.clone());
    if *last_video_id.read() != video_id {
        last_video_id.set(video_id.clone());
        video.set(None);
        error.set(None);
        is_loading.set(true);
    }

    use_effect(move || {
        let video_id = video_id_signal.read().clone();
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => match ctx.video_repo.find_by_id(&video_id) {
                Ok(loaded) => video.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load video: {}", e))),
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (video, load_state)
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

/// Load a single exam with loading and error state.
pub fn use_load_exam_state(
    backend: Option<Arc<AppContext>>,
    exam_id: &ExamId,
) -> (Signal<Option<Exam>>, LoadState) {
    let mut exam = use_signal(|| None);
    let exam_id = exam_id.clone();
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;

    use_effect(move || {
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => match ctx.exam_repo.find_by_id(&exam_id) {
                Ok(loaded) => exam.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load exam: {}", e))),
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (exam, load_state)
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

/// Load all exams with loading and error state.
pub fn use_load_all_exams_state(
    backend: Option<Arc<AppContext>>,
) -> (Signal<Vec<Exam>>, LoadState) {
    let mut exams = use_signal(Vec::new);
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;

    use_effect(move || {
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => match ctx.exam_repo.find_all() {
                Ok(loaded) => exams.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load exams: {}", e))),
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (exams, load_state)
}

/// Load all videos for a specific course (across all modules).
pub fn use_load_videos_by_course(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> Signal<Vec<Video>> {
    let mut videos = use_signal(Vec::new);
    let course_id = course_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            match ctx.video_repo.find_by_course(&course_id) {
                Ok(loaded) => videos.set(loaded),
                Err(e) => log::error!("Failed to load videos for course: {}", e),
            }
        }
    });

    videos
}

/// Load all videos by course with loading and error state.
pub fn use_load_videos_by_course_state(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> (Signal<Vec<Video>>, LoadState) {
    let mut videos = use_signal(Vec::new);
    let course_id = course_id.clone();
    let mut course_id_signal = use_signal(|| course_id.clone());
    if *course_id_signal.read() != course_id {
        course_id_signal.set(course_id.clone());
    }
    let load_state = use_load_state();
    let mut is_loading = load_state.is_loading;
    let mut error = load_state.error;
    let mut last_course_id = use_signal(|| course_id.clone());
    if *last_course_id.read() != course_id {
        last_course_id.set(course_id.clone());
        videos.set(Vec::new());
        error.set(None);
        is_loading.set(true);
    }

    use_effect(move || {
        let course_id = course_id_signal.read().clone();
        is_loading.set(true);
        error.set(None);

        match backend.as_ref() {
            Some(ctx) => match ctx.video_repo.find_by_course(&course_id) {
                Ok(loaded) => videos.set(loaded),
                Err(e) => error.set(Some(format!("Failed to load videos for course: {}", e))),
            },
            None => error.set(Some("Backend not available".to_string())),
        }

        is_loading.set(false);
    });

    (videos, load_state)
}

/// Load all tags from the database.
pub fn use_load_tags(
    backend: Option<Arc<AppContext>>,
) -> Signal<Vec<crate::domain::entities::Tag>> {
    let mut tags = use_signal(Vec::new);

    use_effect(move || {
        if let Some(ref ctx) = backend {
            use crate::domain::ports::TagRepository;
            match ctx.tag_repo.find_all() {
                Ok(loaded) => tags.set(loaded),
                Err(e) => log::error!("Failed to load tags: {}", e),
            }
        }
    });

    tags
}

/// Load tags for a specific course.
pub fn use_load_course_tags(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> Signal<Vec<crate::domain::entities::Tag>> {
    let mut tags = use_signal(Vec::new);
    let course_id = course_id.clone();

    use_effect(move || {
        if let Some(ref ctx) = backend {
            use crate::domain::ports::TagRepository;
            match ctx.tag_repo.find_by_course(&course_id) {
                Ok(loaded) => tags.set(loaded),
                Err(e) => log::error!("Failed to load course tags: {}", e),
            }
        }
    });

    tags
}

/// Search across courses, videos, and notes.
pub fn use_search(
    backend: Option<Arc<AppContext>>,
    query: String,
) -> Signal<Vec<crate::domain::entities::SearchResult>> {
    let mut results = use_signal(Vec::new);

    use_effect(move || {
        if query.trim().is_empty() {
            results.set(Vec::new());
            return;
        }

        if let Some(ref ctx) = backend {
            use crate::domain::ports::SearchRepository;
            match ctx.search_repo.search(&query, 20) {
                Ok(loaded) => results.set(loaded),
                Err(e) => log::error!("Search failed: {}", e),
            }
        }
    });

    results
}

/// Hook to synchronize the application state with Discord Rich Presence.
/// Monitors route and state changes to update the user's status.
pub fn use_presence_sync(backend: Option<Arc<AppContext>>) {
    let route = use_route::<Route>();
    let state = use_context::<AppState>();

    use_effect(move || {
        let backend = match backend.as_ref() {
            Some(b) => b,
            None => return,
        };

        let activity = match route {
            Route::Dashboard {} => Activity::Dashboard,
            Route::CourseList {} => Activity::BrowsingCourses,
            Route::CourseView { .. } => Activity::BrowsingCourses,
            Route::VideoPlayer { .. } => {
                let course = state.current_course.read();
                let video_id = state.current_video_id.read();
                let videos = state.current_videos.read();

                let video = video_id
                    .as_ref()
                    .and_then(|id| videos.iter().find(|v| v.id().to_string() == *id));

                if let (Some(c), Some(v)) = (course.as_ref(), video) {
                    Activity::Watching {
                        course_title: c.name().to_string(),
                        video_title: v.title().to_string(),
                    }
                } else {
                    Activity::BrowsingCourses
                }
            },
            Route::QuizList {} => Activity::BrowsingCourses,
            Route::QuizView { .. } => {
                let course = state.current_course.read();
                if let Some(c) = course.as_ref() {
                    Activity::TakingExam {
                        course_title: c.name().to_string(),
                        exam_title: "Active Quiz".to_string(),
                    }
                } else {
                    Activity::BrowsingCourses
                }
            },
            Route::Settings {} => Activity::Settings,
        };

        backend.presence.update_activity(activity);
    });
}
