//! LLM adapter using genai-rs for Gemini.

use genai_rs::Client;

use crate::domain::ports::{
    CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion, SummarizerAI,
};

/// Gemini API adapter for AI features.
pub struct GeminiAdapter {
    client: Client,
}

impl GeminiAdapter {
    /// Creates a new Gemini adapter with the given API key.
    pub fn new(api_key: String) -> Self {
        let client = Client::new(api_key);
        Self { client }
    }
}

impl CompanionAI for GeminiAdapter {
    async fn ask(&self, question: &str, context: &CompanionContext) -> Result<String, LLMError> {
        let prompt = format!(
            r#"You are a learning companion for course "{}".
Video: "{}" (Module: "{}")
{}

Student asks: {}

Provide a concise, academic response."#,
            context.course_name,
            context.video_title,
            context.module_title,
            context.video_description.as_deref().unwrap_or(""),
            question
        );

        let response = self
            .client
            .interaction()
            .with_model("gemini-flash-latest")
            .with_text(&prompt)
            .create()
            .await
            .map_err(|e| LLMError::Api(e.to_string()))?;

        Ok(response.text().unwrap_or("No response").to_string())
    }
}

impl ExaminerAI for GeminiAdapter {
    async fn generate_mcq(
        &self,
        video_title: &str,
        video_description: Option<&str>,
        num_questions: u8,
    ) -> Result<Vec<MCQuestion>, LLMError> {
        let prompt = format!(
            r#"Generate {} MCQs for: "{}"
{}

Reply ONLY with JSON array:
[{{"question": "...", "options": ["A", "B", "C", "D"], "correct_index": 0, "explanation": "..."}}]"#,
            num_questions,
            video_title,
            video_description.unwrap_or("")
        );

        let response = self
            .client
            .interaction()
            .with_model("gemini-flash-latest")
            .with_text(&prompt)
            .create()
            .await
            .map_err(|e| LLMError::Api(e.to_string()))?;

        let text = response.text().unwrap_or("");
        let json_text = text
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        serde_json::from_str(json_text)
            .map_err(|e| LLMError::InvalidResponse(format!("JSON parse error: {}", e)))
    }
}

impl SummarizerAI for GeminiAdapter {
    async fn summarize_transcript(
        &self,
        transcript: &str,
        video_title: &str,
    ) -> Result<String, LLMError> {
        // Truncate long transcripts to ~10k chars to stay within token limits
        let truncated = if transcript.len() > 10000 { &transcript[..10000] } else { transcript };

        let prompt = format!(
            r#"Summarize this video transcript into key learning points.

Video: "{}"
Transcript:
{}

Provide a structured summary with:
1. Main Topic (1 sentence)
2. Key Points (3-5 bullet points)
3. Key Terms (if any technical terms are introduced)

Keep it concise and educational."#,
            video_title, truncated
        );

        let response = self
            .client
            .interaction()
            .with_model("gemini-flash-latest")
            .with_text(&prompt)
            .create()
            .await
            .map_err(|e| LLMError::Api(e.to_string()))?;

        Ok(response.text().unwrap_or("Unable to generate summary").to_string())
    }
}
