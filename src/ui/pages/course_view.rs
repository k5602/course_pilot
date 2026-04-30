use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use adw::NavigationView;
use adw::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::{CreateModuleInput, MoveVideoInput, UpdateModuleTitleInput};
use crate::domain::entities::Module;
use crate::domain::ports::{CourseRepository, ModuleRepository, VideoRepository};
use crate::domain::value_objects::ModuleId;
use crate::ui::navigation::PAGE_VIDEO_PLAYER;
use crate::ui::state::SharedState;

#[allow(clippy::type_complexity)]
pub struct CourseViewPage {
    widget: gtk::Box,
    state: SharedState,
    nav: Rc<NavigationView>,
    nav_pages: Rc<RefCell<Rc<HashMap<&'static str, adw::NavigationPage>>>>,
    content_box: gtk::Box,
    status_page: adw::StatusPage,
    refresh_cb: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
}

impl CourseViewPage {
    pub fn new(state: SharedState, nav: Rc<NavigationView>) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");

        let status_page = adw::StatusPage::new();
        status_page.set_title("Loading...");
        status_page.set_description(Some("Loading course..."));
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
            nav_pages: Rc::new(RefCell::new(Rc::new(HashMap::new()))),
            content_box,
            status_page,
            refresh_cb: Rc::new(RefCell::new(None)),
        }
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn set_refresh_cb(&self, cb: Rc<dyn Fn()>) {
        *self.refresh_cb.borrow_mut() = Some(cb);
    }

    pub fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, adw::NavigationPage>>) {
        *self.nav_pages.borrow_mut() = pages;
    }

    pub fn refresh(&self) {
        while let Some(child) = self.content_box.first_child() {
            self.content_box.remove(&child);
        }
        self.content_box.set_visible(false);
        self.status_page.set_visible(true);

        let refresh_cb =
            self.refresh_cb.borrow().as_ref().cloned().unwrap_or_else(|| Rc::new(|| {}));

        let state = self.state.borrow();
        let course_id_str = match state.current_course_id {
            Some(ref id) => id.clone(),
            None => {
                self.status_page.set_title("No Selection");
                self.status_page.set_description(Some("No course selected."));
                self.status_page.set_visible(true);
                return;
            },
        };

        if let Some(ref ctx) = state.backend {
            let course_id = match course_id_str.parse::<crate::domain::value_objects::CourseId>() {
                Ok(id) => id,
                Err(_) => {
                    self.status_page.set_title("Invalid ID");
                    self.status_page.set_description(Some("Invalid course ID."));
                    self.status_page.set_visible(true);
                    return;
                },
            };

            match ctx.course_repo.find_by_id(&course_id) {
                Ok(Some(course)) => {
                    self.status_page.set_visible(false);
                    self.content_box.set_visible(true);

                    let header = gtk::Box::new(gtk::Orientation::Vertical, 4);
                    let name_label = gtk::Label::new(Some(course.name()));
                    name_label.add_css_class("heading");
                    name_label.set_halign(gtk::Align::Start);
                    header.append(&name_label);

                    if let Some(desc) = course.description() {
                        let desc_label = gtk::Label::new(Some(desc));
                        desc_label.set_halign(gtk::Align::Start);
                        desc_label.set_wrap(true);
                        desc_label.add_css_class("subtitle");
                        header.append(&desc_label);
                    }

                    self.content_box.append(&header);

                    // New Module button
                    let new_mod_btn = gtk::Button::with_label("+ New Module");
                    new_mod_btn.set_halign(gtk::Align::Start);
                    new_mod_btn.add_css_class("flat");
                    new_mod_btn.set_margin_start(8);
                    new_mod_btn.set_margin_bottom(8);

                    let new_mod_state = self.state.clone();
                    let course_id_new = course.id().clone();
                    let new_mod_cb = refresh_cb.clone();
                    new_mod_btn.connect_clicked(move |_| {
                        let s = new_mod_state.borrow();
                        if let Some(ref ctx) = s.backend {
                            let modules =
                                ctx.module_repo.find_by_course(&course_id_new).unwrap_or_default();
                            let next_order = modules.len() as u32;
                            let _ = ServiceFactory::create_module(ctx).execute(CreateModuleInput {
                                course_id: course_id_new.clone(),
                                title: format!("Module {}", next_order + 1),
                                sort_order: next_order,
                            });
                        }
                        drop(s);
                        new_mod_cb();
                    });
                    self.content_box.append(&new_mod_btn);

                    let modules = match ctx.module_repo.find_by_course(course.id()) {
                        Ok(m) => m,
                        Err(_) => {
                            let err_mod = gtk::Label::new(Some("Failed to load modules."));
                            err_mod.add_css_class("subtitle");
                            self.content_box.append(&err_mod);
                            return;
                        },
                    };

                    for module in &modules {
                        let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                        let title_label = gtk::Label::new(Some(module.title()));
                        title_label.set_halign(gtk::Align::Start);
                        title_label.set_hexpand(true);
                        title_box.append(&title_label);

                        let edit_btn = gtk::Button::from_icon_name("document-edit-symbolic");
                        edit_btn.add_css_class("flat");
                        edit_btn.set_valign(gtk::Align::Center);
                        title_box.append(&edit_btn);

                        let expander = gtk::Expander::new(Some(""));
                        expander.set_label_widget(Some(&title_box));
                        expander.set_expanded(false);
                        expander.set_margin_start(8);

                        let edit_state = self.state.clone();
                        let edit_module_id = module.id().clone();
                        let edit_title = module.title().to_string();
                        let edit_cb = refresh_cb.clone();
                        edit_btn.connect_clicked(move |_| {
                            show_rename_module_dialog(
                                edit_state.clone(),
                                edit_module_id.clone(),
                                edit_title.clone(),
                                edit_cb.clone(),
                            );
                        });

                        let video_box = gtk::Box::new(gtk::Orientation::Vertical, 4);
                        video_box.set_margin_start(12);
                        video_box.set_margin_top(4);
                        video_box.set_margin_bottom(4);

                        let videos = match ctx.video_repo.find_by_module(module.id()) {
                            Ok(v) => v,
                            Err(_) => {
                                let err_vid = gtk::Label::new(Some("Failed to load videos."));
                                err_vid.add_css_class("subtitle");
                                video_box.append(&err_vid);
                                expander.set_child(Some(&video_box));
                                self.content_box.append(&expander);
                                continue;
                            },
                        };

                        let pages_borrow = self.nav_pages.borrow();
                        for (vid_idx, video) in videos.iter().enumerate() {
                            let row = Self::build_video_row(
                                video,
                                &self.state,
                                &self.nav,
                                &pages_borrow,
                                vid_idx,
                                videos.len(),
                                module.id(),
                                &modules,
                                refresh_cb.clone(),
                            );
                            video_box.append(&row);
                        }

                        if videos.is_empty() {
                            let empty_mod = gtk::Label::new(Some("No videos in this module."));
                            empty_mod.add_css_class("subtitle");
                            empty_mod.set_margin_start(12);
                            video_box.append(&empty_mod);
                        }

                        expander.set_child(Some(&video_box));
                        self.content_box.append(&expander);
                    }

                    if modules.is_empty() {
                        let no_modules = gtk::Label::new(Some("No modules in this course."));
                        no_modules.add_css_class("subtitle");
                        self.content_box.append(&no_modules);
                    }
                },
                Ok(None) => {
                    self.status_page.set_title("Not Found");
                    self.status_page.set_description(Some("Course not found."));
                    self.status_page.set_visible(true);
                },
                Err(e) => {
                    self.status_page.set_title("Error");
                    self.status_page.set_description(Some(&format!("Error loading course: {}", e)));
                    self.status_page.set_visible(true);
                },
            }
        } else {
            self.status_page.set_title("No Backend");
            self.status_page.set_description(Some("No backend connected."));
            self.status_page.set_visible(true);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_video_row(
        video: &crate::domain::entities::Video,
        state: &SharedState,
        nav: &Rc<NavigationView>,
        nav_pages: &HashMap<&'static str, adw::NavigationPage>,
        video_position: usize,
        video_count: usize,
        current_module_id: &ModuleId,
        all_modules: &[Module],
        refresh_cb: Rc<dyn Fn()>,
    ) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_margin_top(2);
        row.set_margin_bottom(2);

        let complete_btn = gtk::CheckButton::new();
        complete_btn.set_active(video.is_completed());
        complete_btn.set_valign(gtk::Align::Center);
        row.append(&complete_btn);

        // Move Up button
        if video_position > 0 {
            let up_btn = gtk::Button::from_icon_name("go-up-symbolic");
            up_btn.add_css_class("flat");
            up_btn.set_valign(gtk::Align::Center);
            up_btn.set_tooltip_text(Some("Move Up"));

            let up_state = state.clone();
            let up_video_id = video.id().clone();
            let up_module_id = current_module_id.clone();
            let up_cb = refresh_cb.clone();
            up_btn.connect_clicked(move |_| {
                let s = up_state.borrow();
                if let Some(ref ctx) = s.backend
                    && let Ok(videos) = ctx.video_repo.find_by_module(&up_module_id)
                {
                    let mut sorted: Vec<_> = videos.iter().collect();
                    sorted.sort_by_key(|v| v.sort_order());
                    let pos = sorted.iter().position(|v| v.id() == &up_video_id);
                    if let Some(pos) = pos
                        && pos > 0
                    {
                        let current = sorted[pos];
                        let adjacent = sorted[pos - 1];
                        let _ = ctx.video_repo.update_module(
                            current.id(),
                            &up_module_id,
                            adjacent.sort_order(),
                        );
                        let _ = ctx.video_repo.update_module(
                            adjacent.id(),
                            &up_module_id,
                            current.sort_order(),
                        );
                    }
                }
                drop(s);
                up_cb();
            });
            row.append(&up_btn);
        }

        // Move Down button
        if video_position + 1 < video_count {
            let down_btn = gtk::Button::from_icon_name("go-down-symbolic");
            down_btn.add_css_class("flat");
            down_btn.set_valign(gtk::Align::Center);
            down_btn.set_tooltip_text(Some("Move Down"));

            let down_state = state.clone();
            let down_video_id = video.id().clone();
            let down_module_id = current_module_id.clone();
            let down_cb = refresh_cb.clone();
            down_btn.connect_clicked(move |_| {
                let s = down_state.borrow();
                if let Some(ref ctx) = s.backend
                    && let Ok(videos) = ctx.video_repo.find_by_module(&down_module_id)
                {
                    let mut sorted: Vec<_> = videos.iter().collect();
                    sorted.sort_by_key(|v| v.sort_order());
                    let pos = sorted.iter().position(|v| v.id() == &down_video_id);
                    if let Some(pos) = pos
                        && pos + 1 < sorted.len()
                    {
                        let current = sorted[pos];
                        let adjacent = sorted[pos + 1];
                        let _ = ctx.video_repo.update_module(
                            current.id(),
                            &down_module_id,
                            adjacent.sort_order(),
                        );
                        let _ = ctx.video_repo.update_module(
                            adjacent.id(),
                            &down_module_id,
                            current.sort_order(),
                        );
                    }
                }
                drop(s);
                down_cb();
            });
            row.append(&down_btn);
        }

        // Move to module dropdown
        if all_modules.len() > 1 {
            let module_titles: Vec<&str> = all_modules.iter().map(|m| m.title()).collect();
            let dropdown = gtk::DropDown::from_strings(&module_titles);
            let current_mod_idx =
                all_modules.iter().position(|m| m.id() == current_module_id).unwrap_or(0);
            dropdown.set_selected(current_mod_idx as u32);
            dropdown.set_valign(gtk::Align::Center);

            let mv_state = state.clone();
            let mv_video_id = video.id().clone();
            let mv_modules: Vec<Module> = all_modules.to_vec();
            let cur_mod_id = current_module_id.clone();
            let mv_ref_cb = refresh_cb.clone();
            dropdown.connect_selected_notify(move |dd| {
                let idx = dd.selected() as usize;
                if idx >= mv_modules.len() {
                    return;
                }
                let target = &mv_modules[idx];
                if target.id() == &cur_mod_id {
                    return;
                }
                let s = mv_state.borrow();
                if let Some(ref ctx) = s.backend {
                    let uc = ServiceFactory::move_video_to_module(ctx);
                    let _ = uc.execute(MoveVideoInput {
                        video_id: mv_video_id.clone(),
                        target_module_id: target.id().clone(),
                        sort_order: 0,
                    });
                }
                drop(s);
                mv_ref_cb();
            });
            row.append(&dropdown);
        }

        let title_label = gtk::Label::new(Some(video.title()));
        title_label.set_halign(gtk::Align::Start);
        title_label.set_hexpand(true);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        row.append(&title_label);

        let dur_mins = video.duration_secs() / 60;
        let dur_secs = video.duration_secs() % 60;
        let dur_label = gtk::Label::new(Some(&format!("{:02}:{:02}", dur_mins, dur_secs)));
        dur_label.add_css_class("subtitle");
        dur_label.set_valign(gtk::Align::Center);
        row.append(&dur_label);

        let play_btn = gtk::Button::with_label("Play");
        play_btn.set_valign(gtk::Align::Center);
        play_btn.add_css_class("circular");

        let state_cl = state.clone();
        let nav_cl = nav.clone();
        let vp_page = nav_pages.get(PAGE_VIDEO_PLAYER).cloned();
        let video_id = video.id().to_string();
        play_btn.connect_clicked(move |_| {
            state_cl.borrow_mut().current_video_id = Some(video_id.clone());
            if let Some(ref vp_page) = vp_page {
                nav_cl.push(vp_page);
            }
        });
        row.append(&play_btn);

        let has_llm = state
            .borrow()
            .backend
            .as_ref()
            .and_then(|ctx| crate::application::ServiceFactory::summarize_video(ctx))
            .is_some();

        let summarize_btn = gtk::Button::with_label("Summarize");
        summarize_btn.add_css_class("flat");
        summarize_btn.set_valign(gtk::Align::Center);
        summarize_btn.set_sensitive(has_llm);
        if !has_llm {
            summarize_btn.set_tooltip_text(Some("No LLM API key configured in Settings"));
        }
        let sum_state = state.clone();
        let sum_video_id = video.id().clone();
        summarize_btn.connect_clicked(move |_| {
            sum_state.borrow_mut().current_video_id = Some(sum_video_id.to_string());
            crate::ui::toast::Toast::show("Summarization started. Check the AI Chat panel.");
            let s = sum_state.borrow();
            if let Some(ref ctx) = s.backend
                && let Some(uc) = crate::application::ServiceFactory::summarize_video(ctx)
            {
                let input = crate::application::use_cases::SummarizeVideoInput {
                    video_id: sum_video_id.clone(),
                    force_refresh: false,
                };
                crate::infrastructure::tokio_bridge::spawn(async move {
                    if let Err(e) = uc.execute(input).await {
                        log::error!("Summarization failed: {e}");
                    }
                });
            }
        });
        row.append(&summarize_btn);

        let quiz_btn = gtk::Button::with_label("Generate Quiz");
        quiz_btn.add_css_class("flat");
        quiz_btn.set_valign(gtk::Align::Center);
        quiz_btn.set_sensitive(has_llm);
        if !has_llm {
            quiz_btn.set_tooltip_text(Some("No LLM API key configured in Settings"));
        }
        let quiz_state = state.clone();
        let quiz_video_id = video.id().clone();
        quiz_btn.connect_clicked(move |_| {
            quiz_state.borrow_mut().current_video_id = Some(quiz_video_id.to_string());
            crate::ui::toast::Toast::show("Quiz generation started. Check the AI Chat panel.");
            let s = quiz_state.borrow();
            if let Some(ref ctx) = s.backend
                && let Some(uc) = crate::application::ServiceFactory::take_exam(ctx)
            {
                use crate::domain::value_objects::ExamDifficulty;
                let input = crate::application::use_cases::GenerateExamInput {
                    video_id: quiz_video_id.clone(),
                    num_questions: 5,
                    difficulty: ExamDifficulty::Medium,
                };
                crate::infrastructure::tokio_bridge::spawn(async move {
                    if let Err(e) = uc.generate(input).await {
                        log::error!("Quiz generation failed: {e}");
                    }
                });
            }
        });
        row.append(&quiz_btn);

        row
    }
}

fn show_rename_module_dialog(
    state: SharedState,
    module_id: ModuleId,
    current_title: String,
    refresh_cb: Rc<dyn Fn()>,
) {
    let dialog = adw::Dialog::new();
    dialog.set_title("Rename Module");
    dialog.set_content_width(350);
    dialog.set_content_height(180);

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
    vbox.set_margin_start(16);
    vbox.set_margin_end(16);
    vbox.set_margin_top(16);
    vbox.set_margin_bottom(16);

    let entry = gtk::Entry::new();
    entry.set_text(&current_title);
    vbox.append(&entry);

    let save_btn = gtk::Button::with_label("Save");
    save_btn.add_css_class("suggested-action");
    vbox.append(&save_btn);

    dialog.set_child(Some(&vbox));

    let dialog_cl = dialog.clone();
    save_btn.connect_clicked(move |_| {
        let new_title = entry.text().to_string();
        if !new_title.trim().is_empty() {
            let s = state.borrow();
            if let Some(ref ctx) = s.backend {
                let uc = ServiceFactory::update_module_title(ctx);
                let _ = uc.execute(UpdateModuleTitleInput {
                    module_id: module_id.clone(),
                    title: new_title.trim().to_string(),
                });
            }
            drop(s);
            refresh_cb();
        }
        dialog_cl.close();
    });

    dialog.present(None::<&gtk::Window>);
}
