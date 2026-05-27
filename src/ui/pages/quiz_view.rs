use std::cell::RefCell;
use std::rc::Rc;

use adw::NavigationView;
use adw::prelude::*;

use crate::domain::entities::QuizQuestion;
use crate::domain::ports::ExamRepository;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;

struct QuizState {
    questions: Vec<QuizQuestion>,
    answers: Vec<Option<usize>>,
    current_index: usize,
}

pub struct QuizViewPage {
    widget: gtk::Box,
    state: SharedState,
    nav: Rc<NavigationView>,
    content_box: gtk::Box,
    status_page: adw::StatusPage,
    quiz_state: Rc<RefCell<Option<QuizState>>>,
}

impl QuizViewPage {
    pub fn new(state: SharedState, nav: Rc<NavigationView>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");

        let heading = gtk::Label::new(Some("Quiz"));
        heading.add_css_class("heading");
        widget.append(&heading);

        let status_page = adw::StatusPage::new();
        status_page.set_title("Loading...");
        status_page.set_description(Some("Loading quiz..."));
        widget.append(&status_page);

        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let content_box = gtk::Box::new(gtk::Orientation::Vertical, 12);
        content_box.set_margin_start(16);
        content_box.set_margin_end(16);
        content_box.set_margin_bottom(16);
        scroll.set_child(Some(&content_box));

        widget.append(&scroll);

        Self {
            widget,
            state,
            nav,
            content_box,
            status_page,
            quiz_state: Rc::new(RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn refresh(&self) {
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }
        self.content_box.set_visible(false);
        self.status_page.set_visible(true);

        let state = self.state.borrow();
        let quiz_id_str = match state.current_quiz_id {
            Some(ref id) => id.clone(),
            None => {
                self.status_page.set_title("No Selection");
                self.status_page.set_description(Some("No quiz selected."));
                self.status_page.set_visible(true);
                return;
            },
        };

        if let Some(ref ctx) = state.backend {
            let exam_id = match quiz_id_str.parse::<crate::domain::value_objects::ExamId>() {
                Ok(id) => id,
                Err(_) => {
                    self.status_page.set_title("Invalid ID");
                    self.status_page.set_description(Some("Invalid quiz ID."));
                    self.status_page.set_visible(true);
                    return;
                },
            };

            match ctx.exam_repo.find_by_id(&exam_id) {
                Ok(Some(exam)) => {
                    self.status_page.set_visible(false);
                    self.content_box.set_visible(true);

                    match serde_json::from_str::<Vec<QuizQuestion>>(exam.question_json()) {
                        Ok(questions) => {
                            let n = questions.len();
                            *self.quiz_state.borrow_mut() = Some(QuizState {
                                questions,
                                answers: vec![None; n],
                                current_index: 0,
                            });

                            if exam.is_taken() {
                                let qs = self.quiz_state.borrow();
                                if let Some(ref qs) = *qs {
                                    Self::show_results(
                                        &self.content_box,
                                        &self.nav,
                                        qs,
                                        exam.score(),
                                        exam.passed(),
                                        exam.user_answers_json(),
                                    );
                                }
                            } else {
                                self.show_current_question();
                            }
                        },
                        Err(e) => {
                            let err_label =
                                gtk::Label::new(Some(&format!("Failed to parse quiz: {}", e)));
                            err_label.add_css_class("subtitle");
                            self.content_box.append(&err_label);
                        },
                    }
                },
                Ok(None) => {
                    self.status_page.set_title("Not Found");
                    self.status_page.set_description(Some("Quiz not found."));
                    self.status_page.set_visible(true);
                },
                Err(e) => {
                    Toast::show_error(&format!("Error loading quiz: {}", e));
                    self.status_page.set_title("Error");
                    self.status_page.set_description(Some(&format!("Error loading quiz: {}", e)));
                    self.status_page.set_visible(true);
                },
            }
        } else {
            self.status_page.set_title("No Backend");
            self.status_page.set_description(Some("No backend connected."));
            self.status_page.set_visible(true);
        }
    }

    fn show_current_question(&self) {
        show_question_inner(&self.quiz_state, &self.content_box, &self.state, &self.nav);
    }

    fn show_results(
        content_box: &gtk::Box,
        nav: &Rc<NavigationView>,
        qs: &QuizState,
        score: Option<f32>,
        passed: Option<bool>,
        user_answers_json: Option<&str>,
    ) {
        while let Some(child) = content_box.first_child() {
            content_box.remove(&child);
        }

        let score_val = score.unwrap_or(0.0);
        let passed_val = passed.unwrap_or(false);
        let total = qs.questions.len();
        let correct_count = (score_val * total as f32).round() as usize;

        // Score summary card
        let score_frame = gtk::Frame::new(None);
        score_frame.add_css_class("card");
        score_frame.set_margin_bottom(24);

        let score_box = gtk::Box::new(gtk::Orientation::Vertical, 8);
        score_box.set_margin_start(16);
        score_box.set_margin_end(16);
        score_box.set_margin_top(16);
        score_box.set_margin_bottom(16);
        score_box.set_halign(gtk::Align::Center);

        let status_icon = gtk::Image::from_icon_name(if passed_val {
            "emblem-ok-symbolic"
        } else {
            "window-close-symbolic"
        });
        status_icon.set_pixel_size(64);
        status_icon.add_css_class(if passed_val { "success" } else { "error" });
        score_box.append(&status_icon);

        let status_label = gtk::Label::new(None);
        status_label.set_markup(&format!(
            "<big><b>Quiz {}</b></big>",
            if passed_val { "Passed" } else { "Failed" }
        ));
        status_label.add_css_class(if passed_val { "success" } else { "error" });
        status_label.add_css_class("heading");
        score_box.append(&status_label);

        let percent_label = gtk::Label::new(None);
        percent_label.set_markup(&format!(
            "Score: <b>{:.0}%</b> ({} of {} questions correct)",
            score_val * 100.0,
            correct_count,
            total
        ));
        percent_label.add_css_class("title");
        score_box.append(&percent_label);

        score_frame.set_child(Some(&score_box));
        content_box.append(&score_frame);

        let parsed_answers: Vec<Option<usize>> = user_answers_json
            .and_then(|j| serde_json::from_str::<Vec<Option<usize>>>(j).ok())
            .unwrap_or_default();

        // Question review cards
        for (i, q) in qs.questions.iter().enumerate() {
            let card_frame = gtk::Frame::new(None);
            card_frame.add_css_class("course-progress-card");
            card_frame.set_margin_bottom(16);

            let q_box = gtk::Box::new(gtk::Orientation::Vertical, 10);
            q_box.set_margin_start(16);
            q_box.set_margin_end(16);
            q_box.set_margin_top(16);
            q_box.set_margin_bottom(16);

            // Question title
            let q_label = gtk::Label::new(Some(&format!("{}. {}", i + 1, q.question)));
            q_label.set_wrap(true);
            q_label.set_halign(gtk::Align::Start);
            q_label.add_css_class("title");
            q_box.append(&q_label);

            let user_ans = parsed_answers.get(i).copied().flatten();

            // Options List
            let list_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
            for (oi, opt) in q.options.iter().enumerate() {
                let option_row = gtk::Box::new(gtk::Orientation::Horizontal, 12);
                option_row.add_css_class("quiz-option");

                // Highlight correct/incorrect answers
                if oi == q.correct_index {
                    option_row.add_css_class("quiz-option-correct");
                } else if Some(oi) == user_ans {
                    option_row.add_css_class("quiz-option-incorrect");
                }

                let badge = gtk::Label::new(Some(&format!("{}.", (b'A' + oi as u8) as char)));
                badge.add_css_class("heading");
                badge.set_valign(gtk::Align::Center);
                option_row.append(&badge);

                let opt_text = gtk::Label::new(Some(opt));
                opt_text.set_wrap(true);
                opt_text.set_halign(gtk::Align::Start);
                opt_text.set_valign(gtk::Align::Center);
                option_row.append(&opt_text);

                // Add indicator badges for student reference
                if oi == q.correct_index {
                    let correct_tag = gtk::Label::new(Some("Correct Answer"));
                    correct_tag.add_css_class("caption");
                    correct_tag.add_css_class("success");
                    correct_tag.set_halign(gtk::Align::End);
                    correct_tag.set_hexpand(true);
                    option_row.append(&correct_tag);
                } else if Some(oi) == user_ans {
                    let incorrect_tag = gtk::Label::new(Some("Your Choice"));
                    incorrect_tag.add_css_class("caption");
                    incorrect_tag.add_css_class("error");
                    incorrect_tag.set_halign(gtk::Align::End);
                    incorrect_tag.set_hexpand(true);
                    option_row.append(&incorrect_tag);
                }

                list_box.append(&option_row);
            }
            q_box.append(&list_box);

            // Explanation box
            let expl_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
            expl_box.add_css_class("explanation-box");

            let expl_title = gtk::Label::new(Some("Explanation"));
            expl_title.add_css_class("title");
            expl_title.set_halign(gtk::Align::Start);
            expl_box.append(&expl_title);

            let expl_text = gtk::Label::new(Some(&q.explanation));
            expl_text.set_wrap(true);
            expl_text.set_halign(gtk::Align::Start);
            expl_text.add_css_class("subtitle");
            expl_box.append(&expl_text);

            q_box.append(&expl_box);

            card_frame.set_child(Some(&q_box));
            content_box.append(&card_frame);
        }

        let back_btn = gtk::Button::with_label("Back to Quizzes");
        back_btn.set_halign(gtk::Align::Center);
        back_btn.add_css_class("suggested-action");
        back_btn.set_margin_top(16);
        back_btn.set_margin_bottom(16);
        let nav_cl = nav.clone();
        back_btn.connect_clicked(move |_| {
            nav_cl.pop();
        });
        content_box.append(&back_btn);
    }
}

fn show_question_inner(
    quiz_state: &Rc<RefCell<Option<QuizState>>>,
    content_box: &gtk::Box,
    state: &SharedState,
    nav: &Rc<NavigationView>,
) {
    while let Some(child) = content_box.first_child() {
        content_box.remove(&child);
    }

    let qs_borrow = quiz_state.borrow();
    let qs = match *qs_borrow {
        Some(ref qs) => qs,
        None => return,
    };

    if qs.current_index >= qs.questions.len() {
        drop(qs_borrow);
        submit_quiz_inner(quiz_state, state, nav, content_box);
        return;
    }

    let q = &qs.questions[qs.current_index];
    let idx = qs.current_index;
    let total = qs.questions.len();
    let saved_answer = qs.answers[idx];

    // Progress Bar Indicator
    let progress_sec = gtk::Box::new(gtk::Orientation::Vertical, 6);
    progress_sec.set_margin_bottom(12);

    let counter = gtk::Label::new(Some(&format!("Question {} of {}", idx + 1, total)));
    counter.add_css_class("subtitle");
    counter.set_halign(gtk::Align::Start);
    progress_sec.append(&counter);

    let progress_bar = gtk::ProgressBar::new();
    progress_bar.set_fraction((idx as f64) / (total as f64));
    progress_sec.append(&progress_bar);

    content_box.append(&progress_sec);

    // Question
    let q_label = gtk::Label::new(Some(&q.question));
    q_label.set_wrap(true);
    q_label.set_halign(gtk::Align::Start);
    q_label.add_css_class("heading");
    q_label.set_margin_bottom(16);
    content_box.append(&q_label);

    // ListBox for Option Cards
    let list_box = gtk::ListBox::new();
    list_box.set_selection_mode(gtk::SelectionMode::Single);
    list_box.add_css_class("boxed-list");
    list_box.set_margin_bottom(24);

    for (oi, opt) in q.options.iter().enumerate() {
        let row = gtk::ListBoxRow::new();
        row.add_css_class("quiz-option");

        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);
        row_box.set_margin_top(4);
        row_box.set_margin_bottom(4);

        let badge = gtk::Label::new(Some(&format!("{}.", (b'A' + oi as u8) as char)));
        badge.add_css_class("heading");
        badge.set_valign(gtk::Align::Center);
        row_box.append(&badge);

        let opt_text = gtk::Label::new(Some(opt));
        opt_text.set_wrap(true);
        opt_text.set_halign(gtk::Align::Start);
        opt_text.set_valign(gtk::Align::Center);
        row_box.append(&opt_text);

        row.set_child(Some(&row_box));
        list_box.append(&row);

        // Pre-select row if already saved
        if saved_answer == Some(oi) {
            let lb = list_box.clone();
            let r = row.clone();
            glib::idle_add_local_once(move || {
                lb.select_row(Some(&r));
            });
        }
    }

    content_box.append(&list_box);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);

    let is_last = idx + 1 >= total;
    let next_btn = gtk::Button::with_label(if is_last { "Submit" } else { "Next" });
    next_btn.add_css_class("suggested-action");

    let qs = quiz_state.clone();
    let list_box_clone = list_box;
    let content_box_clone = content_box.clone();
    let state_clone = state.clone();
    let nav_clone = nav.clone();

    next_btn.connect_clicked(move |_| {
        let selected_row = list_box_clone.selected_row();
        let selected_idx = selected_row.map(|r| r.index() as usize);

        // Ensure student has selected an option
        if selected_idx.is_none() {
            Toast::show("Please select an option before continuing.");
            return;
        }

        let mut qs_borrow = qs.borrow_mut();
        let qs_ref = match *qs_borrow {
            Some(ref mut qs) => qs,
            None => return,
        };

        qs_ref.answers[qs_ref.current_index] = selected_idx;

        if is_last {
            drop(qs_borrow);
            submit_quiz_inner(&qs, &state_clone, &nav_clone, &content_box_clone);
        } else {
            qs_ref.current_index += 1;
            drop(qs_borrow);
            show_question_inner(&qs, &content_box_clone, &state_clone, &nav_clone);
        }
    });

    btn_box.append(&next_btn);
    content_box.append(&btn_box);
}

fn submit_quiz_inner(
    quiz_state: &Rc<RefCell<Option<QuizState>>>,
    state: &SharedState,
    nav: &Rc<NavigationView>,
    content_box: &gtk::Box,
) {
    while let Some(child) = content_box.first_child() {
        content_box.remove(&child);
    }

    let mut qs_borrow = quiz_state.borrow_mut();
    let qs = match *qs_borrow {
        Some(ref mut qs) => qs,
        None => return,
    };

    let correct = qs
        .questions
        .iter()
        .zip(qs.answers.iter())
        .filter(|(q, a)| a.map(|a| a == q.correct_index).unwrap_or(false))
        .count();

    let total = qs.questions.len();
    let score = if total > 0 { correct as f32 / total as f32 } else { 0.0 };
    let passed = score >= crate::domain::entities::PASS_THRESHOLD;

    // Save answers snapshot before we write to database and reload
    let answers_snapshot = qs.answers.clone();

    let s = state.borrow();
    if let Some(ref ctx) = s.backend {
        let quiz_id_str = s.current_quiz_id.as_ref().cloned().unwrap_or_default();
        if let Ok(exam_id) = quiz_id_str.parse::<crate::domain::value_objects::ExamId>() {
            let answers_json = serde_json::to_string(&answers_snapshot).ok();
            if let Err(e) = ctx.exam_repo.update_result(&exam_id, score, passed, answers_json) {
                Toast::show_error(&format!("Failed to save quiz result: {}", e));
            }
        }
    }

    // Direct transition to results display for instant visual gratification
    QuizViewPage::show_results(
        content_box,
        nav,
        qs,
        Some(score),
        Some(passed),
        serde_json::to_string(&answers_snapshot).ok().as_deref(),
    );
}
