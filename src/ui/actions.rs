//! UI Actions - Async handlers for UI events

use std::sync::Arc;

use crate::application::use_cases::IngestPlaylistInput;
use crate::application::{AppContext, ServiceFactory};
use crate::domain::value_objects::CourseId;

/// Result of playlist import action.
#[derive(Clone, Debug)]
pub enum ImportResult {
    Success { course_id: CourseId, modules: usize, videos: usize },
    Error(String),
}

/// Import a playlist from YouTube.
/// Returns None if YouTube or embedder is not configured.
pub async fn import_playlist(
    backend: Option<Arc<AppContext>>,
    url: String,
    name: Option<String>,
) -> ImportResult {
    let ctx = match backend {
        Some(ctx) => ctx,
        None => return ImportResult::Error("Backend not initialized".to_string()),
    };

    // Check if required services are available
    if !ctx.has_youtube() {
        return ImportResult::Error("YouTube API not configured".to_string());
    }

    // Get the use case from factory
    let use_case = match ServiceFactory::ingest_playlist(&ctx) {
        Some(uc) => uc,
        None => {
            return ImportResult::Error(
                "Required services not available (YouTube + ML)".to_string(),
            );
        },
    };

    // Execute the use case
    let input = IngestPlaylistInput { playlist_url: url, course_name: name };

    match use_case.execute(input).await {
        Ok(output) => ImportResult::Success {
            course_id: output.course_id,
            modules: output.modules_count,
            videos: output.videos_count,
        },
        Err(e) => ImportResult::Error(e.to_string()),
    }
}
