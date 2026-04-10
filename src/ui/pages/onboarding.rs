use adw::prelude::{AdwDialogExt, AlertDialogExt};

use crate::ui::state::SharedState;

pub fn check_onboarding(state: SharedState, window: &adw::ApplicationWindow) {
    let mut s = state.borrow_mut();
    if s.onboarding_completed {
        return;
    }
    s.onboarding_completed = true;
    drop(s);

    let dialog = adw::AlertDialog::new(
        Some("Welcome to Course Pilot"),
        Some("Import a YouTube playlist or local media to get started with your learning journey."),
    );
    dialog.add_response("ok", "Get Started");
    dialog.set_default_response(Some("ok"));
    dialog.present(Some(window));
}
