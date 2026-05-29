use std::rc::Rc;

use adw::NavigationView;
use adw::prelude::*;

use crate::application::ServiceFactory;
use crate::domain::ports::SecretStore;
use crate::domain::value_objects::VideoQuality;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;
use crate::ui::widgets::QualitySelector;

/// Placeholder shown in the API-key entry when a key is already stored.
/// If the user submits this exact string, the key is left unchanged.
const MASKED_KEY: &str = "●●●●●●●●";

pub struct SettingsPage {
    widget: gtk::Box,
    state: SharedState,
    _nav: Rc<NavigationView>,
    api_key_entry: adw::EntryRow,
    api_status_label: gtk::Label,
    db_path_row: adw::ActionRow,
    discord_entry: adw::EntryRow,
    cookie_entry: adw::EntryRow,
    theme_switch: adw::SwitchRow,
    quality_selector: QualitySelector,
    cognitive_limit_row: adw::SpinRow,
    batch_size_row: adw::SpinRow,
    save_status_label: gtk::Label,
    save_btn: gtk::Button,
}

impl SettingsPage {
    pub fn new(state: SharedState, nav: Rc<NavigationView>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");

        let heading = gtk::Label::new(Some("Settings"));
        heading.add_css_class("heading");
        heading.set_margin_start(16);
        heading.set_margin_top(16);
        widget.append(&heading);

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let prefs_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        prefs_box.set_margin_start(16);
        prefs_box.set_margin_end(16);
        prefs_box.set_margin_bottom(16);

        let api_group = adw::PreferencesGroup::new();
        api_group.set_title("Gemini API");
        api_group
            .set_description(Some("API key for AI companion, quiz generation, and summaries."));

        let api_key_entry = adw::EntryRow::new();
        api_key_entry.set_title("Gemini API Key");
        api_key_entry.set_input_purpose(gtk::InputPurpose::Password);
        api_group.add(&api_key_entry);

        let api_status_label = gtk::Label::new(None);
        api_status_label.set_halign(gtk::Align::Start);
        api_status_label.add_css_class("subtitle");
        api_group.add(&api_status_label);

        prefs_box.append(&api_group);

        let db_group = adw::PreferencesGroup::new();
        db_group.set_title("Database");

        let db_path_row = adw::ActionRow::new();
        db_path_row.set_title("Database Path");
        db_path_row.set_subtitle("unknown");
        db_group.add(&db_path_row);

        prefs_box.append(&db_group);

        let discord_group = adw::PreferencesGroup::new();
        discord_group.set_title("Discord");
        discord_group.set_description(Some("Discord Rich Presence client ID (optional)."));

        let discord_entry = adw::EntryRow::new();
        discord_entry.set_title("Discord Client ID");
        discord_group.add(&discord_entry);

        prefs_box.append(&discord_group);

        let youtube_group = adw::PreferencesGroup::new();
        youtube_group.set_title("YouTube");
        youtube_group.set_description(Some("Cookies file path for restricted content (optional)."));

        let cookie_entry = adw::EntryRow::new();
        cookie_entry.set_title("Cookies File Path");
        youtube_group.add(&cookie_entry);

        let quality_selector = QualitySelector::new("Preferred Quality");
        quality_selector.widget().set_subtitle("Default quality for YouTube videos.");
        youtube_group.add(quality_selector.widget());

        prefs_box.append(&youtube_group);

        let learning_group = adw::PreferencesGroup::new();
        learning_group.set_title("Learning");
        learning_group.set_description(Some("Module grouping and session planning."));

        let cognitive_limit_row = adw::SpinRow::new(None::<&gtk::Adjustment>, 1.0, 0);
        cognitive_limit_row.set_title("Daily Study Limit");
        cognitive_limit_row.set_subtitle("Maximum minutes per day for study sessions.");
        cognitive_limit_row.set_range(10.0, 180.0);
        cognitive_limit_row.set_value(45.0);
        cognitive_limit_row.set_digits(0);
        learning_group.add(&cognitive_limit_row);

        let batch_size_row = adw::SpinRow::new(None::<&gtk::Adjustment>, 1.0, 0);
        batch_size_row.set_title("Module Batch Size");
        batch_size_row.set_subtitle("Videos per module when no labeled boundaries detected.");
        batch_size_row.set_range(1.0, 20.0);
        batch_size_row.set_value(5.0);
        batch_size_row.set_digits(0);
        learning_group.add(&batch_size_row);

        prefs_box.append(&learning_group);

        let theme_group = adw::PreferencesGroup::new();
        theme_group.set_title("Appearance");

        let theme_switch = adw::SwitchRow::new();
        theme_switch.set_title("Dark Mode");
        theme_switch.set_subtitle("Use dark color scheme.");
        theme_group.add(&theme_switch);

        prefs_box.append(&theme_group);

        let save_btn = gtk::Button::with_label("Save Settings");
        save_btn.add_css_class("suggested-action");
        save_btn.set_halign(gtk::Align::Center);
        prefs_box.append(&save_btn);

        let save_status_label = gtk::Label::new(None);
        save_status_label.set_halign(gtk::Align::Center);
        save_status_label.add_css_class("subtitle");
        prefs_box.append(&save_status_label);

        scroll.set_child(Some(&prefs_box));
        widget.append(&scroll);

        let page = Self {
            widget,
            state: state.clone(),
            _nav: nav,
            api_key_entry,
            api_status_label,
            db_path_row,
            discord_entry,
            cookie_entry,
            theme_switch,
            quality_selector,
            cognitive_limit_row,
            batch_size_row,
            save_status_label,
            save_btn,
        };

        let state_cl = state.clone();
        let api_entry = page.api_key_entry.clone();
        let discord_entry_cl = page.discord_entry.clone();
        let cookie_entry_cl = page.cookie_entry.clone();
        let theme_sw = page.theme_switch.clone();
        let status = page.save_status_label.clone();
        let quality_sel = page.quality_selector.widget().clone();
        let cognitive_limit_row_cl = page.cognitive_limit_row.clone();
        let batch_size_row_cl = page.batch_size_row.clone();

        // When user starts typing in the API key entry, clear the masked placeholder
        // so the real key can be entered fresh.
        let api_entry_change = page.api_key_entry.clone();
        api_entry_change.connect_changed(move |entry| {
            let text = entry.text();
            // If the text still equals the placeholder AND has the placeholder length,
            // it means the user hasn't modified it yet — do nothing.
            // Once it differs (user added/removed a char) the natural text is in place.
            if text.as_str() != MASKED_KEY && text.as_str().contains('●') {
                // User started editing inside the masked value — clear the field so they
                // can enter the new key cleanly.
                entry.set_text("");
            }
        });

        page.save_btn.connect_clicked(move |_| {
            let s = state_cl.borrow();
            if let Some(ref ctx) = s.backend {
                let key = api_entry.text().as_str().trim().to_string();
                // Only save if the user actually typed a new key (not the masked placeholder).
                if !key.is_empty() && key != MASKED_KEY {
                    match ctx.keystore.store("gemini_api_key", &key) {
                        Ok(_) => {
                            // Hot-swap the LLM so summarize/quiz buttons become active
                            // immediately without requiring an app restart.
                            if let Err(e) = ctx.reload_llm() {
                                log::warn!("reload_llm failed after API key save: {e}");
                            }
                            status.set_text("API key saved.");
                        },
                        Err(e) => {
                            status.set_text(&format!("Failed to save key: {}", e));
                        },
                    }
                }

                let discord_id = discord_entry_cl.text().as_str().to_string();
                if !discord_id.is_empty()
                    && let Err(e) = ctx.keystore.store("discord_client_id", &discord_id)
                {
                    Toast::show_error(&format!("Failed to save Discord client ID: {}", e));
                }

                let cookie_path = cookie_entry_cl.text().as_str().to_string();
                if !cookie_path.is_empty()
                    && let Err(e) = ctx.keystore.store("youtube_cookies", &cookie_path)
                {
                    Toast::show_error(&format!("Failed to save cookies path: {}", e));
                }

                let selected = quality_sel.selected();
                let idx = selected as usize;
                let quality = [
                    VideoQuality::P240,
                    VideoQuality::P360,
                    VideoQuality::P480,
                    VideoQuality::P720,
                    VideoQuality::P1080,
                    VideoQuality::Best,
                ]
                .get(idx)
                .copied()
                .unwrap_or(VideoQuality::P720);

                use crate::application::use_cases::UpdatePreferencesInput;
                let uc = ServiceFactory::preferences(ctx);
                let input = UpdatePreferencesInput {
                    ml_boundary_enabled: false,
                    cognitive_limit_minutes: cognitive_limit_row_cl.value() as u32,
                    boundary_batch_size: batch_size_row_cl.value() as u32,
                    right_panel_visible: s.right_panel_visible,
                    right_panel_width: s.right_panel_width as u32,
                    onboarding_completed: s.onboarding_completed,
                    preferred_quality: quality,
                };
                match uc.update(input) {
                    Ok(prefs) => {
                        drop(s);
                        let mut s2 = state_cl.borrow_mut();
                        s2.preferred_quality = prefs.preferred_quality();
                        s2.session_quality = prefs.preferred_quality();
                        s2.cognitive_limit_minutes = prefs.cognitive_limit_minutes();
                        status.set_text("Settings saved.");
                    },
                    Err(e) => {
                        status.set_text(&format!("Failed to save: {e}"));
                    },
                }
            } else {
                status.set_text("No backend connected.");
            }

            if theme_sw.is_active() {
                adw::StyleManager::default().set_color_scheme(adw::ColorScheme::ForceDark);
            } else {
                adw::StyleManager::default().set_color_scheme(adw::ColorScheme::Default);
            }
        });

        page.refresh();
        page
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn refresh(&self) {
        let state = self.state.borrow();
        if let Some(ref ctx) = state.backend {
            self.db_path_row.set_subtitle(&ctx.config.database_url);

            match ctx.keystore.retrieve("gemini_api_key") {
                Ok(Some(_)) => {
                    // Show a visual placeholder to indicate a key is stored,
                    // without exposing the actual key value.
                    self.api_key_entry.set_text(MASKED_KEY);
                    self.api_status_label.set_text("API key is set");
                },
                _ => {
                    self.api_key_entry.set_text("");
                    self.api_status_label.set_text("No API key set");
                },
            }

            if let Ok(Some(id)) = ctx.keystore.retrieve("discord_client_id") {
                self.discord_entry.set_text(&id);
            }

            if let Ok(Some(path)) = ctx.keystore.retrieve("youtube_cookies") {
                self.cookie_entry.set_text(&path);
            }

            self.quality_selector.set_quality(state.preferred_quality);

            self.batch_size_row.set_value(state.boundary_batch_size as f64);
            self.cognitive_limit_row.set_value(state.cognitive_limit_minutes as f64);

            let is_dark =
                matches!(adw::StyleManager::default().color_scheme(), adw::ColorScheme::ForceDark);
            self.theme_switch.set_active(is_dark);
        } else {
            self.db_path_row.set_subtitle("unknown (no backend)");
        }
    }
}
