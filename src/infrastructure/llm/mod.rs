//! LLM adapter using google_ai_rs (Gemini).

use crate::domain::ports::{CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion};

/// Gemini API adapter using google_ai_rs.
pub struct GeminiAdapter {
    api_key: String,
}

impl GeminiAdapter {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }
}

impl CompanionAI for GeminiAdapter {
    async fn ask(&self, question: &str, context: &CompanionContext) -> Result<String, LLMError> {
        // TODO: Implement with google_ai_rs
        // 1. Build prompt with context
        // 2. Call Gemini API
        // 3. Parse response

        let _ = (&self.api_key, question, context);
        todo!("Implement with google_ai_rs crate")
    }
}

impl ExaminerAI for GeminiAdapter {
    async fn generate_mcq(
        &self,
        video_title: &str,
        video_description: Option<&str>,
        num_questions: u8,
    ) -> Result<Vec<MCQuestion>, LLMError> {
        // TODO: Implement with google_ai_rs
        // 1. Build MCQ generation prompt
        // 2. Call Gemini API with JSON mode
        // 3. Parse structured response

        let _ = (&self.api_key, video_title, video_description, num_questions);
        todo!("Implement with google_ai_rs crate")
    }
}
