//! UI Actions - Async handlers for UI events

use std::fs;
use std::sync::Arc;

use crate::application::use_cases::AttachTranscriptInput;
use crate::application::use_cases::ExportCourseNotesInput;
use crate::application::use_cases::GenerateExamInput;
use crate::application::use_cases::IngestLocalInput;
use crate::application::use_cases::IngestPlaylistInput;
use crate::application::{AppContext, ServiceFactory};
use crate::domain::ports::{CourseRepository, ExamRepository};
use crate::domain::value_objects::{CourseId, ExamDifficulty, ExamId, VideoId};

/// Result of playlist import action.
#[derive(Clone, Debug)]
pub enum ImportResult {
    Success { course_id: CourseId, modules: usize, videos: usize },
    Error(String),
}

/// Import a playlist from YouTube.
pub async fn import_playlist(
    backend: Option<Arc<AppContext>>,
    url: String,
    name: Option<String>,
) -> ImportResult {
    let ctx = match backend {
        Some(ctx) => ctx,
        None => return ImportResult::Error("Backend not initialized".to_string()),
    };

    // Get the use case from factory (always available)
    let use_case = ServiceFactory::ingest_playlist(&ctx);

    // Execute the use case
    let input = IngestPlaylistInput { playlist_url: url, course_name: name };

    match use_case.execute(input).await {
        Ok(output) => ImportResult::Success {
            course_id: output.course_id,
            modules: output.modules_count,
            videos: output.videos_count,
        },
        Err(e) => ImportResult::Error(format!("Failed to fetch playlist: {}", e)),
    }
}

/// Import a local media folder (recursive).
pub async fn import_local_folder(
    backend: Option<Arc<AppContext>>,
    root_path: String,
    name: Option<String>,
) -> ImportResult {
    let ctx = match backend {
        Some(ctx) => ctx,
        None => return ImportResult::Error("Backend not initialized".to_string()),
    };

    let use_case = ServiceFactory::ingest_local(&ctx);

    let input = IngestLocalInput { root_path, course_name: name };

    match use_case.execute(input).await {
        Ok(output) => ImportResult::Success {
            course_id: output.course_id,
            modules: output.modules_count,
            videos: output.videos_count,
        },
        Err(e) => ImportResult::Error(format!("Failed to scan local media: {}", e)),
    }
}

/// Attach a subtitle or transcript file to a video.
pub async fn import_subtitle_for_video(
    backend: Option<Arc<AppContext>>,
    video_id: VideoId,
    subtitle_path: String,
) -> Result<usize, String> {
    let ctx = match backend {
        Some(ctx) => ctx,
        None => return Err("Backend not initialized".to_string()),
    };

    let raw = fs::read_to_string(&subtitle_path)
        .map_err(|e| format!("Failed to read subtitle file: {e}"))?;

    let use_case = ServiceFactory::attach_transcript(&ctx);
    let input = AttachTranscriptInput { video_id, transcript_text: raw };

    let output = use_case.execute(input).map_err(|e| e.to_string())?;
    Ok(output.cleaned_length)
}

/// Start an exam for a video.
/// If an exam already exists, it returns the existing one.
/// Otherwise, it generates a new one using AI.
pub async fn start_exam(
    backend: Option<Arc<AppContext>>,
    video_id: VideoId,
    num_questions: u8,
    difficulty: ExamDifficulty,
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
    let input = GenerateExamInput { video_id, num_questions, difficulty };

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

/// Export notes for a course as Markdown and save using a file dialog.
pub async fn export_course_notes_with_dialog(
    backend: Option<Arc<AppContext>>,
    course_id: CourseId,
) -> Result<String, String> {
    let ctx = match backend {
        Some(ctx) => ctx,
        None => return Err("Backend not initialized".to_string()),
    };

    let use_case = ServiceFactory::export_course_notes(&ctx);
    let markdown = use_case
        .execute(ExportCourseNotesInput { course_id: course_id.clone() })
        .map_err(|e| e.to_string())?;

    let filename = default_course_filename(&ctx, &course_id);

    let Some(path) =
        rfd::FileDialog::new().add_filter("Markdown", &["md"]).set_file_name(&filename).save_file()
    else {
        return Err("Save cancelled".to_string());
    };

    fs::write(&path, markdown).map_err(|e| format!("Failed to save notes: {e}"))?;
    Ok(path.display().to_string())
}

fn default_course_filename(ctx: &AppContext, course_id: &CourseId) -> String {
    let base = ctx
        .course_repo
        .find_by_id(course_id)
        .ok()
        .flatten()
        .map(|course| course.name().to_string())
        .unwrap_or_else(|| "course-notes".to_string());

    let sanitized = sanitize_filename(&base);

    format!("{sanitized}.md")
}

fn sanitize_filename(input: &str) -> String {
    let mut cleaned = String::new();
    for ch in input.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' || ch == ' ' {
            cleaned.push(ch);
        }
    }

    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return "course-notes".to_string();
    }

    trimmed.replace(' ', "_").to_lowercase()
}
