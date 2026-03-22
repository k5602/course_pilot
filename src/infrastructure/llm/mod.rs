//! LLM adapter using genai for multi-provider AI.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use genai::Client;
use genai::chat::{ChatMessage, ChatRequest};
use genai::resolver::AuthData;

use crate::domain::ports::{
    CompanionAI, CompanionContext, ExaminerAI, LLMError, MCQuestion, ModuleTitleGenerator,
    SummarizerAI,
};
use crate::domain::value_objects::ExamDifficulty;

/// Multi-provider AI adapter (Gemini by default).
pub struct GeminiAdapter {
    client: Client,
    model: String,
}

impl GeminiAdapter {
    /// Creates a new adapter with the given API key.
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .with_auth_resolver_fn(move |_: genai::ModelIden| {
                Ok(Some(AuthData::from_single(api_key.clone())))
            })
            .build();
        Self { client, model: "gemini/gemini-2.5-flash".to_string() }
    }

    /// Executes a chat request with automatic retry on transient errors.
    async fn execute_with_retry(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
    ) -> Result<String, LLMError> {
        const MAX_RETRIES: u32 = 3;
        let delays =
            [Duration::from_millis(500), Duration::from_millis(1000), Duration::from_millis(2000)];

        for attempt in 0..MAX_RETRIES {
            let mut messages = Vec::new();
            if let Some(sys) = system_prompt {
                messages.push(ChatMessage::system(sys));
            }
            messages.push(ChatMessage::user(user_prompt));

            let req = ChatRequest::new(messages);
            let result = self.client.exec_chat(&self.model, req, None).await;

            match result {
                Ok(resp) => {
                    return Ok(resp.first_text().unwrap_or("No response").to_string());
                },
                Err(e) => {
                    let is_retryable = e.to_string().contains("rate")
                        || e.to_string().contains("timeout")
                        || e.to_string().contains("429")
                        || e.to_string().contains("503")
                        || e.to_string().contains("network")
                        || e.to_string().contains("server error")
                        || e.to_string().contains("internal");

                    if is_retryable && attempt < MAX_RETRIES - 1 {
                        tokio::time::sleep(delays[attempt as usize]).await;
                        continue;
                    }
                    return Err(LLMError::Api(e.to_string()));
                },
            }
        }

        Err(LLMError::Api("Max retries exceeded".to_string()))
    }
}

/// A no-op module title generator that triggers fallback (default) title logic.
pub struct NoopTitleGenerator;

impl NoopTitleGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopTitleGenerator {
    fn default() -> Self {
        Self
    }
}

impl ModuleTitleGenerator for NoopTitleGenerator {
    fn generate_module_title(
        &self,
        _titles: &[String],
        _course: &str,
        _idx: usize,
    ) -> Pin<Box<dyn Future<Output = Result<String, LLMError>> + Send>> {
        Box::pin(async { Err(LLMError::NoApiKey) })
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

        let response = self.execute_with_retry(None, &prompt).await?;
        Ok(response)
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

        let text = self.execute_with_retry(None, &prompt).await?;
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

        let text = self.execute_with_retry(None, &prompt).await?;
        Ok(text)
    }
}

impl ModuleTitleGenerator for GeminiAdapter {
    fn generate_module_title(
        &self,
        video_titles: &[String],
        course_name: &str,
        module_index: usize,
    ) -> Pin<Box<dyn Future<Output = Result<String, LLMError>> + Send + '_>> {
        let titles_str = video_titles.join("\n");
        let prompt = format!(
            "You are a course designer. Given these video titles from Module {} of \"{}\", produce ONE concise, descriptive title (under 8 words).\n\nVideo titles:\n{}\n\nModule Title:",
            module_index + 1,
            course_name,
            titles_str
        );
        Box::pin(async move {
            let response = self.execute_with_retry(None, &prompt).await?;
            let title = response.trim().to_string();
            if title.is_empty() || title.len() > 100 {
                return Err(LLMError::InvalidResponse("Empty or overly long title".into()));
            }
            Ok(title)
        })
    }
}
