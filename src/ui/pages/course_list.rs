use std::rc::Rc;

use adw::prelude::*;

use crate::domain::ports::{CourseRepository, ModuleRepository};
use crate::ui::navigation::PAGE_COURSE_VIEW;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;

pub struct CourseListPage {
    widget: gtk::Box,
    state: SharedState,
    stack: Rc<gtk::Stack>,
    search_entry: gtk::SearchEntry,
    list_box: gtk::Box,
    status_page: adw::StatusPage,
}

impl CourseListPage {
    pub fn new(state: SharedState, stack: Rc<gtk::Stack>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");

        let heading = gtk::Label::new(Some("Courses"));
        heading.add_css_class("heading");
        widget.append(&heading);

        let subtitle = gtk::Label::new(Some("Your course library."));
        subtitle.add_css_class("subtitle");
        widget.append(&subtitle);

        let search_entry = gtk::SearchEntry::new();
        search_entry.set_placeholder_text(Some("Filter courses..."));
        search_entry.set_margin_start(16);
        search_entry.set_margin_end(16);
        widget.append(&search_entry);

        let status_page = adw::StatusPage::new();
        status_page.set_title("No Courses Yet");
        status_page
            .set_description(Some("Import a YouTube playlist or local media to get started."));
        status_page.set_icon_name(Some("folder-videos-symbolic"));
        status_page.set_margin_top(16);
        widget.append(&status_page);

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let list_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        list_box.set_margin_start(16);
        list_box.set_margin_end(16);
        list_box.set_margin_bottom(16);
        scroll.set_child(Some(&list_box));

        widget.append(&scroll);

        let page = Self {
            widget,
            state: state.clone(),
            stack,
            search_entry: search_entry.clone(),
            list_box: list_box.clone(),
            status_page,
        };

        let state_cl = state.clone();
        let search_cl = search_entry.clone();
        let list_cl = list_box.clone();
        let status_cl = page.status_page.clone();
        let stack_cl = page.stack.clone();

        search_entry.connect_search_changed(move |_| {
            Self::repopulate(&state_cl, &search_cl, &list_cl, &status_cl, &stack_cl);
        });

        page
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn refresh(&self) {
        Self::repopulate(
            &self.state,
            &self.search_entry,
            &self.list_box,
            &self.status_page,
            &self.stack,
        );
    }

    fn repopulate(
        state: &SharedState,
        search: &gtk::SearchEntry,
        list_box: &gtk::Box,
        status_page: &adw::StatusPage,
        stack: &Rc<gtk::Stack>,
    ) {
        while let Some(child) = list_box.first_child() {
            list_box.remove(&child);
        }
        list_box.set_visible(false);
        status_page.set_visible(false);

        let s = state.borrow();
        if let Some(ref ctx) = s.backend {
            match ctx.course_repo.find_all() {
                Ok(courses) => {
                    if courses.is_empty() {
                        status_page.set_title("No Courses Yet");
                        status_page.set_description(Some(
                            "Import a YouTube playlist or local media to get started.",
                        ));
                        status_page.set_visible(true);
                    } else {
                        let filter = search.text().as_str().to_lowercase();
                        let filtered: Vec<_> = courses
                            .iter()
                            .filter(|c| {
                                filter.is_empty() || c.name().to_lowercase().contains(&filter)
                            })
                            .collect();

                        if filtered.is_empty() && !filter.is_empty() {
                            status_page.set_title("No Results");
                            status_page.set_description(Some("No courses match your filter."));
                            status_page.set_visible(true);
                        } else {
                            for course in &filtered {
                                let card = Self::build_course_card(course, state, stack);
                                list_box.append(&card);
                            }
                            list_box.set_visible(true);
                        }
                    }
                },
                Err(e) => {
                    Toast::show_error(&format!("Failed to load courses: {}", e));
                    status_page.set_title("Error Loading Courses");
                    status_page.set_description(Some(&format!("Failed to load courses: {}", e)));
                    status_page.set_visible(true);
                },
            }
        } else {
            status_page.set_title("No Backend");
            status_page.set_description(Some("No backend connected."));
            status_page.set_visible(true);
        }
    }

    fn build_course_card(
        course: &crate::domain::entities::Course,
        state: &SharedState,
        stack: &Rc<gtk::Stack>,
    ) -> gtk::Frame {
        let frame = gtk::Frame::new(None);
        frame.add_css_class("card");
        frame.set_hexpand(true);

        let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);

        let title = gtk::Label::new(Some(course.name()));
        title.set_halign(gtk::Align::Start);
        title.add_css_class("heading");
        vbox.append(&title);

        if let Some(desc) = course.description() {
            let desc_label = gtk::Label::new(Some(desc));
            desc_label.set_halign(gtk::Align::Start);
            desc_label.set_wrap(true);
            desc_label.add_css_class("subtitle");
            vbox.append(&desc_label);
        }

        let module_count = state
            .borrow()
            .backend
            .as_ref()
            .and_then(|ctx| ctx.module_repo.find_by_course(course.id()).ok().map(|m| m.len()))
            .unwrap_or(0);

        let info_label = gtk::Label::new(Some(&format!("{} modules", module_count)));
        info_label.set_halign(gtk::Align::Start);
        info_label.add_css_class("caption");
        vbox.append(&info_label);

        let gesture = gtk::GestureClick::new();
        let state_clone = state.clone();
        let stack_clone = stack.clone();
        let course_id = course.id().to_string();
        gesture.connect_pressed(move |_, _, _, _| {
            state_clone.borrow_mut().current_course_id = Some(course_id.clone());
            stack_clone.set_visible_child_name(PAGE_COURSE_VIEW);
        });
        frame.add_controller(gesture);

        frame.set_child(Some(&vbox));
        frame
    }
}
