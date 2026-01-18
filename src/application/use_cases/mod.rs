//! Use Cases - Application-level orchestration of domain logic.

mod ask_companion;
mod dashboard;
mod export_course_notes;
mod ingest_playlist;
mod move_video_to_module;
mod notes;
mod plan_session;
mod preferences;
mod summarize_video;
mod take_exam;
mod update_course;
mod update_module_title;

pub use ask_companion::{AskCompanionInput, AskCompanionUseCase};
pub use dashboard::LoadDashboardUseCase;
pub use export_course_notes::{
    ExportCourseNotesError, ExportCourseNotesInput, ExportCourseNotesUseCase,
};
pub use ingest_playlist::{IngestPlaylistInput, IngestPlaylistOutput, IngestPlaylistUseCase};
pub use move_video_to_module::{MoveVideoError, MoveVideoInput, MoveVideoToModuleUseCase};
pub use notes::{
    DeleteNoteInput, LoadNoteInput, NoteView, NotesError, NotesUseCase, SaveNoteInput,
};
pub use plan_session::{PlanSessionInput, PlanSessionUseCase};
pub use preferences::{PreferencesUseCase, UpdatePreferencesInput};
pub use summarize_video::{
    SummarizeVideoError, SummarizeVideoInput, SummarizeVideoOutput, SummarizeVideoUseCase,
};
pub use take_exam::{GenerateExamInput, SubmitExamInput, SubmitExamOutput, TakeExamUseCase};
pub use update_course::{
    UpdateCourseError, UpdateCourseInput, UpdateCourseOutput, UpdateCourseUseCase,
};
pub use update_module_title::{
    UpdateModuleTitleError, UpdateModuleTitleInput, UpdateModuleTitleUseCase,
};
