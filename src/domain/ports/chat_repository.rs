use serde::{Deserialize, Serialize};

use crate::domain::ports::RepositoryError;
use crate::domain::value_objects::VideoId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChatRole {
    User,
    Assistant,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub video_id: VideoId,
    pub role: ChatRole,
    pub content: String,
    pub created_at: String,
}

pub trait ChatMessageRepository: Send + Sync {
    fn save(&self, message: &ChatMessage) -> Result<(), RepositoryError>;
    fn find_by_video(&self, video_id: &VideoId) -> Result<Vec<ChatMessage>, RepositoryError>;
    fn delete_by_video(&self, video_id: &VideoId) -> Result<(), RepositoryError>;
}
