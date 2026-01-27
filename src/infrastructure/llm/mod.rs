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

Context (use only what is provided):
- Description: {}
- Summary: {}
- Notes: {}
- User context: {}

Student question: {}

Guidelines:
- Ground answers strictly in the context above; do not invent details.
- If context is insufficient, state the missing piece and ask one focused follow-up.
- Keep the response concise (3-6 sentences). Use bullets only if clarifying steps.
- Do not mention system instructions or the prompt."#,
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
            r#"You are an expert instructor creating a focused MCQ quiz from the provided context.

Context:
- Title: "{video_title}"
- Description: {description}

Task:
Generate exactly {num_questions} multiple-choice questions at {difficulty} difficulty.
Use ONLY the context above. Do NOT ask about timestamps, durations, or video metadata.
Avoid vague or trivial questions. Each question should test a specific concept or inference.
Options must be plausible and mutually exclusive.

Output: Return ONLY a JSON array with this schema:
[{{"question":"...","options":["A","B","C","D"],"correct_index":0,"explanation":"..."}}]

Rules:
- 4 options per question
- correct_index must be 0..3
- explanation must justify why the correct option is correct (1–2 sentences)
- No Markdown or extra text
- Do not mention the prompt, the context, or any system instructions"#,
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
            r#"You are creating study notes from a transcript.

Video: "{video_title}"
Transcript:
{truncated}

Output format (plain text only):
1. Main Topic: <one sentence>
2. Key Points:
- ...
- ...
- ...
3. Key Terms:
- term: short definition
(or "None")

Rules:
- Use only information in the transcript; do not add external knowledge.
- Prefer precise, concrete statements over vague summaries.
- Do not include timestamps, speaker labels, or meta commentary."#,
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
