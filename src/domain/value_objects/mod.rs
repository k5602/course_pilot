//! Value Objects - Immutable domain primitives.

mod embedding;
mod ids;
mod session;
mod youtube;

pub use embedding::Embedding;
pub use ids::{CourseId, ExamId, ModuleId, VideoId};
pub use session::{CognitiveLimit, SessionPlan};
pub use youtube::{PlaylistUrl, YouTubeVideoId};
