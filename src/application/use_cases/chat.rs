//! Chat history use case for the AI companion panel.
//!
//! Orchestrates ChatMessageRepository with video context, providing
//! message persistence and history loading for the right-panel chat UI.

use std::sync::Arc;

use crate::domain::ports::{ChatMessageRepository, RepositoryError, VideoRepository};
use crate::domain::value_objects::VideoId;

/// Error type for chat operations.
#[derive(Debug, thiserror::Error)]
pub enum ChatError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Input for sending a chat message.
pub struct SendChatMessageInput {
    pub video_id: VideoId,
    pub role: ChatRole,
    pub content: String,
}

/// Input for loading chat history.
pub struct LoadChatHistoryInput {
    pub video_id: VideoId,
}

/// Input for deleting chat history.
pub struct DeleteChatHistoryInput {
    pub video_id: VideoId,
}

/// Chat role mirroring the domain type for UI consumption.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChatRole {
    User,
    Assistant,
}

/// View model for a chat message returned to the UI.
#[derive(Clone, Debug)]
pub struct ChatMessageView {
    pub id: String,
    pub video_id: VideoId,
    pub role: ChatRole,
    pub content: String,
    pub created_at: String,
}

/// Use case for chat message persistence and history management.
pub struct ChatUseCase {
    chat_repo: Arc<dyn ChatMessageRepository>,
    #[allow(dead_code)]
    video_repo: Arc<dyn VideoRepository>,
}

impl ChatUseCase {
    pub fn new(
        chat_repo: Arc<dyn ChatMessageRepository>,
        video_repo: Arc<dyn VideoRepository>,
    ) -> Self {
        Self { chat_repo, video_repo }
    }

    /// Generates a UUID, creates a timestamp, saves to repo, returns view.
    pub fn send_message(&self, input: SendChatMessageInput) -> Result<ChatMessageView, ChatError> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();

        let domain_role = match input.role {
            ChatRole::User => crate::domain::ports::ChatRole::User,
            ChatRole::Assistant => crate::domain::ports::ChatRole::Assistant,
        };

        let msg = crate::domain::ports::ChatMessage {
            id: id.clone(),
            video_id: input.video_id,
            role: domain_role,
            content: input.content.clone(),
            created_at: created_at.clone(),
        };

        self.chat_repo.save(&msg)?;

        Ok(ChatMessageView {
            id,
            video_id: input.video_id,
            role: input.role,
            content: input.content,
            created_at,
        })
    }

    /// Loads all chat messages for a video from the repository.
    pub fn load_history(
        &self,
        input: LoadChatHistoryInput,
    ) -> Result<Vec<ChatMessageView>, ChatError> {
        let messages = self.chat_repo.find_by_video(&input.video_id)?;

        let views = messages
            .into_iter()
            .map(|msg| ChatMessageView {
                id: msg.id,
                video_id: msg.video_id,
                role: match msg.role {
                    crate::domain::ports::ChatRole::User => ChatRole::User,
                    crate::domain::ports::ChatRole::Assistant => ChatRole::Assistant,
                },
                content: msg.content,
                created_at: msg.created_at,
            })
            .collect();

        Ok(views)
    }

    /// Deletes all chat messages for a video.
    pub fn delete_history(&self, input: DeleteChatHistoryInput) -> Result<(), ChatError> {
        self.chat_repo.delete_by_video(&input.video_id)?;
        Ok(())
    }
}
