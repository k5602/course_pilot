use std::rc::Rc;

use adw::prelude::*;

use crate::domain::ports::SecretStore;
use crate::ui::state::SharedState;

pub struct SettingsPage {
    widget: gtk::Box,
    state: SharedState,
    _stack: Rc<gtk::Stack>,
    api_key_entry: adw::EntryRow,
    api_status_label: gtk::Label,
    db_path_row: adw::ActionRow,
    discord_entry: adw::EntryRow,
    cookie_entry: adw::EntryRow,
    theme_switch: adw::SwitchRow,
    save_status_label: gtk::Label,
    save_btn: gtk::Button,
}

impl SettingsPage {
    pub fn new(state: SharedState, stack: Rc<gtk::Stack>) -> Self {
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

        prefs_box.append(&youtube_group);

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
            _stack: stack,
            api_key_entry,
            api_status_label,
            db_path_row,
            discord_entry,
            cookie_entry,
            theme_switch,
            save_status_label,
            save_btn,
        };

        let state_cl = state.clone();
        let api_entry = page.api_key_entry.clone();
        let discord_entry_cl = page.discord_entry.clone();
        let cookie_entry_cl = page.cookie_entry.clone();
        let theme_sw = page.theme_switch.clone();
        let status = page.save_status_label.clone();

        page.save_btn.connect_clicked(move |_| {
            let s = state_cl.borrow();
            if let Some(ref ctx) = s.backend {
                let key = api_entry.text().as_str().to_string();
                if !key.is_empty() {
                    match ctx.keystore.store("gemini_api_key", &key) {
                        Ok(_) => {
                            status.set_text("API key saved.");
                        },
                        Err(e) => {
                            status.set_text(&format!("Failed to save key: {}", e));
                        },
                    }
                }

                let discord_id = discord_entry_cl.text().as_str().to_string();
                if !discord_id.is_empty() {
                    let _ = ctx.keystore.store("discord_client_id", &discord_id);
                }

                let cookie_path = cookie_entry_cl.text().as_str().to_string();
                if !cookie_path.is_empty() {
                    let _ = ctx.keystore.store("youtube_cookies", &cookie_path);
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
                    self.api_status_label.set_text("API key is set");
                },
                _ => {
                    self.api_status_label.set_text("No API key set");
                },
            }

            if let Ok(Some(id)) = ctx.keystore.retrieve("discord_client_id") {
                self.discord_entry.set_text(&id);
            }

            if let Ok(Some(path)) = ctx.keystore.retrieve("youtube_cookies") {
                self.cookie_entry.set_text(&path);
            }

            let is_dark =
                matches!(adw::StyleManager::default().color_scheme(), adw::ColorScheme::ForceDark);
            self.theme_switch.set_active(is_dark);
        } else {
            self.db_path_row.set_subtitle("unknown (no backend)");
        }
    }
}
