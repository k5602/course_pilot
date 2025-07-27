use crate::types::Note;
use crate::ui::components::toast::toast;
use dioxus::prelude::*;
use uuid::Uuid;

/// Hook for loading notes with video index
pub fn use_notes_with_video_index_resource(
    course_id: Uuid,
    video_index: Option<usize>,
) -> Resource<Result<Vec<Note>, anyhow::Error>> {
    let backend = crate::ui::hooks::use_backend_adapter();

    use_resource(move || {
        let backend = backend.clone();
        async move {
            match video_index {
                Some(index) => backend.list_notes_by_video_index(course_id, index).await,
                None => backend.list_notes_by_course(course_id).await,
            }
        }
    })
}

/// Hook for saving notes
pub fn use_save_note_action() -> Callback<Note> {
    let backend = crate::ui::hooks::use_backend_adapter();

    use_callback(move |note: Note| {
        let backend = backend.clone();
        spawn(async move {
            match backend.save_note(note).await {
                Ok(_) => {
                    toast::success("Note saved successfully");
                }
                Err(e) => {
                    toast::error(format!("Failed to save note: {e}"));
                }
            }
        });
    })
}

/// Hook for deleting notes
pub fn use_delete_note_action() -> Callback<Uuid> {
    let backend = crate::ui::hooks::use_backend_adapter();

    use_callback(move |note_id: Uuid| {
        let backend = backend.clone();
        spawn(async move {
            match backend.delete_note(note_id).await {
                Ok(_) => {
                    toast::success("Note deleted successfully");
                }
                Err(e) => {
                    toast::error(format!("Failed to delete note: {e}"));
                }
            }
        });
    })
}

/// Hook for loading all notes
pub fn use_all_notes_resource() -> Resource<Result<Vec<Note>, anyhow::Error>> {
    let backend = crate::ui::hooks::use_backend_adapter();

    use_resource(move || {
        let backend = backend.clone();
        async move { backend.list_all_notes().await }
    })
}
