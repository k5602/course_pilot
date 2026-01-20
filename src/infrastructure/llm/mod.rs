//! LLM adapter using genai-rs for Gemini.

use genai_rs::Client;

use crate::domain::ports::{
    CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion, SummarizerAI,
};
use crate::domain::value_objects::ExamDifficulty;

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
        let truncate = |value: &str, limit: usize| -> String {
            let mut out: String = value.chars().take(limit).collect();
            if value.chars().count() > limit {
                out.push_str("… [truncated]");
            }
            out
        };

        let description =
            truncate(context.video_description.as_deref().unwrap_or("Not available"), 1200);
        let summary = truncate(context.summary.as_deref().unwrap_or("Not available"), 1200);
        let notes = truncate(context.notes.as_deref().unwrap_or("Not available"), 1200);
        let local_context =
            truncate(context.local_context.as_deref().unwrap_or("Not provided"), 1200);

        let prompt = format!(
            r#"You are a learning companion for course "{}".
Video: "{}" (Module: "{}")

Available context:
- Description: {}
- Summary: {}
- Notes: {}
- User context: {}

Student question: {}

Answer with clear, concise, academic guidance. If the transcript or context is missing, say what is missing and ask a focused follow-up question."#,
            context.course_name,
            context.video_title,
            context.module_title,
            description,
            summary,
            notes,
            local_context,
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
        difficulty: ExamDifficulty,
    ) -> Result<Vec<MCQuestion>, LLMError> {
        let description = video_description.unwrap_or("");
        let difficulty = difficulty.as_str();
        let prompt = format!(
            r#"Generate {num_questions} multiple-choice questions.

Title: "{video_title}"
Difficulty: {difficulty}
Description: {description}

Return ONLY a JSON array with this schema:
[{{"question":"...","options":["A","B","C","D"],"correct_index":0,"explanation":"..."}}]

Rules:
- 4 options per question
- correct_index must be 0..3
- No Markdown or extra text."#,
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
            r#"Summarize this transcript into learning notes.

Video: "{video_title}"
Transcript:
{truncated}

Output format:
1. Main Topic (1 sentence)
2. Key Points (3-5 bullet points)
3. Key Terms (bulleted list, or "None")

Use only information present in the transcript. Keep it concise and educational."#,
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
