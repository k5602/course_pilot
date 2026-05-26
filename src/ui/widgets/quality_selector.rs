//! Reusable quality selector widgets.
//!
//! - [`QualitySelector`] wraps `adw::ComboRow` — correct for use inside an
//!   `adw::PreferencesGroup` (e.g. the Settings page).
//! - [`QualityDropDown`] wraps `gtk::DropDown` — compact, inline-friendly widget
//!   designed for the video player controls bar.

use adw::prelude::*;

use crate::domain::value_objects::VideoQuality;

const QUALITY_LABELS: &[&str] = &["240p", "360p", "480p", "720p", "1080p", "Best"];
const QUALITY_VARIANTS: &[VideoQuality] = &[
    VideoQuality::P240,
    VideoQuality::P360,
    VideoQuality::P480,
    VideoQuality::P720,
    VideoQuality::P1080,
    VideoQuality::Best,
];

// ---------------------------------------------------------------------------
// QualitySelector — adw::ComboRow, for PreferencesGroup (Settings page)
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct QualitySelector {
    row: adw::ComboRow,
}

impl QualitySelector {
    pub fn new(title: &str) -> Self {
        let model = gtk::StringList::new(QUALITY_LABELS);
        let row = adw::ComboRow::new();
        row.set_title(title);
        row.set_model(Some(&model));
        Self { row }
    }

    pub fn selected_quality(&self) -> VideoQuality {
        let selected = self.row.selected();
        let idx = selected as usize;
        if idx < QUALITY_VARIANTS.len() { QUALITY_VARIANTS[idx] } else { VideoQuality::P720 }
    }

    pub fn set_quality(&self, quality: VideoQuality) {
        let idx = QUALITY_VARIANTS.iter().position(|&v| v == quality).unwrap_or(3);
        self.row.set_selected(idx as u32);
    }

    pub fn connect_selected<F: Fn(VideoQuality) + 'static>(&self, f: F) {
        let row_clone = self.row.clone();
        self.row.connect_selected_item_notify(move |_| {
            let selected = row_clone.selected();
            let idx = selected as usize;
            let quality = if idx < QUALITY_VARIANTS.len() {
                QUALITY_VARIANTS[idx]
            } else {
                VideoQuality::P720
            };
            f(quality);
        });
    }

    pub fn widget(&self) -> &adw::ComboRow {
        &self.row
    }
}

// ---------------------------------------------------------------------------
// QualityDropDown — gtk::DropDown, compact, for the video player controls bar
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct QualityDropDown {
    dropdown: gtk::DropDown,
}

impl QualityDropDown {
    pub fn new() -> Self {
        let model = gtk::StringList::new(QUALITY_LABELS);
        let dropdown = gtk::DropDown::new(Some(model), gtk::Expression::NONE);
        dropdown.add_css_class("quality-dropdown");
        // Default to 720p (index 3)
        dropdown.set_selected(3);
        Self { dropdown }
    }

    pub fn selected_quality(&self) -> VideoQuality {
        let idx = self.dropdown.selected() as usize;
        if idx < QUALITY_VARIANTS.len() { QUALITY_VARIANTS[idx] } else { VideoQuality::P720 }
    }

    pub fn set_quality(&self, quality: VideoQuality) {
        let idx = QUALITY_VARIANTS.iter().position(|&v| v == quality).unwrap_or(3);
        self.dropdown.set_selected(idx as u32);
    }

    pub fn connect_selected<F: Fn(VideoQuality) + 'static>(&self, f: F) {
        let dd = self.dropdown.clone();
        self.dropdown.connect_selected_notify(move |_| {
            let idx = dd.selected() as usize;
            let quality = if idx < QUALITY_VARIANTS.len() {
                QUALITY_VARIANTS[idx]
            } else {
                VideoQuality::P720
            };
            f(quality);
        });
    }

    pub fn widget(&self) -> &gtk::DropDown {
        &self.dropdown
    }
}

impl Default for QualityDropDown {
    fn default() -> Self {
        Self::new()
    }
}
