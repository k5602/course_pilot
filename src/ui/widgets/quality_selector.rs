//! Reusable quality selector widget backed by an adw::ComboRow.

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
