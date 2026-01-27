//! Data loading hooks for courses and videos (async, debounced, unified signatures)

use std::sync::Arc;
use std::time::Duration;

use dioxus::prelude::*;

use crate::application::{AppContext, ServiceFactory};
use crate::domain::entities::{AppAnalytics, Course, Exam, Module, SearchResult, Tag, Video};
use crate::domain::ports::{
    Activity, CourseRepository, ExamRepository, ModuleRepository, SearchRepository, TagRepository,
    VideoRepository,
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

/// Result wrapper for load hooks.
#[derive(Clone)]
pub struct LoadResult<T> {
    pub data: Signal<T>,
    pub state: LoadState,
}

/// Initialize loading and error signals for a hook.
pub fn use_load_state() -> LoadState {
    LoadState { is_loading: use_signal(|| false), error: use_signal(|| None) }
}

fn backend_key(backend: &Option<Arc<AppContext>>) -> String {
    backend
        .as_ref()
        .map(|ctx| format!("{:p}", Arc::as_ptr(ctx)))
        .unwrap_or_else(|| "none".to_string())
}

fn use_keyed_effect<K>(key: K, mut effect: impl FnMut(K) + 'static)
where
    K: PartialEq + Clone + 'static,
{
    let mut last_key = use_signal(|| None::<K>);
    let mut key_signal = use_signal(|| key.clone());
    if *key_signal.read() != key {
        key_signal.set(key.clone());
    }

    use_effect(move || {
        let current = key_signal.read().clone();
        let should_run = last_key.read().as_ref().map(|k| k != &current).unwrap_or(true);
        if should_run {
            last_key.set(Some(current.clone()));
            effect(current);
        }
    });
}

type Loader<T> = Arc<dyn Fn(Arc<AppContext>) -> Result<T, String> + Send + Sync>;

fn use_async_loader<T, K>(
    backend: Option<Arc<AppContext>>,
    key: K,
    loader: Loader<T>,
) -> LoadResult<T>
where
    T: Default + Send + 'static,
    K: PartialEq + Clone + 'static,
{
    let data = use_signal(T::default);
    let state = use_load_state();
    let is_loading = state.is_loading;
    let error = state.error;
    let mut request_id = use_signal(|| 0u64);
    let backend_slot = use_signal(|| backend.clone());
    let mut backend_slot_for_effect = backend_slot;
    let backend_for_effect = backend.clone();

    let loader_for_future = loader.clone();
    let mut future = use_future(move || {
        let backend = backend_slot.read().clone();
        let loader = loader_for_future.clone();
        let mut data = data;
        let mut is_loading = is_loading;
        let mut error = error;
        let request_snapshot = *request_id.read();

        async move {
            is_loading.set(true);
            error.set(None);

            let result = match backend {
                Some(ctx) => match tokio::task::spawn_blocking(move || (loader)(ctx)).await {
                    Ok(inner) => inner.map_err(|e| format!("Loader task failed: {e}")),
                    Err(e) => Err(format!("Loader task failed: {e}")),
                },
                None => Err("Backend not available".to_string()),
            };

            if request_snapshot != *request_id.read() {
                return;
            }

            match result {
                Ok(value) => data.set(value),
                Err(e) => error.set(Some(e)),
            }

            is_loading.set(false);
        }
    });

    use_keyed_effect(key, move |_| {
        backend_slot_for_effect.set(backend_for_effect.clone());
        let next = request_id.read().wrapping_add(1);
        request_id.set(next);
        future.restart();
    });

    LoadResult { data, state }
}

/// Debounce a value to reduce downstream work (e.g., search queries).
pub fn use_debounced_value(value: String, delay_ms: u64) -> Signal<String> {
    let debounced = use_signal(|| value.clone());
    let mut latest = use_signal(|| value.clone());
    if *latest.read() != value {
        latest.set(value.clone());
    }

    let mut request_id = use_signal(|| 0u64);
    let mut future = use_future(move || {
        let mut debounced = debounced;
        let latest = latest;
        let request_snapshot = *request_id.read();
        async move {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            if request_snapshot != *request_id.read() {
                return;
            }
            debounced.set(latest.read().clone());
        }
    });

    use_keyed_effect(value, move |_| {
        let next = request_id.read().wrapping_add(1);
        request_id.set(next);
        future.restart();
    });

    debounced
}

/// Load dashboard analytics with loading and error state.
pub fn use_load_dashboard_analytics(
    backend: Option<Arc<AppContext>>,
) -> LoadResult<Option<AppAnalytics>> {
    let key = backend_key(&backend);
    let loader: Loader<Option<AppAnalytics>> = Arc::new(move |ctx| {
        let use_case = ServiceFactory::dashboard(&ctx);
        use_case.execute().map(Some).map_err(|e| format!("Failed to load analytics: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load all courses from the database.
pub fn use_load_courses(backend: Option<Arc<AppContext>>) -> LoadResult<Vec<Course>> {
    let key = backend_key(&backend);
    let loader: Loader<Vec<Course>> = Arc::new(move |ctx| {
        ctx.course_repo.find_all().map_err(|e| format!("Failed to load courses: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load modules for a specific course.
pub fn use_load_modules(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> LoadResult<Vec<Module>> {
    let course_id = course_id.clone();
    let key = format!("{}|{}", backend_key(&backend), course_id.as_uuid());
    let loader: Loader<Vec<Module>> = Arc::new(move |ctx| {
        ctx.module_repo
            .find_by_course(&course_id)
            .map_err(|e| format!("Failed to load modules: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load videos for a specific module.
pub fn use_load_videos(
    backend: Option<Arc<AppContext>>,
    module_id: &ModuleId,
) -> LoadResult<Vec<Video>> {
    let module_id = module_id.clone();
    let key = format!("{}|{}", backend_key(&backend), module_id.as_uuid());
    let loader: Loader<Vec<Video>> = Arc::new(move |ctx| {
        ctx.video_repo.find_by_module(&module_id).map_err(|e| format!("Failed to load videos: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load a single course by ID.
pub fn use_load_course(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> LoadResult<Option<Course>> {
    let course_id = course_id.clone();
    let key = format!("{}|{}", backend_key(&backend), course_id.as_uuid());
    let loader: Loader<Option<Course>> = Arc::new(move |ctx| {
        ctx.course_repo.find_by_id(&course_id).map_err(|e| format!("Failed to load course: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load a single video by ID.
pub fn use_load_video(
    backend: Option<Arc<AppContext>>,
    video_id: &VideoId,
) -> LoadResult<Option<Video>> {
    let video_id = video_id.clone();
    let key = format!("{}|{}", backend_key(&backend), video_id.as_uuid());
    let loader: Loader<Option<Video>> = Arc::new(move |ctx| {
        ctx.video_repo.find_by_id(&video_id).map_err(|e| format!("Failed to load video: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load a single exam by ID.
pub fn use_load_exam(
    backend: Option<Arc<AppContext>>,
    exam_id: &ExamId,
) -> LoadResult<Option<Exam>> {
    let exam_id = exam_id.clone();
    let key = format!("{}|{}", backend_key(&backend), exam_id.as_uuid());
    let loader: Loader<Option<Exam>> = Arc::new(move |ctx| {
        ctx.exam_repo.find_by_id(&exam_id).map_err(|e| format!("Failed to load exam: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load exams for a specific video.
pub fn use_load_exams(
    backend: Option<Arc<AppContext>>,
    video_id: &VideoId,
) -> LoadResult<Vec<Exam>> {
    let video_id = video_id.clone();
    let key = format!("{}|{}", backend_key(&backend), video_id.as_uuid());
    let loader: Loader<Vec<Exam>> = Arc::new(move |ctx| {
        ctx.exam_repo.find_by_video(&video_id).map_err(|e| format!("Failed to load exams: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load all exams from the database.
pub fn use_load_all_exams(backend: Option<Arc<AppContext>>) -> LoadResult<Vec<Exam>> {
    let key = backend_key(&backend);
    let loader: Loader<Vec<Exam>> = Arc::new(move |ctx| {
        ctx.exam_repo.find_all().map_err(|e| format!("Failed to load exams: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load all videos for a specific course (across all modules).
pub fn use_load_videos_by_course(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> LoadResult<Vec<Video>> {
    let course_id = course_id.clone();
    let key = format!("{}|{}", backend_key(&backend), course_id.as_uuid());
    let loader: Loader<Vec<Video>> = Arc::new(move |ctx| {
        ctx.video_repo
            .find_by_course(&course_id)
            .map_err(|e| format!("Failed to load videos for course: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load all tags from the database.
pub fn use_load_tags(backend: Option<Arc<AppContext>>) -> LoadResult<Vec<Tag>> {
    let key = backend_key(&backend);
    let loader: Loader<Vec<Tag>> = Arc::new(move |ctx| {
        ctx.tag_repo.find_all().map_err(|e| format!("Failed to load tags: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Load tags for a specific course.
pub fn use_load_course_tags(
    backend: Option<Arc<AppContext>>,
    course_id: &CourseId,
) -> LoadResult<Vec<Tag>> {
    let course_id = course_id.clone();
    let key = format!("{}|{}", backend_key(&backend), course_id.as_uuid());
    let loader: Loader<Vec<Tag>> = Arc::new(move |ctx| {
        ctx.tag_repo
            .find_by_course(&course_id)
            .map_err(|e| format!("Failed to load course tags: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Search across courses, videos, and notes (debounced).
pub fn use_search(
    backend: Option<Arc<AppContext>>,
    query: String,
) -> LoadResult<Vec<SearchResult>> {
    let debounced = use_debounced_value(query.trim().to_string(), 300);
    let key = format!("{}|{}", backend_key(&backend), debounced.read().clone());
    let debounced_query = debounced.read().clone();

    let loader: Loader<Vec<SearchResult>> = Arc::new(move |ctx| {
        let query = debounced_query.clone();
        if query.trim().is_empty() {
            return Ok(Vec::new());
        }
        ctx.search_repo.search(&query, 20).map_err(|e| format!("Search failed: {e}"))
    });

    use_async_loader(backend, key, loader)
}

/// Hook to synchronize the application state with Discord Rich Presence.
/// Monitors route and state changes to update the user's status.
pub fn use_presence_sync(backend: Option<Arc<AppContext>>) {
    let route = use_route::<Route>();
    let state = use_context::<AppState>();

    let (course_id, video_id) = match &route {
        Route::VideoPlayer { course_id, video_id } => (course_id.clone(), video_id.clone()),
        Route::CourseView { course_id } => (course_id.clone(), String::new()),
        _ => {
            let course_id = state
                .current_course
                .read()
                .as_ref()
                .map(|c| c.id().as_uuid().to_string())
                .unwrap_or_default();
            let video_id = state.current_video_id.read().clone().unwrap_or_default();
            (course_id, video_id)
        },
    };

    let key = format!("{}|{:?}|{}|{}", backend_key(&backend), route, course_id, video_id);

    use_keyed_effect(key, move |_| {
        let backend = match backend.as_ref() {
            Some(b) => b,
            None => return,
        };

        log::debug!(
            "Presence sync route={:?} course_id={} video_id={}",
            route,
            course_id,
            video_id
        );

        let activity = match route {
            Route::Dashboard {} => Activity::Dashboard,
            Route::CourseList {} => Activity::BrowsingCourses,
            Route::CourseView { .. } => Activity::BrowsingCourses,
            Route::VideoPlayer { ref course_id, ref video_id } => {
                let course = state.current_course.read();
                let videos = state.current_videos.read();

                let video = videos.iter().find(|v| v.id().to_string() == video_id.as_str());

                if let (Some(c), Some(v)) = (course.as_ref(), video) {
                    Activity::Watching {
                        course_title: c.name().to_string(),
                        video_title: v.title().to_string(),
                    }
                } else if let Some(c) = course.as_ref() {
                    Activity::Watching {
                        course_title: c.name().to_string(),
                        video_title: "Video".to_string(),
                    }
                } else if !course_id.is_empty() {
                    Activity::Watching {
                        course_title: "Course".to_string(),
                        video_title: "Video".to_string(),
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

        log::debug!("Presence sync activity decided: {:?}", activity);

        let use_case = ServiceFactory::update_presence(backend);
        use_case.execute(crate::application::use_cases::UpdatePresenceInput { activity });
    });
}
