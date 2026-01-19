//! Value Objects - Immutable domain primitives.

mod ids;
mod session;
mod tag_id;
mod video_source;
mod youtube;

pub use ids::{CourseId, ExamId, ModuleId, VideoId};
pub use session::{CognitiveLimit, SessionPlan};
pub use tag_id::TagId;
pub use video_source::{VideoSource, VideoSourceError};
pub use youtube::{PlaylistUrl, YouTubeVideoId};
