use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use adw::{NavigationPage, NavigationView};

use crate::domain::ports::{CourseRepository, VideoRepository};
use crate::ui::navigation::PAGE_COURSE_VIEW;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;

pub struct DashboardPage {
    widget: gtk::Box,
    state: SharedState,
    nav: Rc<NavigationView>,
    nav_pages: Rc<RefCell<Rc<HashMap<&'static str, NavigationPage>>>>,
    stat_courses: gtk::Label,
    stat_modules: gtk::Label,
    stat_videos: gtk::Label,
    stat_completed: gtk::Label,
    progress_bar: gtk::LevelBar,
    progress_label: gtk::Label,
    coverage_label: gtk::Label,
    courses_container: gtk::Box,
    content_box: gtk::Box,
    status_page: adw::StatusPage,
}

impl DashboardPage {
    pub fn new(state: SharedState, nav: Rc<NavigationView>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 0);
        widget.add_css_class("content-area");

        let status_page = adw::StatusPage::new();
        status_page.set_title("Loading...");
        status_page.set_description(Some("Loading analytics..."));
        widget.append(&status_page);

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 24);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);
        content_box.set_margin_bottom(24);
        scroll.set_child(Some(&content_box));

        // 1. Beautiful Dashboard Hero Banner
        let hero_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
        hero_box.add_css_class("dashboard-hero");

        let hero_title = gtk::Label::new(Some("Welcome back, Scholar!"));
        hero_title.add_css_class("heading");
        hero_title.set_halign(gtk::Align::Start);
        hero_box.append(&hero_title);

        let hero_desc = gtk::Label::new(Some(
            "Ready to expand your horizon? Select an in-progress course below to resume studying or click the '+' in the top right to ingest new material.",
        ));
        hero_desc.add_css_class("subtitle");
        hero_desc.set_wrap(true);
        hero_desc.set_halign(gtk::Align::Start);
        hero_box.append(&hero_desc);

        content_box.append(&hero_box);

        // 2. Glassmorphic Stats Grid Row
        let stats_grid = gtk::Grid::new();
        stats_grid.set_column_spacing(16);
        stats_grid.set_row_spacing(16);
        stats_grid.set_column_homogeneous(true);

        let stat_courses = make_stat_card("Enrolled Courses", "0");
        let stat_modules = make_stat_card("Modules Scheduled", "0");
        let stat_videos = make_stat_card("Total Lessons", "0");
        let stat_completed = make_stat_card("Completed Ratio", "0%");

        stats_grid.attach(&stat_courses.0, 0, 0, 1, 1);
        stats_grid.attach(&stat_modules.0, 1, 0, 1, 1);
        stats_grid.attach(&stat_videos.0, 2, 0, 1, 1);
        stats_grid.attach(&stat_completed.0, 3, 0, 1, 1);

        content_box.append(&stats_grid);

        // 3. Overall Completion Progress Section
        let progress_section = gtk::Frame::new(None);
        progress_section.add_css_class("card");

        let progress_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
        progress_box.set_margin_start(8);
        progress_box.set_margin_end(8);
        progress_box.set_margin_top(8);
        progress_box.set_margin_bottom(8);

        let progress_header = gtk::Label::new(Some("Global Learning Progress"));
        progress_header.add_css_class("title");
        progress_header.set_halign(gtk::Align::Start);
        progress_box.append(&progress_header);

        let progress_bar = gtk::LevelBar::new();
        progress_bar.set_min_value(0.0);
        progress_bar.set_max_value(100.0);
        progress_bar.set_value(0.0);
        progress_bar.set_hexpand(true);
        progress_box.append(&progress_bar);

        let progress_label = gtk::Label::new(Some("0 / 0 videos watched (0%)"));
        progress_label.set_halign(gtk::Align::Start);
        progress_label.add_css_class("subtitle");
        progress_box.append(&progress_label);

        let coverage_label = gtk::Label::new(Some("No summaries generated yet."));
        coverage_label.set_halign(gtk::Align::Start);
        coverage_label.add_css_class("caption");
        progress_box.append(&coverage_label);

        progress_section.set_child(Some(&progress_box));
        content_box.append(&progress_section);

        // 4. In Progress / Recent Courses Section
        let courses_title = gtk::Label::new(Some("Resume Studying"));
        courses_title.add_css_class("heading");
        courses_title.set_halign(gtk::Align::Start);
        content_box.append(&courses_title);

        let courses_container = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content_box.append(&courses_container);

        widget.append(&scroll);

        let nav_pages = Rc::new(RefCell::new(Rc::new(HashMap::new())));

        let s = Self {
            widget,
            state,
            nav,
            nav_pages,
            stat_courses: stat_courses.1,
            stat_modules: stat_modules.1,
            stat_videos: stat_videos.1,
            stat_completed: stat_completed.1,
            progress_bar,
            progress_label,
            coverage_label,
            courses_container,
            content_box,
            status_page,
        };
        s.refresh();
        s
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, NavigationPage>>) {
        *self.nav_pages.borrow_mut() = pages;
    }

    pub fn refresh(&self) {
        // Clear recent courses grid
        while let Some(child) = self.courses_container.first_child() {
            self.courses_container.remove(&child);
        }

        let backend = {
            let state = self.state.borrow();
            state.backend.clone()
        };
        if let Some(ref ctx) = backend {
            match crate::application::ServiceFactory::dashboard(ctx).execute() {
                Ok(analytics) => {
                    self.status_page.set_visible(false);
                    self.content_box.set_visible(true);

                    // Update stats
                    self.stat_courses.set_text(&analytics.total_courses().to_string());
                    self.stat_modules.set_text(&analytics.total_modules().to_string());
                    self.stat_videos.set_text(&analytics.total_videos().to_string());
                    self.stat_completed
                        .set_text(&format!("{:.0}%", analytics.completion_percent()));

                    let pct = analytics.completion_percent() as f64;
                    self.progress_bar.set_value(pct);
                    self.progress_label.set_text(&format!(
                        "{} of {} lessons completed ({:.0}%)",
                        analytics.completed_videos(),
                        analytics.total_videos(),
                        pct
                    ));

                    let cov = analytics.summary_coverage_percent();
                    self.coverage_label.set_text(&format!(
                        "AI Summaries active for {:.0}% of video library ({})",
                        cov,
                        if cov > 50.0 {
                            "Excellent coverage!"
                        } else if cov > 0.0 {
                            "Summarizing courses actively."
                        } else {
                            "No summaries generated yet."
                        }
                    ));

                    // Load list of all courses dynamically to build completion progress cards
                    if let Ok(courses) = ctx.course_repo.find_all() {
                        if courses.is_empty() {
                            let empty_label = gtk::Label::new(Some(
                                "No courses in your library yet. Import a YouTube playlist to get started!",
                            ));
                            empty_label.add_css_class("subtitle");
                            empty_label.set_margin_top(16);
                            empty_label.set_halign(gtk::Align::Start);
                            self.courses_container.append(&empty_label);
                        } else {
                            for course in courses {
                                let c_id = course.id();
                                // Get video statistics for this course
                                let (total_vids, completed_vids) =
                                    match ctx.video_repo.find_by_course(c_id) {
                                        Ok(videos) => {
                                            let tot = videos.len();
                                            let comp =
                                                videos.iter().filter(|v| v.is_completed()).count();
                                            (tot, comp)
                                        },
                                        Err(_) => (0, 0),
                                    };

                                let course_card = gtk::Frame::new(None);
                                course_card.add_css_class("course-progress-card");

                                let card_box = gtk::Box::new(gtk::Orientation::Horizontal, 16);
                                card_box.set_margin_start(16);
                                card_box.set_margin_end(16);
                                card_box.set_margin_top(16);
                                card_box.set_margin_bottom(16);

                                let text_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
                                text_box.set_hexpand(true);

                                let c_title = gtk::Label::new(Some(course.name()));
                                c_title.add_css_class("title");
                                c_title.set_halign(gtk::Align::Start);
                                c_title.set_wrap(true);
                                text_box.append(&c_title);

                                let c_progress_lbl = gtk::Label::new(Some(&format!(
                                    "{} of {} videos completed ({:.0}%)",
                                    completed_vids,
                                    total_vids,
                                    if total_vids > 0 {
                                        (completed_vids as f64 / total_vids as f64) * 100.0
                                    } else {
                                        0.0
                                    }
                                )));
                                c_progress_lbl.add_css_class("subtitle");
                                c_progress_lbl.set_halign(gtk::Align::Start);
                                text_box.append(&c_progress_lbl);

                                let c_level = gtk::LevelBar::new();
                                c_level.set_min_value(0.0);
                                c_level.set_max_value(100.0);
                                c_level.set_value(if total_vids > 0 {
                                    (completed_vids as f64 / total_vids as f64) * 100.0
                                } else {
                                    0.0
                                });
                                text_box.append(&c_level);

                                card_box.append(&text_box);

                                let resume_btn = gtk::Button::with_label("Resume Study");
                                resume_btn.add_css_class("suggested-action");
                                resume_btn.set_valign(gtk::Align::Center);

                                let state_cl = self.state.clone();
                                let nav_cl = self.nav.clone();
                                let nav_pages_cl = self.nav_pages.clone();
                                let course_id_str = c_id.to_string();

                                resume_btn.connect_clicked(move |_| {
                                    state_cl.borrow_mut().current_course_id =
                                        Some(course_id_str.parse().unwrap());
                                    let pages = nav_pages_cl.borrow();
                                    if let Some(page) = pages.get(PAGE_COURSE_VIEW) {
                                        nav_cl.push(page);
                                    }
                                });

                                card_box.append(&resume_btn);
                                course_card.set_child(Some(&card_box));
                                self.courses_container.append(&course_card);
                            }
                        }
                    }
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
            self.status_page.set_title("Welcome to Course Pilot");
            self.status_page
                .set_description(Some("No backend connected. Start by creating or importing a course playlist in the upper right."));
            self.status_page.set_visible(true);
            self.content_box.set_visible(false);
        }
    }
}

fn make_stat_card(title: &str, value: &str) -> (gtk::Frame, gtk::Label) {
    let frame = gtk::Frame::new(None);
    frame.add_css_class("stat-card");

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
    vbox.set_margin_start(4);
    vbox.set_margin_end(4);
    vbox.set_margin_top(4);
    vbox.set_margin_bottom(4);
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
