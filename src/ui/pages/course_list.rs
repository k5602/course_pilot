use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use adw::{NavigationPage, NavigationView};
use gio::prelude::ListModelExt;

use crate::domain::ports::{CourseRepository, ModuleRepository};
use crate::ui::list_models::CourseObject;
use crate::ui::navigation::PAGE_COURSE_VIEW;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;

fn bind_course_widgets(list_item: &gtk::ListItem) -> Option<(gtk::Label, gtk::Label, gtk::Label)> {
    let frame = list_item.child()?.downcast::<gtk::Frame>().ok()?;
    let vbox = frame.child()?.downcast::<gtk::Box>().ok()?;
    let title = vbox.first_child()?.downcast::<gtk::Label>().ok()?;
    let desc_label = title.next_sibling()?.downcast::<gtk::Label>().ok()?;
    let info_label = desc_label.next_sibling()?.downcast::<gtk::Label>().ok()?;
    Some((title, desc_label, info_label))
}

pub struct CourseListPage {
    widget: gtk::Box,
    state: SharedState,
    nav_pages: Rc<RefCell<Rc<HashMap<&'static str, NavigationPage>>>>,
    store: gio::ListStore,
    search_entry: gtk::SearchEntry,
    status_page: adw::StatusPage,
    list_view: gtk::ListView,
}

impl CourseListPage {
    pub fn new(state: SharedState, nav: Rc<NavigationView>) -> Self {
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

        let store = gio::ListStore::new::<CourseObject>();
        let no_selection = gtk::NoSelection::new(Some(store.clone()));

        let factory = gtk::SignalListItemFactory::new();

        factory.connect_setup(|_factory, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
                log::warn!("connect_setup: failed to downcast to gtk::ListItem");
                return;
            };

            let frame = gtk::Frame::new(None);
            frame.add_css_class("card");
            frame.set_hexpand(true);

            let vbox = gtk::Box::new(gtk::Orientation::Vertical, 4);
            vbox.set_margin_start(12);
            vbox.set_margin_end(12);
            vbox.set_margin_top(12);
            vbox.set_margin_bottom(12);

            let title = gtk::Label::new(None);
            title.set_halign(gtk::Align::Start);
            title.add_css_class("heading");
            vbox.append(&title);

            let desc_label = gtk::Label::new(None);
            desc_label.set_halign(gtk::Align::Start);
            desc_label.set_wrap(true);
            desc_label.add_css_class("subtitle");
            vbox.append(&desc_label);

            let info_label = gtk::Label::new(None);
            info_label.set_halign(gtk::Align::Start);
            info_label.add_css_class("caption");
            vbox.append(&info_label);

            frame.set_child(Some(&vbox));
            list_item.set_child(Some(&frame));
        });

        factory.connect_bind(move |_factory, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
                log::warn!("connect_bind: failed to downcast to gtk::ListItem");
                return;
            };

            let Some((title, desc_label, info_label)) = bind_course_widgets(list_item) else {
                log::warn!("connect_bind: failed to downcast course widget tree");
                return;
            };

            let item = list_item.item();
            if let Some(course) = item.as_ref().and_then(|i| i.downcast_ref::<CourseObject>()) {
                let course_title = course.title();
                title.set_text(&course_title);
                match course.description() {
                    Some(ref d) => {
                        desc_label.set_text(d);
                        desc_label.set_visible(true);
                    },
                    None => desc_label.set_visible(false),
                }
                info_label.set_text(&format!("{} modules", course.module_count()));
            }
        });

        let list_view = gtk::ListView::new(Some(no_selection.clone()), Some(factory.clone()));
        list_view.set_single_click_activate(true);
        list_view.set_margin_start(16);
        list_view.set_margin_end(16);
        list_view.add_css_class("boxed-list");
        list_view.set_vexpand(true);
        list_view.set_hexpand(true);

        let nav_pages_rc: Rc<RefCell<Rc<HashMap<&'static str, NavigationPage>>>> =
            Rc::new(RefCell::new(Rc::new(HashMap::new())));
        let store_for_activate = store.clone();
        let nav_for_activate = nav.clone();
        let state_for_activate = state.clone();
        let pages_for_activate = nav_pages_rc.clone();
        list_view.connect_activate(move |_, pos| {
            let item = store_for_activate.item(pos);
            if let Some(course) = item.as_ref().and_then(|i| i.downcast_ref::<CourseObject>()) {
                state_for_activate.borrow_mut().current_course_id = Some(course.id());
                let pages = pages_for_activate.borrow();
                if let Some(page) = pages.get(PAGE_COURSE_VIEW) {
                    nav_for_activate.push(page);
                }
            }
        });

        widget.append(&list_view);

        let page = Self {
            widget,
            state: state.clone(),
            nav_pages: nav_pages_rc,
            store: store.clone(),
            search_entry: search_entry.clone(),
            status_page,
            list_view,
        };

        let state_cl = state.clone();
        let search_cl = search_entry.clone();
        let store_cl = page.store.clone();
        let status_cl = page.status_page.clone();
        let lv_clone = page.list_view.clone();
        search_entry.connect_search_changed(move |_| {
            Self::repopulate(&state_cl, &search_cl, &store_cl, &status_cl, &lv_clone);
        });

        page
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, NavigationPage>>) {
        *self.nav_pages.borrow_mut() = pages;
    }

    pub fn refresh(&self) {
        Self::repopulate(
            &self.state,
            &self.search_entry,
            &self.store,
            &self.status_page,
            &self.list_view,
        );
    }

    fn repopulate(
        state: &SharedState,
        search: &gtk::SearchEntry,
        store: &gio::ListStore,
        status_page: &adw::StatusPage,
        list_view: &gtk::ListView,
    ) {
        store.remove_all();
        list_view.set_visible(false);
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
                                let mc = ctx
                                    .module_repo
                                    .find_by_course(course.id())
                                    .ok()
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                                let obj = CourseObject::new(
                                    course.id().to_string(),
                                    course.name().to_string(),
                                    course.description().map(|s| s.to_string()),
                                    mc as i32,
                                );
                                store.append(&obj);
                            }
                            list_view.set_visible(true);
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
}
