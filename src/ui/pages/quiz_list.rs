use std::rc::Rc;

use adw::prelude::*;

use crate::domain::ports::ExamRepository;
use crate::ui::navigation::PAGE_QUIZ_VIEW;
use crate::ui::state::SharedState;

pub struct QuizListPage {
    widget: gtk::Box,
    state: SharedState,
    stack: Rc<gtk::Stack>,
    list_box: gtk::Box,
    status_page: adw::StatusPage,
}

impl QuizListPage {
    pub fn new(state: SharedState, stack: Rc<gtk::Stack>) -> Self {
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

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let list_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        list_box.set_margin_start(16);
        list_box.set_margin_end(16);
        scroll.set_child(Some(&list_box));

        widget.append(&scroll);

        Self { widget, state, stack, list_box, status_page }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn refresh(&self) {
        while let Some(child) = self.list_box.first_child() {
            self.list_box.remove(&child);
        }
        self.list_box.set_visible(false);
        self.status_page.set_visible(false);

        let state = self.state.borrow();
        if let Some(ref ctx) = state.backend {
            match ctx.exam_repo.find_all() {
                Ok(exams) => {
                    if exams.is_empty() {
                        self.status_page.set_visible(true);
                    } else {
                        for exam in &exams {
                            let card = self.build_exam_card(exam);
                            self.list_box.append(&card);
                        }
                        self.list_box.set_visible(true);
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

    fn build_exam_card(&self, exam: &crate::domain::entities::Exam) -> gtk::Frame {
        let frame = gtk::Frame::new(None);
        frame.add_css_class("card");
        frame.set_hexpand(true);

        let vbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        vbox.set_margin_start(12);
        vbox.set_margin_end(12);
        vbox.set_margin_top(12);
        vbox.set_margin_bottom(12);

        let info_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        info_box.set_hexpand(true);

        let title = gtk::Label::new(Some(&format!("Exam for video {}", exam.video_id())));
        title.set_halign(gtk::Align::Start);
        title.add_css_class("heading");
        info_box.append(&title);

        if exam.is_taken() {
            let score_text = format!(
                "Score: {:.0}% {}",
                exam.score().unwrap_or(0.0) * 100.0,
                if exam.passed().unwrap_or(false) { "[PASS]" } else { "[FAIL]" }
            );
            let score_label = gtk::Label::new(Some(&score_text));
            score_label.set_halign(gtk::Align::Start);
            score_label.add_css_class("subtitle");
            info_box.append(&score_label);
        } else {
            let pending = gtk::Label::new(Some("Not taken yet"));
            pending.set_halign(gtk::Align::Start);
            pending.add_css_class("subtitle");
            info_box.append(&pending);
        }

        vbox.append(&info_box);

        let start_btn = gtk::Button::with_label(if exam.is_taken() { "Review" } else { "Start" });
        start_btn.set_valign(gtk::Align::Center);

        let state_cl = self.state.clone();
        let stack_cl = self.stack.clone();
        let exam_id = exam.id().to_string();
        start_btn.connect_clicked(move |_| {
            state_cl.borrow_mut().current_quiz_id = Some(exam_id.clone());
            stack_cl.set_visible_child_name(PAGE_QUIZ_VIEW);
        });
        vbox.append(&start_btn);

        frame.set_child(Some(&vbox));
        frame
    }
}
