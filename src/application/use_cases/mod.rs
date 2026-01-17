//! Use Cases - Application-level orchestration of domain logic.

mod ask_companion;
mod dashboard;
mod ingest_playlist;
mod notes;
mod plan_session;
mod preferences;
mod summarize_video;
mod take_exam;

pub use ask_companion::{AskCompanionInput, AskCompanionUseCase};
pub use dashboard::LoadDashboardUseCase;
pub use ingest_playlist::{IngestPlaylistInput, IngestPlaylistOutput, IngestPlaylistUseCase};
pub use notes::{
    DeleteNoteInput, LoadNoteInput, NoteView, NotesError, NotesUseCase, SaveNoteInput,
};
pub use plan_session::{PlanSessionInput, PlanSessionUseCase};
pub use preferences::{PreferencesUseCase, UpdatePreferencesInput};
pub use summarize_video::{
    SummarizeVideoError, SummarizeVideoInput, SummarizeVideoOutput, SummarizeVideoUseCase,
};
pub use take_exam::{GenerateExamInput, SubmitExamInput, SubmitExamOutput, TakeExamUseCase};
