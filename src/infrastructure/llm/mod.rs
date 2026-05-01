//! LLM adapter using genai for multi-provider AI.

use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use genai::Client;
use genai::chat::{ChatMessage, ChatOptions, ChatRequest};
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
    /// Creates a new adapter with the given API key and model name.
    pub fn new(api_key: String, model: String) -> Self {
        let client = Client::builder()
            .with_auth_resolver_fn(move |_: genai::ModelIden| {
                Ok(Some(AuthData::from_single(api_key.clone())))
            })
            .build();
        Self { client, model }
    }

    /// Executes a chat request with automatic retry on transient errors.
    async fn execute_with_retry(
        &self,
        system_prompt: Option<&str>,
        user_prompt: &str,
        temperature: Option<f64>,
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
            let options = temperature.map(|t| ChatOptions::default().with_temperature(t));
            let result = self.client.exec_chat(&self.model, req, options.as_ref()).await;

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

/// A fallback module title generator that returns an error to trigger default title logic.
pub struct FallbackTitleGenerator;

impl FallbackTitleGenerator {
    pub fn new() -> Self {
        Self
    }
}

impl Default for FallbackTitleGenerator {
    fn default() -> Self {
        Self
    }
}

impl ModuleTitleGenerator for FallbackTitleGenerator {
    fn generate_module_title(
        &self,
        _titles: &[String],
        _course: &str,
        _idx: usize,
    ) -> Pin<Box<dyn Future<Output = Result<String, LLMError>> + Send>> {
        Box::pin(async { Err(LLMError::NoApiKey) })
    }
}

/// Extracts a JSON array or object from an LLM response string, skipping
/// any surrounding markdown fences or explanatory text.
fn extract_json_from_response(text: &str) -> Result<&str, LLMError> {
    let text = text.trim();
    let start = text.find('[').or_else(|| text.find('{')).ok_or_else(|| {
        LLMError::InvalidResponse("No JSON array or object found in response".into())
    })?;
    let end = if text.as_bytes()[start] == b'[' {
        text.rfind(']').ok_or_else(|| LLMError::InvalidResponse("Unclosed JSON array".into()))? + 1
    } else {
        text.rfind('}').ok_or_else(|| LLMError::InvalidResponse("Unclosed JSON object".into()))? + 1
    };
    Ok(&text[start..end])
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
        let transcript = truncate(context.transcript.as_deref().unwrap_or("Not available"), 5000);
        let local_context =
            truncate(context.local_context.as_deref().unwrap_or("Not provided"), 1200);

        let prompt = format!(
            r#"You are a learning companion for course "{}".
Video: "{}" (Module: "{}")

Context (use only what is provided):
- Description: {}
- Summary: {}
- Notes: {}
- Transcript: {}
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
            transcript,
            local_context,
            question
        );

        let response = self.execute_with_retry(None, &prompt, Some(0.7)).await?;
        Ok(response)
    }
}

impl ExaminerAI for GeminiAdapter {
    async fn generate_mcq(
        &self,
        video_title: &str,
        video_description: Option<&str>,
        video_transcript: Option<&str>,
        num_questions: u8,
        difficulty: ExamDifficulty,
    ) -> Result<Vec<MCQuestion>, LLMError> {
        let description = video_description.unwrap_or("");
        let difficulty = difficulty.as_str();
        let transcript_info = video_transcript
            .and_then(|t| {
                if t.len() > 5000 {
                    t.char_indices().nth(5000).map(|(i, _)| &t[..i])
                } else {
                    Some(t)
                }
            })
            .unwrap_or("");
        let prompt = format!(
            r#"You are an expert instructor creating a focused MCQ quiz from the provided context.

Context:
- Title: "{video_title}"
- Description: {description}
- Transcript: {transcript_info}

Task:
Generate exactly {num_questions} multiple-choice questions at {difficulty} difficulty.
Base questions on the transcript content when available. Use ONLY the provided context. Do NOT ask about timestamps, durations, or video metadata.
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

        let text = self.execute_with_retry(None, &prompt, Some(0.2)).await?;
        let json_text = extract_json_from_response(&text)?;

        let questions: Vec<MCQuestion> = serde_json::from_str(json_text)
            .map_err(|e| LLMError::InvalidResponse(format!("JSON parse error: {}", e)))?;

        if questions.is_empty() {
            return Err(LLMError::InvalidResponse("No questions generated".into()));
        }
        if questions.len() > 20 {
            return Err(LLMError::InvalidResponse("Too many questions generated".into()));
        }
        for (i, q) in questions.iter().enumerate() {
            if q.options.len() < 2 {
                return Err(LLMError::InvalidResponse(format!(
                    "Question {} has fewer than 2 options",
                    i
                )));
            }
            if q.correct_index >= q.options.len() {
                return Err(LLMError::InvalidResponse(format!(
                    "Question {} correct_index out of range",
                    i
                )));
            }
            if q.question.is_empty() {
                return Err(LLMError::InvalidResponse(format!(
                    "Question {} has empty question text",
                    i
                )));
            }
        }
        Ok(questions)
    }
}

impl SummarizerAI for GeminiAdapter {
    async fn summarize_transcript(
        &self,
        transcript: &str,
        video_title: &str,
    ) -> Result<String, LLMError> {
        // Truncate long transcripts to stay within token limits
        let truncated: &str = if transcript.len() > 100_000 {
            if let Some((idx, _)) = transcript.char_indices().nth(100_000) {
                &transcript[..idx]
            } else {
                transcript
            }
        } else {
            transcript
        };

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

        let text = self.execute_with_retry(None, &prompt, Some(0.3)).await?;
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
            let response = self.execute_with_retry(None, &prompt, Some(0.3)).await?;
            let title = response.trim().to_string();
            if title.is_empty() || title.len() > 100 {
                return Err(LLMError::InvalidResponse("Empty or overly long title".into()));
            }
            Ok(title)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_json_from_plain_array() {
        let result = extract_json_from_response("[{\"question\":\"test\"}]").unwrap();
        assert_eq!(result, "[{\"question\":\"test\"}]");
    }

    #[test]
    fn extracts_json_from_markdown_fence() {
        let input = "```json\n[{\"question\":\"test\"}]\n```";
        let result = extract_json_from_response(input).unwrap();
        assert_eq!(result, "[{\"question\":\"test\"}]");
    }

    #[test]
    fn extracts_json_with_leading_text() {
        let input = "Here is your JSON: [{\"question\":\"test\"}]";
        let result = extract_json_from_response(input).unwrap();
        assert_eq!(result, "[{\"question\":\"test\"}]");
    }

    #[test]
    fn extracts_json_with_trailing_text() {
        let input = "[{\"question\":\"test\"}] End of response";
        let result = extract_json_from_response(input).unwrap();
        assert_eq!(result, "[{\"question\":\"test\"}]");
    }

    #[test]
    fn extract_json_returns_error_when_missing() {
        let result = extract_json_from_response("No JSON here");
        assert!(result.is_err());
    }

    #[test]
    fn extract_json_handles_unclosed_array() {
        let result = extract_json_from_response("[1, 2, 3");
        assert!(result.is_err());
    }

    #[test]
    fn extract_json_handles_nested_object() {
        let input = "{\"data\": {\"inner\": \"value\"}}";
        let result = extract_json_from_response(input).unwrap();
        assert_eq!(result, input.trim());
    }
}
