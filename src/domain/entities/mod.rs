//! Domain Entities - Core business objects with identity.

mod analytics;
mod course;
mod exam;
mod module;
mod note;
mod search;
mod tag;
mod user_preferences;
mod video;

pub use analytics::AppAnalytics;
pub use course::Course;
pub use exam::Exam;
pub use module::Module;
pub use note::{Note, NoteId};
pub use search::{SearchResult, SearchResultType};
pub use tag::{TAG_COLORS, Tag};
pub use user_preferences::UserPreferences;
pub use video::Video;
