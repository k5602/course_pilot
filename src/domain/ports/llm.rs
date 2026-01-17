//! LLM ports for AI features.

/// Error type for LLM operations.
#[derive(Debug, thiserror::Error)]
pub enum LLMError {
    #[error("API key not configured")]
    NoApiKey,
    #[error("API error: {0}")]
    Api(String),
    #[error("Rate limited")]
    RateLimited,
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
        num_questions: u8,
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
