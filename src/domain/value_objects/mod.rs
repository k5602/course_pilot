//! Value Objects - Immutable domain primitives.

mod exam_difficulty;
mod ids;
mod session;
mod tag_id;
mod video_quality;
mod video_source;
mod youtube;

pub use exam_difficulty::ExamDifficulty;
pub use ids::{CourseId, ExamId, ModuleId, UserId, VideoId};
pub use session::{CognitiveLimit, SessionPlan};
pub use tag_id::TagId;
pub use video_quality::VideoQuality;
pub use video_source::{VideoSource, VideoSourceError};
pub use youtube::{PlaylistUrl, YouTubeVideoId};
