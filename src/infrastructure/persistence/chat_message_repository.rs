use std::str::FromStr;
use std::sync::Arc;

use diesel::prelude::*;

use crate::domain::ports::{ChatMessage, ChatMessageRepository, ChatRole, RepositoryError};
use crate::domain::value_objects::VideoId;
use crate::infrastructure::persistence::connection::DbPool;
use crate::infrastructure::persistence::models::{ChatMessageRow, NewChatMessage};
use crate::schema::chat_messages;

/// SQLite-backed companion chat history repository.
pub struct SqliteChatMessageRepository {
    pool: Arc<DbPool>,
}

impl SqliteChatMessageRepository {
    pub fn new(pool: Arc<DbPool>) -> Self {
        Self { pool }
    }

    fn row_to_entity(row: ChatMessageRow) -> Result<ChatMessage, RepositoryError> {
        let video_id = VideoId::from_str(&row.video_id)
            .map_err(|e| RepositoryError::Database(format!("Invalid video ID in chat: {}", e)))?;
        let role = match row.role.as_str() {
            "user" => ChatRole::User,
            "assistant" => ChatRole::Assistant,
            other => {
                return Err(RepositoryError::Database(format!("Invalid chat role: {}", other)));
            },
        };
        Ok(ChatMessage {
            id: row.id,
            video_id,
            role,
            content: row.content,
            created_at: row.created_at,
        })
    }
}

impl ChatMessageRepository for SqliteChatMessageRepository {
    fn save(&self, message: &ChatMessage) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;
        let role_str = match message.role {
            ChatRole::User => "user",
            ChatRole::Assistant => "assistant",
        };
        let new_msg = NewChatMessage {
            id: &message.id,
            video_id: &message.video_id.as_uuid().to_string(),
            role: role_str,
            content: &message.content,
            created_at: &message.created_at,
        };

        diesel::insert_into(chat_messages::table)
            .values(&new_msg)
            .on_conflict(chat_messages::id)
            .do_update()
            .set((
                chat_messages::content.eq(new_msg.content),
                chat_messages::created_at.eq(new_msg.created_at),
            ))
            .execute(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        Ok(())
    }

    fn find_by_video(&self, video_id: &VideoId) -> Result<Vec<ChatMessage>, RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;
        let rows: Vec<ChatMessageRow> = chat_messages::table
            .filter(chat_messages::video_id.eq(video_id.as_uuid().to_string()))
            .order(chat_messages::created_at.asc())
            .load(&mut conn)
            .map_err(|e| RepositoryError::Database(e.to_string()))?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    fn delete_by_video(&self, video_id: &VideoId) -> Result<(), RepositoryError> {
        let mut conn = self.pool.get().map_err(|e| RepositoryError::Database(e.to_string()))?;
        diesel::delete(
            chat_messages::table.filter(chat_messages::video_id.eq(video_id.as_uuid().to_string())),
        )
        .execute(&mut conn)
        .map_err(|e| RepositoryError::Database(e.to_string()))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entities::{Course, Module, Video};
    use crate::domain::ports::{CourseRepository, ModuleRepository, VideoRepository};
    use crate::domain::value_objects::{CourseId, ModuleId, PlaylistUrl};
    use crate::infrastructure::persistence::repositories::{
        SqliteCourseRepository, SqliteModuleRepository, SqliteVideoRepository,
    };

    #[test]
    fn test_chat_message_persistence_and_retrieval() {
        let pool =
            Arc::new(crate::infrastructure::persistence::establish_connection(":memory:").unwrap());

        let course_repo = SqliteCourseRepository::new(pool.clone());
        let module_repo = SqliteModuleRepository::new(pool.clone());
        let video_repo = SqliteVideoRepository::new(pool.clone());
        let chat_repo = SqliteChatMessageRepository::new(pool.clone());

        // 1. Create a course
        let course_id = CourseId::new();
        let playlist_url =
            PlaylistUrl::new("https://www.youtube.com/playlist?list=PL38E37F4BE52E385D").unwrap();
        let course = Course::new(
            course_id,
            "Test Course".to_string(),
            playlist_url,
            "PL38E37F4BE52E385D".to_string(),
            Some("Description".to_string()),
            None,
        );
        course_repo.save(&course).unwrap();

        // 2. Create a module
        let module_id = ModuleId::new();
        let module = Module::new(module_id, course_id, "Test Module".to_string(), 1);
        module_repo.save(&module).unwrap();

        // 3. Create a video
        let video_id = VideoId::new();
        let video_source =
            crate::domain::value_objects::VideoSource::local_path("/absolute/path/to/video.mp4")
                .unwrap();
        let video = Video::new(video_id, module_id, video_source, "Test Video".to_string(), 120, 1);
        video_repo.save(&video).unwrap();

        // 4. Save a chat message
        let msg_id = "test-msg-1".to_string();
        let msg = ChatMessage {
            id: msg_id.clone(),
            video_id,
            role: ChatRole::User,
            content: "Hello AI".to_string(),
            created_at: "2026-05-30T12:00:00Z".to_string(),
        };
        chat_repo.save(&msg).unwrap();

        // 5. Save an assistant message
        let assistant_msg_id = "test-msg-2".to_string();
        let assistant_msg = ChatMessage {
            id: assistant_msg_id.clone(),
            video_id,
            role: ChatRole::Assistant,
            content: "Hello User".to_string(),
            created_at: "2026-05-30T12:00:05Z".to_string(),
        };
        chat_repo.save(&assistant_msg).unwrap();

        // 6. Find messages by video and verify order
        let history = chat_repo.find_by_video(&video_id).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].id, msg_id);
        assert_eq!(history[0].role, ChatRole::User);
        assert_eq!(history[0].content, "Hello AI");
        assert_eq!(history[1].id, assistant_msg_id);
        assert_eq!(history[1].role, ChatRole::Assistant);
        assert_eq!(history[1].content, "Hello User");

        // 7. Delete by video
        chat_repo.delete_by_video(&video_id).unwrap();
        let history_after_delete = chat_repo.find_by_video(&video_id).unwrap();
        assert!(history_after_delete.is_empty());
    }
}
