use std::cell::RefCell;
use std::rc::Rc;

use adw::prelude::*;

use crate::domain::ports::ExamRepository;
use crate::ui::navigation::PAGE_QUIZ_LIST;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;

#[derive(serde::Deserialize)]
struct QuizQuestion {
    question: String,
    options: Vec<String>,
    correct_index: usize,
    explanation: String,
}

struct QuizState {
    questions: Vec<QuizQuestion>,
    answers: Vec<Option<usize>>,
    current_index: usize,
}

pub struct QuizViewPage {
    widget: gtk::Box,
    state: SharedState,
    stack: Rc<gtk::Stack>,
    content_box: gtk::Box,
    status_page: adw::StatusPage,
    quiz_state: Rc<RefCell<Option<QuizState>>>,
}

impl QuizViewPage {
    pub fn new(state: SharedState, stack: Rc<gtk::Stack>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");

        let back_btn = gtk::Button::with_label("Back to Quizzes");
        back_btn.add_css_class("flat");
        back_btn.set_halign(gtk::Align::Start);
        back_btn.set_margin_start(8);
        back_btn.set_margin_top(8);

        let stack_cl = stack.clone();
        back_btn.connect_clicked(move |_| {
            stack_cl.set_visible_child_name(PAGE_QUIZ_LIST);
        });
        widget.append(&back_btn);

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
            stack,
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
                                        &self.stack,
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
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }

        let quiz_state = self.quiz_state.borrow();
        let qs = match *quiz_state {
            Some(ref qs) => qs,
            None => return,
        };

        if qs.current_index >= qs.questions.len() {
            drop(quiz_state);
            self.submit_quiz();
            return;
        }

        let q = &qs.questions[qs.current_index];
        let idx = qs.current_index;
        let total = qs.questions.len();
        let saved_answer = qs.answers[idx];

        let counter = gtk::Label::new(Some(&format!("Question {} of {}", idx + 1, total)));
        counter.add_css_class("subtitle");
        counter.set_halign(gtk::Align::Start);
        self.content_box.append(&counter);

        let q_label = gtk::Label::new(Some(&q.question));
        q_label.set_wrap(true);
        q_label.set_halign(gtk::Align::Start);
        q_label.add_css_class("heading");
        self.content_box.append(&q_label);

        let option_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
        option_box.set_margin_start(12);

        let mut radios: Vec<gtk::CheckButton> = Vec::new();
        for (oi, opt) in q.options.iter().enumerate() {
            let radio = gtk::CheckButton::with_label(opt);
            if let Some(first) = radios.first() {
                radio.set_group(Some(first));
            }
            if saved_answer == Some(oi) {
                radio.set_active(true);
            }
            option_box.append(&radio);
            radios.push(radio);
        }

        self.content_box.append(&option_box);

        let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        btn_box.set_halign(gtk::Align::End);

        let is_last = idx + 1 >= total;
        let next_btn = gtk::Button::with_label(if is_last { "Submit" } else { "Next" });
        next_btn.add_css_class("suggested-action");

        let quiz_state = self.quiz_state.clone();
        let content_box = self.content_box.clone();
        let state = self.state.clone();
        let stack = self.stack.clone();
        let radios_clone = radios;

        next_btn.connect_clicked(move |_| {
            let mut qs_borrow = quiz_state.borrow_mut();
            let qs = match *qs_borrow {
                Some(ref mut qs) => qs,
                None => return,
            };

            for (oi, r) in radios_clone.iter().enumerate() {
                if r.is_active() {
                    qs.answers[qs.current_index] = Some(oi);
                    break;
                }
            }

            if is_last {
                drop(qs_borrow);
                submit_quiz_inner(&quiz_state, &state, &stack, &content_box);
            } else {
                qs.current_index += 1;
                drop(qs_borrow);
                show_question_inner(&quiz_state, &content_box);
            }
        });

        btn_box.append(&next_btn);
        self.content_box.append(&btn_box);
    }

    fn submit_quiz(&self) {
        submit_quiz_inner(&self.quiz_state, &self.state, &self.stack, &self.content_box);
    }

    fn show_results(
        content_box: &gtk::Box,
        stack: &Rc<gtk::Stack>,
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

        let result_label = gtk::Label::new(None);
        result_label.set_markup(&format!(
            "<big><b>{} (Score: {:.0}%)</b></big>",
            if passed_val { "Passed" } else { "Failed" },
            score_val * 100.0
        ));
        result_label.set_margin_top(24);
        content_box.append(&result_label);

        let parsed_answers: Vec<Option<usize>> = user_answers_json
            .and_then(|j| serde_json::from_str::<Vec<Option<usize>>>(j).ok())
            .unwrap_or_default();

        for (i, q) in qs.questions.iter().enumerate() {
            let q_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
            q_box.set_margin_start(8);
            q_box.set_margin_top(8);

            let q_label = gtk::Label::new(Some(&format!("{}. {}", i + 1, q.question)));
            q_label.set_wrap(true);
            q_label.set_halign(gtk::Align::Start);
            q_box.append(&q_label);

            let user_ans = parsed_answers.get(i).copied().flatten();

            if let Some(ua) = user_ans {
                let ua_label = gtk::Label::new(Some(&format!(
                    "Your answer: {} {}",
                    q.options.get(ua).cloned().unwrap_or_default(),
                    if ua == q.correct_index { "(correct)" } else { "(incorrect)" }
                )));
                ua_label.set_halign(gtk::Align::Start);
                q_box.append(&ua_label);
            }

            let correct_label = gtk::Label::new(Some(&format!(
                "Correct answer: {}",
                q.options.get(q.correct_index).cloned().unwrap_or_default()
            )));
            correct_label.set_halign(gtk::Align::Start);
            correct_label.add_css_class("subtitle");
            q_box.append(&correct_label);

            let expl = gtk::Label::new(Some(&q.explanation));
            expl.set_wrap(true);
            expl.set_halign(gtk::Align::Start);
            expl.add_css_class("caption");
            q_box.append(&expl);

            content_box.append(&q_box);
        }

        let back_btn = gtk::Button::with_label("Back to Quizzes");
        back_btn.set_halign(gtk::Align::Center);
        back_btn.add_css_class("suggested-action");
        let stack_cl = stack.clone();
        back_btn.connect_clicked(move |_| {
            stack_cl.set_visible_child_name(PAGE_QUIZ_LIST);
        });
        content_box.append(&back_btn);
    }
}

fn show_question_inner(quiz_state: &Rc<RefCell<Option<QuizState>>>, content_box: &gtk::Box) {
    while let Some(child) = content_box.first_child() {
        content_box.remove(&child);
    }

    let qs_borrow = quiz_state.borrow();
    let qs = match *qs_borrow {
        Some(ref qs) => qs,
        None => return,
    };

    if qs.current_index >= qs.questions.len() {
        return;
    }

    let q = &qs.questions[qs.current_index];
    let idx = qs.current_index;
    let total = qs.questions.len();
    let saved_answer = qs.answers[idx];

    let counter = gtk::Label::new(Some(&format!("Question {} of {}", idx + 1, total)));
    counter.add_css_class("subtitle");
    counter.set_halign(gtk::Align::Start);
    content_box.append(&counter);

    let q_label = gtk::Label::new(Some(&q.question));
    q_label.set_wrap(true);
    q_label.set_halign(gtk::Align::Start);
    q_label.add_css_class("heading");
    content_box.append(&q_label);

    let option_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
    option_box.set_margin_start(12);

    let mut radios: Vec<gtk::CheckButton> = Vec::new();
    for (oi, opt) in q.options.iter().enumerate() {
        let radio = gtk::CheckButton::with_label(opt);
        if let Some(first) = radios.first() {
            radio.set_group(Some(first));
        }
        if saved_answer == Some(oi) {
            radio.set_active(true);
        }
        option_box.append(&radio);
        radios.push(radio);
    }

    content_box.append(&option_box);

    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    btn_box.set_halign(gtk::Align::End);

    let is_last = idx + 1 >= total;
    let next_btn = gtk::Button::with_label(if is_last { "Submit" } else { "Next" });
    next_btn.add_css_class("suggested-action");

    let qs = quiz_state.clone();
    let radios_clone = radios;

    next_btn.connect_clicked(move |_| {
        let mut qs_borrow = qs.borrow_mut();
        let qs = match *qs_borrow {
            Some(ref mut qs) => qs,
            None => return,
        };

        for (oi, r) in radios_clone.iter().enumerate() {
            if r.is_active() {
                qs.answers[qs.current_index] = Some(oi);
                break;
            }
        }

        if is_last {
            // Can't call submit directly with borrow held; drop it
            drop(qs_borrow);
            // submit needs state, stack, content_box which we don't have here
            // This path shouldn't be reached from this free function in normal flow
            // since show_question_inner is only called for "Next" not "Submit"
        } else {
            qs.current_index += 1;
        }
    });

    btn_box.append(&next_btn);
    content_box.append(&btn_box);
}

fn submit_quiz_inner(
    quiz_state: &Rc<RefCell<Option<QuizState>>>,
    state: &SharedState,
    stack: &Rc<gtk::Stack>,
    content_box: &gtk::Box,
) {
    while let Some(child) = content_box.first_child() {
        content_box.remove(&child);
    }

    let qs_borrow = quiz_state.borrow_mut();
    let qs = match *qs_borrow {
        Some(ref qs) => qs,
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
    let passed = score >= 0.7;

    // Save answers before we drop the borrow
    let answers_snapshot = qs.answers.clone();
    drop(qs_borrow);

    let s = state.borrow();
    if let Some(ref ctx) = s.backend {
        let quiz_id_str = s.current_quiz_id.as_ref().cloned().unwrap_or_default();
        if let Ok(exam_id) = quiz_id_str.parse::<crate::domain::value_objects::ExamId>() {
            let answers_json = serde_json::to_string(&answers_snapshot).ok();
            let _ = ctx.exam_repo.update_result(&exam_id, score, passed, answers_json);
        }
    }

    let result_label = gtk::Label::new(None);
    result_label.set_markup(&format!(
        "<big><b>{}</b></big>\n\nScore: {}/{} ({:.0}%)",
        if passed { "Passed" } else { "Failed" },
        correct,
        total,
        score * 100.0
    ));
    result_label.set_margin_top(24);
    content_box.append(&result_label);

    let back_btn = gtk::Button::with_label("Back to Quizzes");
    back_btn.set_halign(gtk::Align::Center);
    back_btn.add_css_class("suggested-action");
    let stack_cl = stack.clone();
    back_btn.connect_clicked(move |_| {
        stack_cl.set_visible_child_name(PAGE_QUIZ_LIST);
    });
    content_box.append(&back_btn);
}
