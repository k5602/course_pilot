use adw::prelude::*;

use crate::ui::css;
use crate::ui::layout::MainLayout;
use crate::ui::pages::onboarding;
use crate::ui::state::new_shared_state;

pub struct CoursePilotApp;

impl CoursePilotApp {
    pub fn activate(app: &adw::Application) {
        gio::resources_register_include!("course_pilot.gresource")
            .expect("Failed to register GResource");

        css::load_theme();

        adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);

        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Course Pilot")
            .default_width(1280)
            .default_height(800)
            .build();

        window.connect_close_request(move |_| {
            log::info!("Main window closed.");
            glib::Propagation::Proceed
        });

        let state = new_shared_state();
        let layout = MainLayout::build(state.clone(), &window);
        window.set_content(Some(&layout));
        onboarding::check_onboarding(state, &window);
        window.present();
    }
}
