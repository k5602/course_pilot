//! LLM ports for AI features.

use std::future::Future;
use std::pin::Pin;

use crate::domain::value_objects::ExamDifficulty;

/// Error type for LLM operations.
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API error: {0}")]
    Api(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

/// Context for the AI companion.
#[derive(Debug, Clone)]
pub struct CompanionContext {
    pub video_title: String,
    pub video_description: Option<String>,
    pub module_title: String,
    pub course_name: String,
    pub summary: Option<String>,
    pub notes: Option<String>,
    /// Extra user-provided context for local videos without transcripts.
    pub local_context: Option<String>,
}

/// Port for the Sidecar Companion (AI-B).
#[allow(async_fn_in_trait)]
pub trait CompanionAI: Send + Sync {
    /// Answers a question in the context of the current video.
    async fn ask(&self, question: &str, context: &CompanionContext) -> Result<String, LLMError>;
}

/// MCQ question structure.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MCQuestion {
    pub question: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
}

/// Port for the Manual Examiner (AI-C).
#[allow(async_fn_in_trait)]
pub trait ExaminerAI: Send + Sync {
    /// Generates MCQ questions for a video.
    async fn generate_mcq(
        &self,
        video_title: &str,
        video_description: Option<&str>,
        video_summary: Option<&str>,
        num_questions: u8,
        difficulty: ExamDifficulty,
    ) -> Result<Vec<MCQuestion>, LLMError>;
}

/// Port for video transcript summarization.
#[allow(async_fn_in_trait)]
pub trait SummarizerAI: Send + Sync {
    /// Summarizes a video transcript into key points.
    async fn summarize_transcript(
        &self,
        transcript: &str,
        video_title: &str,
    ) -> Result<String, LLMError>;
}

/// Port for generating descriptive module titles from grouped video titles.
pub trait ModuleTitleGenerator: Send + Sync {
    /// Generates a concise module title from the video titles in that module.
    /// Returns the title string, or an error if generation fails.
    fn generate_module_title(
        &self,
        video_titles: &[String],
        course_name: &str,
        module_index: usize,
    ) -> Pin<Box<dyn Future<Output = Result<String, LLMError>> + Send + '_>>;
}
