use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConversationHistory {
    pub messages: Vec<ChatMessage>,
    pub course_context: Option<CourseContext>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseContext {
    pub course_id: uuid::Uuid,
    pub course_name: String,
    pub course_structure: Option<crate::types::CourseStructure>,
    pub youtube_playlist_url: Option<String>,
    pub current_video_context: Option<crate::types::VideoContext>,
    pub source_type: CourseSourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CourseSourceType {
    YouTube { playlist_url: String },
    Local { folder_path: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatbotResponse {
    pub message: String,
    pub suggestions: Vec<String>,
    pub context_used: Option<CourseContext>,
}

// Gemini API request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiRequest {
    pub contents: Vec<GeminiContent>,
    #[serde(rename = "generationConfig")]
    pub generation_config: Option<GenerationConfig>,
    #[serde(rename = "safetySettings")]
    pub safety_settings: Option<Vec<SafetySetting>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiContent {
    pub role: String,
    pub parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiPart {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    pub temperature: Option<f32>,
    #[serde(rename = "topK")]
    pub top_k: Option<i32>,
    #[serde(rename = "topP")]
    pub top_p: Option<f32>,
    #[serde(rename = "maxOutputTokens")]
    pub max_output_tokens: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiResponse {
    pub candidates: Vec<GeminiCandidate>,
    #[serde(rename = "usageMetadata")]
    pub usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiCandidate {
    pub content: GeminiContent,
    #[serde(rename = "finishReason")]
    pub finish_reason: Option<String>,
    pub index: Option<i32>,
    #[serde(rename = "safetyRatings")]
    pub safety_ratings: Option<Vec<SafetyRating>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRating {
    pub category: String,
    pub probability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    pub prompt_token_count: Option<i32>,
    #[serde(rename = "candidatesTokenCount")]
    pub candidates_token_count: Option<i32>,
    #[serde(rename = "totalTokenCount")]
    pub total_token_count: Option<i32>,
}

impl ConversationHistory {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_message(&mut self, role: String, content: String) {
        self.messages.push(ChatMessage { role, content, timestamp: chrono::Utc::now() });
    }

    pub fn set_course_context(&mut self, context: CourseContext) {
        self.course_context = Some(context);
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }
}
