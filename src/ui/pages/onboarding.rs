use adw::prelude::{AdwDialogExt, AlertDialogExt};

use crate::application::ServiceFactory;
use crate::application::use_cases::UpdatePreferencesInput;
use crate::ui::state::SharedState;

pub fn check_onboarding(state: SharedState, window: &adw::ApplicationWindow) {
    // Read completion status from the loaded state (populated from DB in with_backend)
    let already_done = {
        let s = state.borrow();
        s.onboarding_completed
    };

    if already_done {
        return;
    }

    // Mark as completed in memory and persist to DB before showing dialog
    // so that even if the app crashes, it won't show again.
    {
        let mut s = state.borrow_mut();
        s.onboarding_completed = true;

        if let Some(ref ctx) = s.backend {
            let uc = ServiceFactory::preferences(ctx);
            let input = UpdatePreferencesInput {
                ml_boundary_enabled: false,
                cognitive_limit_minutes: s.cognitive_limit_minutes,
                boundary_batch_size: s.boundary_batch_size,
                right_panel_visible: s.right_panel_visible,
                right_panel_width: s.right_panel_width as u32,
                onboarding_completed: true,
                preferred_quality: s.preferred_quality,
            };
            if let Err(e) = uc.update(input) {
                log::warn!("Failed to persist onboarding completion: {e}");
            }
        }
    }

    let dialog = adw::AlertDialog::new(
        Some("Welcome to Course Pilot"),
        Some("Import a YouTube playlist or local media to get started with your learning journey."),
    );
    dialog.add_response("ok", "Get Started");
    dialog.set_default_response(Some("ok"));
    dialog.present(Some(window));
}
