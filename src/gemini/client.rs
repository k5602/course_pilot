use anyhow::{Result, anyhow};
use reqwest::Client;

use super::types::*;
// API key is provided externally via SettingsManager/Backend; no direct settings access here.

#[derive(Clone)]
pub struct GeminiClient {
    client: Client,
    api_key: Option<String>,
    base_url: String,
}

impl GeminiClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key.into()),
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
        }
    }

    pub fn set_api_key(&mut self, api_key: impl Into<String>) {
        self.api_key = Some(api_key.into());
    }

    pub fn clear_api_key(&mut self) {
        self.api_key = None;
    }

    pub async fn initialize(&mut self) -> Result<()> {
        if self.api_key.is_none() {
            return Err(anyhow!(
                "Gemini API key not configured. Please set it in Settings."
            ));
        }
        Ok(())
    }

    pub fn is_configured(&self) -> bool {
        self.api_key.is_some()
    }

    pub async fn send_message(
        &self,
        message: &str,
        conversation_history: &ConversationHistory,
    ) -> Result<ChatbotResponse> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| anyhow!("Gemini API key not configured"))?;

        // Build the request with conversation history
        let mut contents = Vec::new();

        // Add system prompt with course context if available
        if let Some(context) = &conversation_history.course_context {
            let system_prompt = self.generate_system_prompt(context)?;
            contents.push(GeminiContent {
                role: "user".to_string(),
                parts: vec![GeminiPart {
                    text: system_prompt,
                }],
            });
            contents.push(GeminiContent {
                role: "model".to_string(),
                parts: vec![GeminiPart {
                    text: "I understand. I'm ready to help you with this course. What would you like to know?".to_string(),
                }],
            });
        }

        // Add conversation history (limit to last 10 messages to avoid token limits)
        let recent_messages = conversation_history
            .messages
            .iter()
            .rev()
            .take(10)
            .rev()
            .collect::<Vec<_>>();

        for msg in recent_messages {
            contents.push(GeminiContent {
                role: if msg.role == "user" { "user" } else { "model" }.to_string(),
                parts: vec![GeminiPart {
                    text: msg.content.clone(),
                }],
            });
        }

        // Add the current message
        contents.push(GeminiContent {
            role: "user".to_string(),
            parts: vec![GeminiPart {
                text: message.to_string(),
            }],
        });

        let request = GeminiRequest {
            contents,
            generation_config: Some(GenerationConfig {
                temperature: Some(0.7),
                top_k: Some(40),
                top_p: Some(0.95),
                max_output_tokens: Some(1024),
            }),
            safety_settings: Some(vec![
                SafetySetting {
                    category: "HARM_CATEGORY_HARASSMENT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                SafetySetting {
                    category: "HARM_CATEGORY_HATE_SPEECH".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                SafetySetting {
                    category: "HARM_CATEGORY_SEXUALLY_EXPLICIT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
                SafetySetting {
                    category: "HARM_CATEGORY_DANGEROUS_CONTENT".to_string(),
                    threshold: "BLOCK_MEDIUM_AND_ABOVE".to_string(),
                },
            ]),
        };

        let url = format!(
            "{}/models/gemini-1.5-flash-latest:generateContent?key={}",
            self.base_url, api_key
        );

        let response = self
            .client
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request to Gemini API: {}", e))?;

        if !response.status().is_success() {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Gemini API error: {}", error_text));
        }

        let gemini_response: GeminiResponse = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse Gemini API response: {}", e))?;

        // Extract the response text
        let response_text = gemini_response
            .candidates
            .first()
            .and_then(|candidate| candidate.content.parts.first())
            .map(|part| part.text.clone())
            .unwrap_or_else(|| "I'm sorry, I couldn't generate a response.".to_string());

        // Generate suggestions based on the context
        let suggestions = self.generate_suggestions(&conversation_history.course_context);

        Ok(ChatbotResponse {
            message: response_text,
            suggestions,
            context_used: conversation_history.course_context.clone(),
        })
    }

    fn generate_system_prompt(&self, context: &CourseContext) -> Result<String> {
        let mut prompt = String::from("You are an AI assistant helping with course learning. ");

        prompt.push_str(&format!("Course: {}\n", context.course_name));

        if let Some(structure) = &context.course_structure {
            prompt.push_str("Course Structure:\n");
            for (i, module) in structure.modules.iter().enumerate() {
                prompt.push_str(&format!("Module {}: {}\n", i + 1, module.title));
                for section in &module.sections {
                    prompt.push_str(&format!("  - {}\n", section.title));
                }
            }
        }

        match &context.source_type {
            CourseSourceType::YouTube { playlist_url } => {
                prompt.push_str(&format!("YouTube Playlist: {}\n", playlist_url));
            }
            CourseSourceType::Local { folder_path } => {
                prompt.push_str(&format!("Local Course Folder: {}\n", folder_path));
            }
        }

        if let Some(video_context) = &context.current_video_context {
            prompt.push_str(&format!(
                "Current Video: {} (Module: {})\n",
                video_context.video_title, video_context.module_title
            ));
        }

        prompt.push_str("\nHelp the user understand the course content, answer questions about the material, and provide learning guidance. Be concise and helpful.");

        Ok(prompt)
    }

    fn generate_suggestions(&self, context: &Option<CourseContext>) -> Vec<String> {
        let mut suggestions = vec![
            "Explain this topic".to_string(),
            "What should I focus on?".to_string(),
            "Quiz me on this content".to_string(),
        ];

        if let Some(ctx) = context {
            if ctx.current_video_context.is_some() {
                suggestions.push("Summarize this video".to_string());
                suggestions.push("What are the key points?".to_string());
            }

            if ctx.course_structure.is_some() {
                suggestions.push("What's next in the course?".to_string());
                suggestions.push("Show course progress".to_string());
            }
        }

        suggestions
    }

    pub async fn validate_api_key(api_key: &str) -> Result<bool> {
        let client = Client::new();
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models?key={}",
            api_key
        );

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to validate API key: {}", e))?;

        Ok(response.status().is_success())
    }
}

impl Default for GeminiClient {
    fn default() -> Self {
        Self::new()
    }
}
