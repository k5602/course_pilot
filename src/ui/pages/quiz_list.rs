use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::prelude::*;
use adw::{NavigationPage, NavigationView};
use gio::prelude::ListModelExt;

use crate::domain::ports::ExamRepository;
use crate::ui::list_models::QuizObject;
use crate::ui::navigation::PAGE_QUIZ_VIEW;
use crate::ui::state::SharedState;

fn bind_quiz_widgets(list_item: &gtk::ListItem) -> Option<(gtk::Label, gtk::Label, gtk::Button)> {
    let frame = list_item.child()?.downcast::<gtk::Frame>().ok()?;
    let hbox = frame.child()?.downcast::<gtk::Box>().ok()?;
    let info_box = hbox.first_child()?.downcast::<gtk::Box>().ok()?;
    let title = info_box.first_child()?.downcast::<gtk::Label>().ok()?;
    let status_label = title.next_sibling()?.downcast::<gtk::Label>().ok()?;
    let start_btn = info_box.next_sibling()?.downcast::<gtk::Button>().ok()?;
    Some((title, status_label, start_btn))
}

pub struct QuizListPage {
    widget: gtk::Box,
    state: SharedState,
    nav_pages: Rc<RefCell<Rc<HashMap<&'static str, NavigationPage>>>>,
    store: gio::ListStore,
    status_page: adw::StatusPage,
    list_view: gtk::ListView,
}

impl QuizListPage {
    pub fn new(state: SharedState, nav: Rc<NavigationView>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");

        let heading = gtk::Label::new(Some("Quizzes"));
        heading.add_css_class("heading");
        widget.append(&heading);

        let subtitle = gtk::Label::new(Some("Test your knowledge."));
        subtitle.add_css_class("subtitle");
        widget.append(&subtitle);

        let status_page = adw::StatusPage::new();
        status_page.set_title("No Quizzes Yet");
        status_page.set_description(Some("Generate one from a video summary."));
        status_page.set_icon_name(Some("applications-science-symbolic"));
        status_page.set_margin_top(16);
        widget.append(&status_page);

        let store = gio::ListStore::new::<QuizObject>();
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

            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            hbox.set_margin_start(12);
            hbox.set_margin_end(12);
            hbox.set_margin_top(12);
            hbox.set_margin_bottom(12);

            let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            info_box.set_hexpand(true);

            let title = gtk::Label::new(None);
            title.set_halign(gtk::Align::Start);
            title.add_css_class("heading");
            info_box.append(&title);

            let status_label = gtk::Label::new(None);
            status_label.set_halign(gtk::Align::Start);
            status_label.add_css_class("subtitle");
            info_box.append(&status_label);

            hbox.append(&info_box);

            let start_btn = gtk::Button::with_label("Start");
            start_btn.set_valign(gtk::Align::Center);
            hbox.append(&start_btn);

            frame.set_child(Some(&hbox));
            list_item.set_child(Some(&frame));
        });

        factory.connect_bind(move |_factory, list_item| {
            let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
                log::warn!("connect_bind: failed to downcast to gtk::ListItem");
                return;
            };

            let Some((title, status_label, start_btn)) = bind_quiz_widgets(list_item) else {
                log::warn!("connect_bind: failed to downcast quiz widget tree");
                return;
            };

            let item = list_item.item();
            if let Some(quiz) = item.as_ref().and_then(|i| i.downcast_ref::<QuizObject>()) {
                title.set_text(&quiz.title());

                if quiz.is_taken() {
                    let score_text = format!(
                        "Score: {:.0}% {}",
                        quiz.score().unwrap_or(0.0) * 100.0,
                        if quiz.passed().unwrap_or(false) { "[PASS]" } else { "[FAIL]" }
                    );
                    status_label.set_text(&score_text);
                    start_btn.set_label("Review");
                } else {
                    status_label.set_text("Not taken yet");
                    start_btn.set_label("Start");
                }
            }
        });

        let list_view = gtk::ListView::new(Some(no_selection), Some(factory));
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
            if let Some(quiz) = item.as_ref().and_then(|i| i.downcast_ref::<QuizObject>()) {
                state_for_activate.borrow_mut().current_quiz_id = Some(quiz.id());
                let pages = pages_for_activate.borrow();
                if let Some(page) = pages.get(PAGE_QUIZ_VIEW) {
                    nav_for_activate.push(page);
                }
            }
        });

        widget.append(&list_view);

        Self { widget, state, nav_pages: nav_pages_rc, store, status_page, list_view }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, NavigationPage>>) {
        *self.nav_pages.borrow_mut() = pages;
    }

    pub fn refresh(&self) {
        self.store.remove_all();
        self.list_view.set_visible(false);
        self.status_page.set_visible(false);

        let state = self.state.borrow();
        if let Some(ref ctx) = state.backend {
            match ctx.exam_repo.find_all() {
                Ok(exams) => {
                    if exams.is_empty() {
                        self.status_page.set_visible(true);
                    } else {
                        for exam in &exams {
                            let obj = QuizObject::new(
                                exam.id().to_string(),
                                format!("Exam for video {}", exam.video_id()),
                                exam.video_id().to_string(),
                                exam.is_taken(),
                                exam.score(),
                                exam.passed(),
                            );
                            self.store.append(&obj);
                        }
                        self.list_view.set_visible(true);
                    }
                },
                Err(e) => {
                    self.status_page.set_title("Error Loading Quizzes");
                    self.status_page
                        .set_description(Some(&format!("Failed to load quizzes: {}", e)));
                    self.status_page.set_visible(true);
                },
            }
        } else {
            self.status_page.set_title("No Backend");
            self.status_page.set_description(Some("No backend connected."));
            self.status_page.set_visible(true);
        }
    }
}
