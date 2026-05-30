use crate::domain::ports::{DomainEvent, EventBus};
use tokio::sync::broadcast::{Receiver, Sender, channel};

/// An in-memory thread-safe event bus using Tokio broadcast channels.
pub struct InMemoryEventBus {
    sender: Sender<DomainEvent>,
}

impl InMemoryEventBus {
    /// Creates a new `InMemoryEventBus` with a specified capacity.
    pub fn new() -> Self {
        let (sender, _) = channel(100);
        Self { sender }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus for InMemoryEventBus {
    fn publish(&self, event: DomainEvent) {
        // Discard result as it's fine if there are no active subscribers.
        let _ = self.sender.send(event);
    }

    fn subscribe(&self) -> Receiver<DomainEvent> {
        self.sender.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::{CourseId, ExamId, VideoId};

    #[tokio::test]
    async fn test_event_bus_concurrent_subscribers() {
        let bus = InMemoryEventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        let course_id = CourseId::new();
        bus.publish(DomainEvent::CourseIngested(course_id));

        let event1 = rx1.recv().await.expect("rx1 failed to receive event");
        let event2 = rx2.recv().await.expect("rx2 failed to receive event");

        if let (DomainEvent::CourseIngested(id1), DomainEvent::CourseIngested(id2)) =
            (event1, event2)
        {
            assert_eq!(id1, course_id);
            assert_eq!(id2, course_id);
        } else {
            panic!("Expected CourseIngested events");
        }
    }

    #[tokio::test]
    async fn test_event_bus_video_completed() {
        let bus = InMemoryEventBus::new();
        let mut rx = bus.subscribe();

        let video_id = VideoId::new();
        bus.publish(DomainEvent::VideoCompleted { video_id, completed: true });

        let event = rx.recv().await.expect("Failed to receive video completed");
        if let DomainEvent::VideoCompleted { video_id: vid, completed } = event {
            assert_eq!(vid, video_id);
            assert!(completed);
        } else {
            panic!("Expected VideoCompleted event");
        }
    }

    #[tokio::test]
    async fn test_event_bus_notes_updated() {
        let bus = InMemoryEventBus::new();
        let mut rx = bus.subscribe();

        let video_id = VideoId::new();
        bus.publish(DomainEvent::NotesUpdated(video_id));

        let event = rx.recv().await.expect("Failed to receive notes updated");
        if let DomainEvent::NotesUpdated(vid) = event {
            assert_eq!(vid, video_id);
        } else {
            panic!("Expected NotesUpdated event");
        }
    }

    #[tokio::test]
    async fn test_event_bus_quiz_generated() {
        let bus = InMemoryEventBus::new();
        let mut rx = bus.subscribe();

        let exam_id = ExamId::new();
        let video_id = VideoId::new();
        bus.publish(DomainEvent::QuizGenerated { exam_id, video_id });

        let event = rx.recv().await.expect("Failed to receive quiz generated");
        if let DomainEvent::QuizGenerated { exam_id: eid, video_id: vid } = event {
            assert_eq!(eid, exam_id);
            assert_eq!(vid, video_id);
        } else {
            panic!("Expected QuizGenerated event");
        }
    }
}
