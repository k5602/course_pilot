use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use adw::NavigationView;
use adw::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::{
    CreateModuleInput, DeleteModuleInput, MoveVideoInput, UpdateModuleTitleInput,
};
use crate::domain::ports::{CourseRepository, ModuleRepository, VideoRepository};
use crate::domain::value_objects::ModuleId;
use crate::ui::navigation::PAGE_VIDEO_PLAYER;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;
use crate::ui::types::RefreshCallback;

#[allow(clippy::type_complexity)]
pub struct CourseViewPage {
    widget: gtk::Box,
    state: SharedState,
    nav: Rc<NavigationView>,
    nav_pages: Rc<RefCell<Rc<HashMap<&'static str, adw::NavigationPage>>>>,
    content_box: gtk::Box,
    status_page: adw::StatusPage,
    refresh_cb: RefreshCallback,
    expanded_modules: Rc<RefCell<HashSet<String>>>,
}

struct VideoRowContext<'a> {
    video: &'a crate::domain::entities::Video,
    all_videos: &'a [crate::domain::entities::Video],
    state: &'a SharedState,
    nav: &'a Rc<NavigationView>,
    nav_pages: &'a HashMap<&'static str, adw::NavigationPage>,
    current_module_id: &'a ModuleId,
    expanded_modules: Rc<RefCell<HashSet<String>>>,
    refresh_cb: Rc<dyn Fn()>,
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
            expanded_modules: Rc::new(RefCell::new(HashSet::new())),
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

                    let title_row = gtk::Box::new(gtk::Orientation::Horizontal, 8);

                    let name_label = gtk::Label::new(Some(course.name()));
                    name_label.add_css_class("heading");
                    name_label.set_halign(gtk::Align::Start);
                    name_label.set_hexpand(true);
                    title_row.append(&name_label);

                    let delete_course_btn = gtk::Button::from_icon_name("user-trash-symbolic");
                    delete_course_btn.add_css_class("flat");
                    delete_course_btn.set_valign(gtk::Align::Center);
                    delete_course_btn.set_tooltip_text(Some("Delete Course"));
                    title_row.append(&delete_course_btn);

                    header.append(&title_row);

                    let delete_state = self.state.clone();
                    let course_id_del = *course.id();
                    let nav_del = self.nav.clone();
                    let parent_window = self
                        .widget
                        .root()
                        .and_then(|r| r.downcast::<adw::ApplicationWindow>().ok());
                    let refresh_cb_del = refresh_cb.clone();

                    delete_course_btn.connect_clicked(move |_| {
                        let dialog = adw::AlertDialog::new(
                            Some("Delete Course?"),
                            Some("This will permanently delete the course and all associated modules, videos, notes, and exams. This action cannot be undone.")
                        );
                        dialog.add_response("cancel", "Cancel");
                        dialog.add_response("delete", "Delete");
                        dialog.set_response_appearance("delete", adw::ResponseAppearance::Destructive);
                        dialog.set_default_response(Some("cancel"));

                        let delete_state2 = delete_state.clone();
                        let course_id_del2 = course_id_del;
                        let nav_del2 = nav_del.clone();
                        let refresh_cb_del2 = refresh_cb_del.clone();

                        dialog.connect_response(None, move |_, response| {
                            if response == "delete" {
                                let s = delete_state2.borrow();
                                if let Some(ref ctx) = s.backend {
                                    if let Err(e) = ctx.course_repo.delete(&course_id_del2) {
                                        Toast::show_error(&format!("Failed to delete course: {}", e));
                                    } else {
                                        Toast::show("Course deleted successfully.");
                                        drop(s);
                                        let mut s_mut = delete_state2.borrow_mut();
                                        s_mut.current_course_id = None;
                                        drop(s_mut);
                                        nav_del2.pop();
                                        refresh_cb_del2();
                                    }
                                }
                            }
                        });
                        if let Some(ref win) = parent_window {
                            dialog.present(Some(win));
                        }
                    });

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
                    let course_id_new = *course.id();
                    let new_mod_cb = refresh_cb.clone();
                    new_mod_btn.connect_clicked(move |_| {
                        let s = new_mod_state.borrow();
                        if let Some(ref ctx) = s.backend {
                            let modules =
                                ctx.module_repo.find_by_course(&course_id_new).unwrap_or_default();
                            let next_order = modules.len() as u32;
                            if let Err(e) =
                                ServiceFactory::create_module(ctx).execute(CreateModuleInput {
                                    course_id: course_id_new,
                                    title: format!("Module {}", next_order + 1),
                                    sort_order: next_order,
                                })
                            {
                                Toast::show_error(&format!("Failed to create module: {}", e));
                            }
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

                        let delete_btn = gtk::Button::from_icon_name("user-trash-symbolic");
                        delete_btn.add_css_class("flat");
                        delete_btn.set_valign(gtk::Align::Center);
                        delete_btn.set_tooltip_text(Some("Delete Module"));
                        title_box.append(&delete_btn);

                        let expander = gtk::Expander::new(Some(""));
                        expander.set_label_widget(Some(&title_box));
                        let module_id_str = module.id().as_uuid().to_string();
                        let is_expanded = self.expanded_modules.borrow().contains(&module_id_str);
                        expander.set_expanded(is_expanded);
                        expander.set_margin_start(8);

                        let expanded_modules_ref = self.expanded_modules.clone();
                        let exp_module_id = module.id().as_uuid().to_string();
                        expander.connect_notify_local(Some("expanded"), move |exp, _| {
                            let mut set = expanded_modules_ref.borrow_mut();
                            if exp.is_expanded() {
                                set.insert(exp_module_id.clone());
                            } else {
                                set.remove(&exp_module_id);
                            }
                        });

                        let edit_state = self.state.clone();
                        let edit_module_id = *module.id();
                        let edit_title = module.title().to_string();
                        let edit_cb = refresh_cb.clone();
                        edit_btn.connect_clicked(move |_| {
                            show_rename_module_dialog(
                                edit_state.clone(),
                                edit_module_id,
                                edit_title.clone(),
                                edit_cb.clone(),
                            );
                        });

                        let del_state = self.state.clone();
                        let del_module_id = *module.id();
                        let del_module_title = module.title().to_string();
                        let del_cb = refresh_cb.clone();
                        delete_btn.connect_clicked(move |_| {
                            let s = del_state.borrow();
                            if let Some(ref ctx) = s.backend {
                                let has_videos = ctx
                                    .video_repo
                                    .find_by_module(&del_module_id)
                                    .map(|v| !v.is_empty())
                                    .unwrap_or(false);

                                if has_videos {
                                    let dialog = adw::Dialog::new();
                                    dialog.set_title("Delete Module");
                                    dialog.set_content_width(400);
                                    dialog.set_content_height(200);

                                    let vbox =
                                        gtk::Box::new(gtk::Orientation::Vertical, 12);
                                    vbox.set_margin_start(16);
                                    vbox.set_margin_end(16);
                                    vbox.set_margin_top(16);
                                    vbox.set_margin_bottom(16);

                                    let msg = gtk::Label::new(Some(&format!(
                                        "Module \"{}\" has videos. They will be deleted. Delete anyway?",
                                        del_module_title
                                    )));
                                    msg.set_wrap(true);
                                    vbox.append(&msg);

                                    let btn_box =
                                        gtk::Box::new(gtk::Orientation::Horizontal, 8);
                                    btn_box.set_halign(gtk::Align::End);

                                    let cancel_btn =
                                        gtk::Button::with_label("Cancel");
                                    btn_box.append(&cancel_btn);

                                    let confirm_btn =
                                        gtk::Button::with_label("Delete");
                                    confirm_btn
                                        .add_css_class("destructive-action");
                                    btn_box.append(&confirm_btn);

                                    vbox.append(&btn_box);
                                    dialog.set_child(Some(&vbox));

                                    let dialog_cl = dialog.clone();
                                    let dialog_cl2 = dialog.clone();
                                    cancel_btn.connect_clicked(move |_| {
                                        dialog_cl.close();
                                    });

                                    let del_state2 = del_state.clone();
                                    let del_id2 = del_module_id;
                                    let del_cb2 = del_cb.clone();
                                    confirm_btn.connect_clicked(move |_| {
                                        let s2 = del_state2.borrow();
                                        if let Some(ref ctx2) = s2.backend
                                            && let Err(e) =
                                                ServiceFactory::delete_module(ctx2)
                                                    .execute(DeleteModuleInput {
                                                        module_id: del_id2,
                                                        force: true,
                                                    })
                                        {
                                            Toast::show_error(&format!(
                                                "Failed to delete module: {}",
                                                e
                                            ));
                                        }
                                        drop(s2);
                                        del_cb2();
                                        dialog_cl2.close();
                                    });

                                    dialog.present(None::<&gtk::Window>);
                                } else {
                                    if let Err(e) =
                                        ServiceFactory::delete_module(ctx).execute(
                                            DeleteModuleInput {
                                                module_id: del_module_id,
                                                force: false,
                                            },
                                        )
                                    {
                                        Toast::show_error(&format!(
                                            "Failed to delete module: {}",
                                            e
                                        ));
                                    }
                                    drop(s);
                                    del_cb();
                                }
                            }
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
                        for video in videos.iter() {
                            let row = Self::build_video_row(VideoRowContext {
                                video,
                                all_videos: &videos,
                                state: &self.state,
                                nav: &self.nav,
                                nav_pages: &pages_borrow,
                                current_module_id: module.id(),
                                expanded_modules: self.expanded_modules.clone(),
                                refresh_cb: refresh_cb.clone(),
                            });
                            video_box.append(&row);
                        }

                        if videos.is_empty() {
                            let empty_mod = gtk::Label::new(Some("No videos in this module."));
                            empty_mod.add_css_class("subtitle");
                            empty_mod.set_margin_start(12);
                            video_box.append(&empty_mod);
                        }

                        // Fallback drop zone at the end of the module (for empty modules
                        // or dropping after the last item).
                        {
                            let fb_dt = gtk::DropTarget::new(
                                glib::Type::STRING,
                                gtk::gdk::DragAction::MOVE,
                            );
                            let fb_state = self.state.clone();
                            let fb_module_id = *module.id();
                            let fb_expanded = self.expanded_modules.clone();
                            let fb_module_id_str = module.id().as_uuid().to_string();
                            let fb_cb = refresh_cb.clone();
                            fb_dt.connect_drop(move |_, value, _, _| {
                                let Ok(payload) = value.get::<String>() else {
                                    return false;
                                };
                                let Some((vid_str, src_mod_str)) = payload.split_once(':') else {
                                    return false;
                                };
                                let Ok(moved_id) =
                                    vid_str.parse::<crate::domain::value_objects::VideoId>()
                                else {
                                    return false;
                                };
                                let s = fb_state.borrow();
                                if let Some(ref ctx) = s.backend {
                                    let is_same_module = src_mod_str == fb_module_id_str.as_str();
                                    if is_same_module {
                                        // Within-module: move to end by giving it the highest
                                        // sort_order.
                                        let existing = ctx
                                            .video_repo
                                            .find_by_module(&fb_module_id)
                                            .unwrap_or_default();
                                        let max_order = existing
                                            .iter()
                                            .filter(|v| v.id() != &moved_id)
                                            .map(|v| v.sort_order())
                                            .max()
                                            .unwrap_or(0)
                                            .saturating_add(1);
                                        if let Err(e) = ctx.video_repo.update_module(
                                            &moved_id,
                                            &fb_module_id,
                                            max_order,
                                        ) {
                                            Toast::show_error(&format!("Failed to reorder: {}", e));
                                            return false;
                                        }
                                    } else {
                                        // Cross-module: move to end of target module.
                                        let uc = ServiceFactory::move_video_to_module(ctx);
                                        if let Err(e) = uc.execute(MoveVideoInput {
                                            video_id: moved_id,
                                            target_module_id: fb_module_id,
                                            sort_order: 0, // use case appends to end
                                        }) {
                                            Toast::show_error(&format!("Failed to move: {}", e));
                                            return false;
                                        }
                                    }
                                }
                                drop(s);
                                fb_expanded.borrow_mut().insert(fb_module_id_str.clone());
                                fb_cb();
                                true
                            });
                            video_box.add_controller(fb_dt);
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

    fn build_video_row(ctx: VideoRowContext<'_>) -> gtk::Box {
        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        row.set_margin_top(2);
        row.set_margin_bottom(2);

        let complete_btn = gtk::CheckButton::new();
        complete_btn.set_active(ctx.video.is_completed());
        complete_btn.set_valign(gtk::Align::Center);

        let comp_state = ctx.state.clone();
        let comp_video_id = *ctx.video.id();
        let comp_cb = ctx.refresh_cb.clone();
        complete_btn.connect_toggled(move |btn| {
            let s = comp_state.borrow();
            if let Some(ref ctx) = s.backend {
                let _ = ctx.video_repo.update_completion(&comp_video_id, btn.is_active());
            }
            drop(s);
            comp_cb();
        });

        row.append(&complete_btn);

        let title_label = gtk::Label::new(Some(ctx.video.title()));
        title_label.set_halign(gtk::Align::Start);
        title_label.set_hexpand(true);
        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
        title_label.set_tooltip_text(Some(ctx.video.title()));
        row.append(&title_label);

        {
            let drag_source = gtk::DragSource::new();
            drag_source.set_actions(gtk::gdk::DragAction::MOVE);
            let drag_video_id = *ctx.video.id();
            let drag_module_id = *ctx.current_module_id;
            drag_source.connect_prepare(move |_, _, _| {
                let payload = format!("{}:{}", drag_video_id, drag_module_id);
                let val = glib::Value::from(payload);
                Some(gtk::gdk::ContentProvider::for_value(&val))
            });
            let drag_row_begin = row.clone();
            drag_source.connect_drag_begin(move |_, _| {
                drag_row_begin.add_css_class("dragging");
            });
            let drag_row_end = row.clone();
            drag_source.connect_drag_end(move |_, _, _| {
                drag_row_end.remove_css_class("dragging");
            });
            // Attach to the whole row so any part of it can initiate drag
            row.add_controller(drag_source);
        }

        // Source badge (YT / Local)
        let source_badge = gtk::Label::new(Some(match ctx.video.source() {
            crate::domain::value_objects::VideoSource::YouTube(_) => "YT",
            crate::domain::value_objects::VideoSource::LocalPath(_) => "Local",
        }));
        source_badge.add_css_class("caption");
        source_badge.add_css_class("tag");
        source_badge.set_valign(gtk::Align::Center);
        row.append(&source_badge);

        let total_secs = ctx.video.duration_secs();
        let hours = total_secs / 3600;
        let mins = (total_secs % 3600) / 60;
        let secs = total_secs % 60;
        let formatted_dur = if hours > 0 {
            format!("{}:{:02}:{:02}", hours, mins, secs)
        } else {
            format!("{:02}:{:02}", mins, secs)
        };
        let dur_label = gtk::Label::new(Some(&formatted_dur));
        dur_label.add_css_class("subtitle");
        dur_label.set_valign(gtk::Align::Center);
        row.append(&dur_label);

        let play_btn = gtk::Button::with_label("Play");
        play_btn.set_valign(gtk::Align::Center);
        play_btn.add_css_class("circular");

        let state_cl = ctx.state.clone();
        let nav_cl = ctx.nav.clone();
        let vp_page = ctx.nav_pages.get(PAGE_VIDEO_PLAYER).cloned();
        let vid_str = ctx.video.id().to_string();
        play_btn.connect_clicked(move |_| {
            state_cl.borrow_mut().current_video_id = Some(vid_str.clone());
            if let Some(ref vp_page) = vp_page {
                nav_cl.push(vp_page);
            }
        });
        row.append(&play_btn);

        let has_llm = ctx
            .state
            .borrow()
            .backend
            .as_ref()
            .and_then(|app_ctx| ServiceFactory::summarize_video(app_ctx))
            .is_some();

        let menu_model = gio::Menu::new();

        if has_llm {
            let summarize_section = gio::Menu::new();
            summarize_section.append(Some("Summarize"), Some("video.summarize"));
            menu_model.append_section(None::<&str>, &summarize_section);

            let quiz_section = gio::Menu::new();
            quiz_section.append(Some("Generate Quiz"), Some("video.quiz"));
            menu_model.append_section(None::<&str>, &quiz_section);
        } else {
            // Show disabled-looking items with reason
            let no_llm_section = gio::Menu::new();
            no_llm_section.append(Some("Summarize (needs API key)"), Some("video.noop"));
            no_llm_section.append(Some("Generate Quiz (needs API key)"), Some("video.noop"));
            menu_model.append_section(None::<&str>, &no_llm_section);
        }

        // Note: Move Up / Move Down / Move To removed — use drag & drop to reorder

        let popover = gtk::PopoverMenu::from_model(Some(&menu_model));

        let action_group = gio::SimpleActionGroup::new();

        {
            let action = gio::SimpleAction::new("summarize", None);
            let s = ctx.state.clone();
            let vid = *ctx.video.id();
            action.connect_activate(move |_, _| {
                s.borrow_mut().current_video_id = Some(vid.to_string());
                Toast::show("Summarization started. Check the AI Chat panel.");
                let sb = s.borrow();
                if let Some(ref ctx) = sb.backend
                    && let Some(uc) = ServiceFactory::summarize_video(ctx)
                {
                    let input = crate::application::use_cases::SummarizeVideoInput {
                        video_id: vid,
                        force_refresh: false,
                    };
                    crate::infrastructure::tokio_bridge::spawn(async move {
                        if let Err(e) = uc.execute(input).await {
                            log::error!("Summarization failed: {e}");
                        }
                    });
                }
            });
            action_group.add_action(&action);
        }

        {
            let action = gio::SimpleAction::new("quiz", None);
            let s = ctx.state.clone();
            let vid = *ctx.video.id();
            action.connect_activate(move |_, _| {
                s.borrow_mut().current_video_id = Some(vid.to_string());
                Toast::show("Quiz generation started. Check the AI Chat panel.");
                let sb = s.borrow();
                if let Some(ref ctx) = sb.backend
                    && let Some(uc) = ServiceFactory::take_exam(ctx)
                {
                    use crate::domain::value_objects::ExamDifficulty;
                    let input = crate::application::use_cases::GenerateExamInput {
                        video_id: vid,
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
            action_group.add_action(&action);
        }

        // noop action — placeholder for disabled menu items when no LLM key
        {
            let action = gio::SimpleAction::new("noop", None);
            action.set_enabled(false);
            action_group.add_action(&action);
        }

        row.insert_action_group("video", Some(&action_group));

        let menu_btn = gtk::MenuButton::new();
        menu_btn.set_icon_name("view-more-symbolic");
        menu_btn.add_css_class("flat");
        menu_btn.set_valign(gtk::Align::Center);
        menu_btn.set_tooltip_text(Some("Actions"));
        menu_btn.set_popover(Some(&popover));
        row.append(&menu_btn);

        // Per-row DropTarget: detects y-coordinate to insert before or after this row.
        {
            let row_dt = gtk::DropTarget::new(glib::Type::STRING, gtk::gdk::DragAction::MOVE);
            // Visual hover feedback: add CSS indicator class
            let row_hover = row.clone();
            row_dt.connect_motion(move |_, _, y| {
                let h = row_hover.height() as f64;
                let half = if h > 0.0 { h / 2.0 } else { 18.0 };
                if y < half {
                    row_hover.add_css_class("drop-above");
                    row_hover.remove_css_class("drop-below");
                } else {
                    row_hover.add_css_class("drop-below");
                    row_hover.remove_css_class("drop-above");
                }
                gtk::gdk::DragAction::MOVE
            });
            let row_leave = row.clone();
            row_dt.connect_leave(move |_| {
                row_leave.remove_css_class("drop-above");
                row_leave.remove_css_class("drop-below");
            });

            // Capture everything needed in the drop handler.
            let drop_state = ctx.state.clone();
            let drop_module_id = *ctx.current_module_id;
            let drop_module_id_str = ctx.current_module_id.as_uuid().to_string();
            let drop_expanded = ctx.expanded_modules.clone();
            let drop_cb = ctx.refresh_cb.clone();
            let drop_row = row.clone();
            // We need index and sibling IDs to compute the new position.
            // Collect (id, sort_order) pairs sorted by sort_order for insertion math.
            let target_video_id = *ctx.video.id();
            let target_sort_order = ctx.video.sort_order();
            // Capture ordered list of (video_id, sort_order) for the same module.
            let ordered_ids: Vec<(crate::domain::value_objects::VideoId, u32)> =
                ctx.all_videos.iter().map(|v| (*v.id(), v.sort_order())).collect();

            row_dt.connect_drop(move |_, value, _, y| {
                // Clear visual indicator
                drop_row.remove_css_class("drop-above");
                drop_row.remove_css_class("drop-below");

                let Ok(payload) = value.get::<String>() else {
                    return false;
                };
                let Some((vid_str, src_mod_str)) = payload.split_once(':') else {
                    return false;
                };
                let Ok(moved_id) = vid_str.parse::<crate::domain::value_objects::VideoId>() else {
                    return false;
                };

                // Don't drop a row onto itself
                if moved_id == target_video_id {
                    return false;
                }

                let row_height = drop_row.height() as f64;
                let half = if row_height > 0.0 { row_height / 2.0 } else { 18.0 };
                let insert_before = y < half;

                let is_same_module = src_mod_str == drop_module_id_str.as_str();

                let s = drop_state.borrow();
                if let Some(ref ctx) = s.backend {
                    if is_same_module {
                        // Within-module reorder.
                        // Strategy: assign the dragged video a sort_order that places it
                        // just before or after the target.
                        //
                        // We use a simple approach: find the target's sort_order, then
                        // update only the moved video. If a conflict arises (two videos
                        // with the same sort_order), the DB fallback is to rely on stable
                        // ordering; a full re-number isn't required here.
                        //
                        // Better approach: collect all sort_orders, remove the moved one,
                        // re-insert it at the desired position, then update_module for
                        // each changed video.

                        // Build a mutable ordered list excluding the moved item.
                        let mut positions: Vec<(crate::domain::value_objects::VideoId, u32)> =
                            ordered_ids.iter().filter(|(id, _)| id != &moved_id).cloned().collect();

                        // Find where the target sits.
                        let target_pos =
                            positions.iter().position(|(id, _)| id == &target_video_id);

                        let insert_at = match target_pos {
                            Some(tp) => {
                                if insert_before {
                                    tp
                                } else {
                                    tp + 1
                                }
                            },
                            None => positions.len(),
                        };
                        let insert_at = insert_at.min(positions.len());

                        // Insert the dragged item at that position.
                        positions.insert(insert_at, (moved_id, 0));

                        // Assign contiguous sort_orders starting from 0 and persist only
                        // the ones whose sort_order changed.
                        let mut had_error = false;
                        for (new_order, (vid_id, old_order)) in positions.iter().enumerate() {
                            let new_order = new_order as u32;
                            if (new_order != *old_order || vid_id == &moved_id)
                                && let Err(e) =
                                    ctx.video_repo.update_module(vid_id, &drop_module_id, new_order)
                            {
                                Toast::show_error(&format!("Failed to reorder: {}", e));
                                had_error = true;
                                break;
                            }
                        }
                        if had_error {
                            return false;
                        }
                    } else {
                        // Cross-module drop: first move the video to this module.
                        let uc = ServiceFactory::move_video_to_module(ctx);
                        // Determine the target sort_order (before or after target).
                        let new_order = if insert_before {
                            target_sort_order
                        } else {
                            target_sort_order.saturating_add(1)
                        };
                        if let Err(e) = uc.execute(MoveVideoInput {
                            video_id: moved_id,
                            target_module_id: drop_module_id,
                            sort_order: new_order,
                        }) {
                            Toast::show_error(&format!("Failed to move: {}", e));
                            return false;
                        }
                    }
                }
                drop(s);
                drop_expanded.borrow_mut().insert(drop_module_id_str.clone());
                drop_cb();
                true
            });
            row.add_controller(row_dt);
        }

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
                if let Err(e) = uc.execute(UpdateModuleTitleInput {
                    module_id,
                    title: new_title.trim().to_string(),
                }) {
                    Toast::show_error(&format!("Failed to rename module: {}", e));
                }
            }
            drop(s);
            refresh_cb();
        }
        dialog_cl.close();
    });

    dialog.present(None::<&gtk::Window>);
}
