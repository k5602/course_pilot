use crate::domain::value_objects::{CourseId, ExamId, VideoId};
use tokio::sync::broadcast::Receiver;

#[derive(Clone, Debug)]
pub enum DomainEvent {
    CourseIngested(CourseId),
    VideoCompleted { video_id: VideoId, completed: bool },
    NotesUpdated(VideoId),
    QuizGenerated { exam_id: ExamId, video_id: VideoId },
}

pub trait EventBus: Send + Sync {
    fn publish(&self, event: DomainEvent);
    fn subscribe(&self) -> Receiver<DomainEvent>;
}
