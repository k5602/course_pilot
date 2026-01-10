//! Domain Entities - Core business objects with identity.

mod course;
mod exam;
mod module;
mod video;

pub use course::Course;
pub use exam::Exam;
pub use module::Module;
pub use video::Video;
