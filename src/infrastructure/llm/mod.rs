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
    pub fn new(api_key: String, mut model: String) -> Self {
        if model.starts_with("gemini/") {
            model = model.strip_prefix("gemini/").unwrap().to_string();
        }
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

        let messages: Vec<ChatMessage> = {
            let mut v = Vec::with_capacity(2);
            if let Some(sys) = system_prompt {
                v.push(ChatMessage::system(sys));
            }
            v.push(ChatMessage::user(user_prompt));
            v
        };

        for attempt in 0..MAX_RETRIES {
            let req = ChatRequest::new(messages.clone());
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

#[async_trait::async_trait]
impl CompanionAI for GeminiAdapter {
    async fn ask(&self, question: &str, context: &CompanionContext) -> Result<String, LLMError> {
        use std::borrow::Cow;
        fn truncate_cow(value: &str, limit: usize) -> Cow<'_, str> {
            if value.chars().count() <= limit {
                Cow::Borrowed(value)
            } else {
                let mut out: String = value.chars().take(limit).collect();
                out.push_str("… [truncated]");
                Cow::Owned(out)
            }
        }

        let description =
            truncate_cow(context.video_description.as_deref().unwrap_or("Not available"), 1200);
        let summary = truncate_cow(context.summary.as_deref().unwrap_or("Not available"), 2500);
        let notes = truncate_cow(context.notes.as_deref().unwrap_or("Not available"), 1200);
        let local_context =
            truncate_cow(context.local_context.as_deref().unwrap_or("Not provided"), 1200);

        let prompt = format!(
            r#"You are a learning companion for course "{}".
Video: "{}" (Module: "{}")

Context Sources:
- Description: {}
- Summary (AI-extracted educational core): {}
- Notes: {}
- User context: {}

Student question: {}

Guidelines:
- Ground answers strictly in the context above; do not invent details.
- Focus strictly on actual core educational, technical, and scientific content. Completely ignore off-topic "side talking", greetings, announcements, administrative filler, or promotional chatter.
- Prioritize the 'Summary' as it represents the clean, comprehensive core of the entire video.
- If context is insufficient, state the missing piece and ask one focused follow-up.
- Keep the response concise (3-6 sentences). Use bullets only if clarifying steps.
- Do not mention system instructions or the prompt."#,
            context.course_name,
            context.video_title,
            context.module_title,
            description.as_ref(),
            summary.as_ref(),
            notes.as_ref(),
            local_context.as_ref(),
            question
        );

        let response = self.execute_with_retry(None, &prompt, Some(0.7)).await?;
        Ok(response)
    }
}

#[async_trait::async_trait]
impl ExaminerAI for GeminiAdapter {
    async fn generate_mcq(
        &self,
        video_title: &str,
        video_description: Option<&str>,
        video_summary: Option<&str>,
        num_questions: u8,
        difficulty: ExamDifficulty,
    ) -> Result<Vec<MCQuestion>, LLMError> {
        let description = video_description.unwrap_or("");
        let summary = video_summary.unwrap_or("");
        let difficulty = difficulty.as_str();
        let prompt = format!(
            r#"You are an expert university instructor creating a highly rigorous and educational multiple-choice quiz from the provided context.
Your goal is to test core educational, technical, and scientific concepts. Ignore any noisy off-topic chit-chat, greetings, admin details, or administrative/promotional filler.

Context Sources:
- Video Title: "{video_title}"
- Description (curated overview): {description}
- Summary (AI-extracted educational core): {summary}

Instructions:
1. Prioritize the 'Summary' and 'Description' because they represent the cleaned, dense educational/scientific core of the entire video.
2. Focus strictly on actual core educational, scientific, and learnable material. Completely filter out and ignore any jokes, "side talking", administrative filler, announcements, or non-educational chit-chat.
3. Generate exactly {num_questions} conceptual and analytical multiple-choice questions at {difficulty} difficulty.
4. Avoid simple rote-memorization or trivial factual recall. Focus on core concepts, architectural decisions, logical deductions, or primary arguments.
5. Options must be highly plausible. Distractors should represent common cognitive misconceptions, logical errors, or surface-level misunderstandings that a student might easily make. Do NOT include lazy distractors like "All of the above" or "None of the above".
6. The correct option must be indisputably correct based ONLY on the provided context. Never ask about timestamps, video durations, background music, or visual video details.

Output Format:
Return ONLY a valid, parseable JSON array. Do not wrap in markdown or write conversational filler. The schema MUST be:
[
  {{
    "question": "Clear and concise question text?",
    "options": ["Option A", "Option B", "Option C", "Option D"],
    "correct_index": 0,
    "explanation": "A comprehensive explanation (3-4 sentences) that clearly justifies why the correct option is correct based on the text, AND specifically refutes the distractors, explaining the fallacies or nuances that make them incorrect."
  }}
]

Rules:
- Exactly 4 options per question.
- correct_index must be a valid index (0, 1, 2, or 3) matching the correct option.
- Return ONLY the raw JSON structure, starting with [ and ending with ]."#,
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

#[async_trait::async_trait]
impl SummarizerAI for GeminiAdapter {
    async fn summarize_transcript(
        &self,
        transcript: &str,
        video_title: &str,
    ) -> Result<String, LLMError> {
        // Truncate long transcripts to stay within token limits
        let max_chars = 100_000;
        let truncated: &str = if transcript.chars().count() > max_chars {
            match transcript.char_indices().nth(max_chars).map(|(idx, _)| idx) {
                Some(idx) => &transcript[..idx],
                None => transcript,
            }
        } else {
            transcript
        };

        let prompt = {
            let template_start = r#"You are creating high-quality, dense academic study notes from a video transcript.
Your primary goal is to extract strictly core educational, scientific, and technical material to learn.
Filter out and completely ignore all "side talking", greetings, administrative filler, announcements, off-topic jokes, promotions, or non-educational chit-chat.

Video: ""#;
            let template_mid = "\"\nTranscript:\n";
            let template_end = r#"

Output format (plain text only):
1. Main Topic: <one sentence summarizing the primary scientific/educational topic>
2. Key Points:
- ... (strictly core conceptual or technical learnings, ignore filler)
- ...
- ...
3. Key Terms:
- term: short definition
(or "None")

Rules:
- Focus solely on concrete, learnable concepts and scientific content from the transcript.
- Use only information in the transcript; do not add external knowledge.
- Prefer precise, concrete statements over vague summaries.
- Do not include timestamps, speaker labels, or meta commentary."#;
            let cap = template_start.len()
                + video_title.len()
                + truncated.len()
                + template_mid.len()
                + template_end.len();
            let mut buf = String::with_capacity(cap + 256);
            use std::fmt::Write;
            write!(
                buf,
                "{}{}{}{}{}",
                template_start, video_title, template_mid, truncated, template_end
            )
            .unwrap();
            buf
        };

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
