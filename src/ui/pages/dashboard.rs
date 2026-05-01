use std::rc::Rc;

use adw::prelude::*;

use crate::ui::state::SharedState;
use crate::ui::toast::Toast;
use adw::NavigationView;

pub struct DashboardPage {
    widget: gtk::Box,
    state: SharedState,
    _nav: Rc<NavigationView>,
    stat_courses: gtk::Label,
    stat_modules: gtk::Label,
    stat_videos: gtk::Label,
    stat_completed: gtk::Label,
    progress_bar: gtk::LevelBar,
    progress_label: gtk::Label,
    coverage_label: gtk::Label,
    content_box: gtk::Box,
    status_page: adw::StatusPage,
}

impl DashboardPage {
    pub fn new(state: SharedState, nav: Rc<NavigationView>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");

        let heading = gtk::Label::new(Some("Dashboard"));
        heading.add_css_class("heading");
        widget.append(&heading);

        let status_page = adw::StatusPage::new();
        status_page.set_title("Loading...");
        status_page.set_description(Some("Loading analytics..."));
        widget.append(&status_page);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 16);
        content_box.set_margin_top(8);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);

        let stats_grid = gtk::Grid::new();
        stats_grid.set_column_spacing(16);
        stats_grid.set_row_spacing(16);
        stats_grid.set_halign(gtk::Align::Center);

        let stat_courses = make_stat_card("Total Courses", "0");
        let stat_modules = make_stat_card("Total Modules", "0");
        let stat_videos = make_stat_card("Total Videos", "0");
        let stat_completed = make_stat_card("Completed", "0%");

        stats_grid.attach(&stat_courses.0, 0, 0, 1, 1);
        stats_grid.attach(&stat_modules.0, 1, 0, 1, 1);
        stats_grid.attach(&stat_videos.0, 0, 1, 1, 1);
        stats_grid.attach(&stat_completed.0, 1, 1, 1, 1);

        content_box.append(&stats_grid);

        let progress_section = gtk::Box::new(gtk::Orientation::Vertical, 8);
        let progress_header = gtk::Label::new(Some("Completion Progress"));
        progress_header.add_css_class("subtitle");
        progress_header.set_halign(gtk::Align::Start);
        progress_section.append(&progress_header);

        let progress_bar = gtk::LevelBar::new();
        progress_bar.set_min_value(0.0);
        progress_bar.set_max_value(100.0);
        progress_bar.set_value(0.0);
        progress_bar.set_hexpand(true);
        progress_section.append(&progress_bar);

        let progress_label = gtk::Label::new(Some("0 / 0 videos completed"));
        progress_label.set_halign(gtk::Align::Start);
        progress_section.append(&progress_label);

        content_box.append(&progress_section);

        let coverage_section = gtk::Box::new(gtk::Orientation::Vertical, 4);
        let coverage_header = gtk::Label::new(Some("Summary Coverage"));
        coverage_header.add_css_class("subtitle");
        coverage_header.set_halign(gtk::Align::Start);

        let coverage_label = gtk::Label::new(Some("No summaries generated yet."));
        coverage_label.set_halign(gtk::Align::Start);
        coverage_section.append(&coverage_header);
        coverage_section.append(&coverage_label);

        content_box.append(&coverage_section);

        let subtitle = gtk::Label::new(Some("Welcome to Course Pilot."));
        subtitle.add_css_class("subtitle");
        widget.append(&subtitle);

        widget.append(&content_box);

        let s = Self {
            widget,
            state,
            _nav: nav,
            stat_courses: stat_courses.1,
            stat_modules: stat_modules.1,
            stat_videos: stat_videos.1,
            stat_completed: stat_completed.1,
            progress_bar,
            progress_label,
            coverage_label,
            content_box,
            status_page,
        };
        s.refresh();
        s
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn refresh(&self) {
        let state = self.state.borrow();
        if let Some(ref ctx) = state.backend {
            match crate::application::ServiceFactory::dashboard(ctx).execute() {
                Ok(analytics) => {
                    self.status_page.set_visible(false);
                    self.content_box.set_visible(true);

                    self.stat_courses.set_text(&analytics.total_courses().to_string());
                    self.stat_modules.set_text(&analytics.total_modules().to_string());
                    self.stat_videos.set_text(&analytics.total_videos().to_string());
                    self.stat_completed
                        .set_text(&format!("{:.0}%", analytics.completion_percent()));

                    let pct = analytics.completion_percent() as f64;
                    self.progress_bar.set_value(pct);
                    self.progress_label.set_text(&format!(
                        "{} / {} videos completed ({:.0}%)",
                        analytics.completed_videos(),
                        analytics.total_videos(),
                        pct
                    ));

                    let cov = analytics.summary_coverage_percent();
                    self.coverage_label.set_text(&format!(
                        "{:.0}% of videos have summaries ({})",
                        cov,
                        if cov > 50.0 {
                            "Great progress!"
                        } else if cov > 0.0 {
                            "Getting started."
                        } else {
                            "No summaries generated yet."
                        }
                    ));
                },
                Err(e) => {
                    Toast::show_error(&format!("Failed to load analytics: {}", e));
                    self.status_page.set_title("Error");
                    self.status_page
                        .set_description(Some(&format!("Failed to load analytics: {}", e)));
                    self.status_page.set_visible(true);
                    self.content_box.set_visible(false);
                },
            }
        } else {
            self.status_page.set_title("Welcome");
            self.status_page
                .set_description(Some("No backend connected. Start by importing a course."));
            self.status_page.set_visible(true);
            self.content_box.set_visible(false);
        }
    }
}

fn make_stat_card(title: &str, value: &str) -> (gtk::Frame, gtk::Label) {
    let frame = gtk::Frame::new(None);
    frame.set_width_request(180);
    frame.set_height_request(100);
    frame.add_css_class("card");

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
    vbox.set_margin_start(12);
    vbox.set_margin_end(12);
    vbox.set_margin_top(12);
    vbox.set_margin_bottom(12);
    vbox.set_valign(gtk::Align::Center);
    vbox.set_halign(gtk::Align::Center);

    let title_label = gtk::Label::new(Some(title));
    title_label.add_css_class("subtitle");

    let value_label = gtk::Label::new(Some(value));
    value_label.add_css_class("heading");
    value_label.set_markup(&format!("<b>{}</b>", value));

    vbox.append(&title_label);
    vbox.append(&value_label);
    frame.set_child(Some(&vbox));

    (frame, value_label)
}
