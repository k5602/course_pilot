//! UI Actions - Async handlers for UI events

use std::sync::Arc;

use crate::application::use_cases::GenerateExamInput;
use crate::application::use_cases::IngestPlaylistInput;
use crate::application::{AppContext, ServiceFactory};
use crate::domain::ports::ExamRepository;
use crate::domain::value_objects::{CourseId, ExamId, VideoId};

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

/// Start an exam for a video.
/// If an exam already exists, it returns the existing one.
/// Otherwise, it generates a new one using AI.
pub async fn start_exam(
    backend: Option<Arc<AppContext>>,
    video_id: VideoId,
) -> Result<ExamId, String> {
    let ctx = match backend {
        Some(ctx) => ctx,
        None => return Err("Backend not initialized".to_string()),
    };

    if !ctx.has_llm() {
        return Err(
            "AI not configured. Please add an API key in settings to generate exams.".to_string()
        );
    }

    // Check if an exam already exists for this video to avoid re-generating
    if let Ok(existing) = ctx.exam_repo.find_by_video(&video_id) {
        if let Some(exam) = existing.first() {
            return Ok(exam.id().clone());
        }
    }

    // Get the use case from factory
    let use_case = match ServiceFactory::take_exam(&ctx) {
        Some(uc) => uc,
        None => return Err("Exam service not available".to_string()),
    };

    // Execute the use case
    let input = GenerateExamInput { video_id, num_questions: 5 };

    match use_case.generate(input).await {
        Ok(output) => Ok(output.exam_id),
        Err(e) => Err(e.to_string()),
    }
}

/// Ask the AI companion a question about the current video.
pub async fn ask_companion(
    backend: Option<Arc<AppContext>>,
    video_id: VideoId,
    question: String,
) -> Result<String, String> {
    let ctx = match backend {
        Some(ctx) => ctx,
        None => return Err("Backend not initialized".to_string()),
    };

    if !ctx.has_llm() {
        return Err("AI not configured. Please add a Gemini API key in settings.".to_string());
    }

    // Get the use case from factory
    let use_case = match ServiceFactory::ask_companion(&ctx) {
        Some(uc) => uc,
        None => return Err("AI companion service not available".to_string()),
    };

    // Execute the use case
    let input = crate::application::use_cases::AskCompanionInput { video_id, question };

    match use_case.execute(input).await {
        Ok(response) => Ok(response),
        Err(e) => Err(e.to_string()),
    }
}
