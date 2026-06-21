//! Use Cases - Application-level orchestration of domain logic.

mod ask_companion;
mod chat;
mod create_module;
mod dashboard;
mod delete_module;
mod ingest_local;
mod ingest_playlist;
mod move_video_to_module;
mod notes;
mod preferences;
mod summarize_video;
mod take_exam;
mod update_module_title;
mod update_presence;

pub use ask_companion::{AskCompanionInput, AskCompanionUseCase};
pub use chat::{
    ChatError, ChatMessageView, ChatRole, ChatUseCase, DeleteChatHistoryInput,
    LoadChatHistoryInput, SendChatMessageInput,
};
pub use create_module::{CreateModuleError, CreateModuleInput, CreateModuleUseCase};
pub use dashboard::LoadDashboardUseCase;
pub use delete_module::{DeleteModuleError, DeleteModuleInput, DeleteModuleUseCase};
pub use ingest_local::{IngestLocalInput, IngestLocalOutput, IngestLocalUseCase};
pub use ingest_playlist::{
    IngestError, IngestPlaylistInput, IngestPlaylistOutput, IngestPlaylistUseCase,
};
pub use move_video_to_module::{MoveVideoError, MoveVideoInput, MoveVideoToModuleUseCase};
pub use notes::{
    DeleteNoteInput, LoadNoteInput, NoteView, NotesError, NotesUseCase, SaveNoteInput,
};
pub use preferences::{PreferencesUseCase, UpdatePreferencesInput};
pub use summarize_video::{
    SummarizeVideoError, SummarizeVideoInput, SummarizeVideoOutput, SummarizeVideoUseCase,
};
pub use take_exam::{GenerateExamInput, SubmitExamInput, SubmitExamOutput, TakeExamUseCase};
pub use update_module_title::{
    UpdateModuleTitleError, UpdateModuleTitleInput, UpdateModuleTitleUseCase,
};
pub use update_presence::{UpdatePresenceInput, UpdatePresenceUseCase};
