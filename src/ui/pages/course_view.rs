use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::application::ServiceFactory;
use crate::application::use_cases::{
    CreateModuleInput, DeleteModuleInput, MoveVideoInput, UpdateModuleTitleInput,
};
use crate::domain::value_objects::ModuleId;
use crate::ui::list_models::VideoRowObject;
use crate::ui::navigation::PAGE_VIDEO_PLAYER;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;
use crate::ui::types::RefreshCallback;
use adw::NavigationView;
use adw::prelude::*;

// ---------------------------------------------------------------------------
// Helpers: read typed data stored via `glib::ObjectExt::set_data` / `data`
// ---------------------------------------------------------------------------

/// Read a `String` stored on a GObject via [`glib::ObjectExt::set_data`].
///
/// # Safety
///
/// The key must have been set with `set_data::<String>` on the same object
/// and the object must still be alive with the same stored type.
unsafe fn obj_data_str(obj: &impl glib::object::ObjectExt, key: &'static str) -> String {
    unsafe { obj.data::<String>(key).map(|p| p.as_ref().clone()).unwrap_or_default() }
}

/// Read a `bool` stored on a GObject via [`glib::ObjectExt::set_data`].
///
/// # Safety
///
/// Same contract as [`obj_data_str`].
unsafe fn obj_data_bool(obj: &impl glib::object::ObjectExt, key: &'static str) -> bool {
    unsafe { obj.data::<bool>(key).map(|p| *p.as_ref()).unwrap_or(false) }
}

/// Read a `u32` stored on a GObject via [`glib::ObjectExt::set_data`].
///
/// # Safety
///
/// Same contract as [`obj_data_str`].
unsafe fn obj_data_u32(obj: &impl glib::object::ObjectExt, key: &'static str) -> u32 {
    unsafe { obj.data::<u32>(key).map(|p| *p.as_ref()).unwrap_or(0) }
}

// ---------------------------------------------------------------------------
// CourseViewPage
// ---------------------------------------------------------------------------

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
        // Clear previous content.
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

        let Some(ref ctx) = state.backend else {
            self.status_page.set_title("No Backend");
            self.status_page.set_description(Some("No backend connected."));
            self.status_page.set_visible(true);
            return;
        };

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

                // ----------------------------------------------------------------
                // Course header
                // ----------------------------------------------------------------
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

                // Delete course handler.
                {
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
                            Some(
                                "This will permanently delete the course and all associated \
                                  modules, videos, notes, and exams. This action cannot be undone.",
                            ),
                        );
                        dialog.add_response("cancel", "Cancel");
                        dialog.add_response("delete", "Delete");
                        dialog.set_response_appearance(
                            "delete",
                            adw::ResponseAppearance::Destructive,
                        );
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
                                        Toast::show_error(&format!(
                                            "Failed to delete course: {}",
                                            e
                                        ));
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
                }

                if let Some(desc) = course.description() {
                    let desc_label = gtk::Label::new(Some(desc));
                    desc_label.set_halign(gtk::Align::Start);
                    desc_label.set_wrap(true);
                    desc_label.add_css_class("subtitle");
                    header.append(&desc_label);
                }

                self.content_box.append(&header);

                // ----------------------------------------------------------------
                // New Module button
                // ----------------------------------------------------------------
                let new_mod_btn = gtk::Button::with_label("+ New Module");
                new_mod_btn.set_halign(gtk::Align::Start);
                new_mod_btn.add_css_class("flat");
                new_mod_btn.set_margin_start(8);
                new_mod_btn.set_margin_bottom(8);

                {
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
                }
                self.content_box.append(&new_mod_btn);

                // ----------------------------------------------------------------
                // Modules + Video ListStores
                // ----------------------------------------------------------------
                let modules = match ctx.module_repo.find_by_course(course.id()) {
                    Ok(m) => m,
                    Err(_) => {
                        let err_mod = gtk::Label::new(Some("Failed to load modules."));
                        err_mod.add_css_class("subtitle");
                        self.content_box.append(&err_mod);
                        return;
                    },
                };

                // LLM availability (stable for the whole refresh cycle).
                let has_llm = ServiceFactory::summarize_video(ctx).is_some();

                for module in &modules {
                    // --- Module header (title + edit + delete) ---
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

                    // Expand state persistence.
                    {
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
                    }

                    // Edit module handler.
                    {
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
                    }

                    // Delete module handler.
                    {
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

                                    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 12);
                                    vbox.set_margin_start(16);
                                    vbox.set_margin_end(16);
                                    vbox.set_margin_top(16);
                                    vbox.set_margin_bottom(16);

                                    let msg = gtk::Label::new(Some(&format!(
                                        "Module \"{}\" has videos. They will be deleted. \
                                         Delete anyway?",
                                        del_module_title
                                    )));
                                    msg.set_wrap(true);
                                    vbox.append(&msg);

                                    let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                                    btn_box.set_halign(gtk::Align::End);

                                    let cancel_btn = gtk::Button::with_label("Cancel");
                                    btn_box.append(&cancel_btn);

                                    let confirm_btn = gtk::Button::with_label("Delete");
                                    confirm_btn.add_css_class("destructive-action");
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
                                            && let Err(e) = ServiceFactory::delete_module(ctx2)
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
                                    if let Err(e) = ServiceFactory::delete_module(ctx).execute(
                                        DeleteModuleInput {
                                            module_id: del_module_id,
                                            force: false,
                                        },
                                    ) {
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
                    }

                    // --- Video ListStore + SignalListItemFactory + ListView ---
                    let videos = match ctx.video_repo.find_by_module(module.id()) {
                        Ok(v) => v,
                        Err(_) => {
                            let err_vid = gtk::Label::new(Some("Failed to load videos."));
                            err_vid.add_css_class("subtitle");
                            expander.set_child(Some(&err_vid));
                            self.content_box.append(&expander);
                            continue;
                        },
                    };

                    let store = gio::ListStore::new::<VideoRowObject>();
                    let factory = gtk::SignalListItemFactory::new();

                    // Clone data needed by factory closures.
                    let state_fc = self.state.clone();
                    let nav_fc = self.nav.clone();
                    let pages_fc: Rc<HashMap<&'static str, adw::NavigationPage>> =
                        self.nav_pages.borrow().clone();
                    let refresh_fc = refresh_cb.clone();
                    let expanded_fc = self.expanded_modules.clone();

                    // --- Factory: create the row widget tree once per ListItem ---
                    factory.connect_setup(move |_factory, list_item| {
                        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
                            log::warn!("connect_setup: failed to downcast to gtk::ListItem");
                            return;
                        };

                        let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
                        row.set_margin_top(2);
                        row.set_margin_bottom(2);

                        // Initialise data slots (overwritten in connect_bind).
                        // SAFETY: We store trivially-cloneable data on a GObject that
                        // outlives all signal handlers. The gst-pbutils re-export of
                        // `set_data` is marked unsafe but the operation is sound here.
                        unsafe {
                            row.set_data::<String>("video-id", String::new());
                            row.set_data::<String>("module-id", String::new());
                            row.set_data::<u32>("sort-order", 0u32);
                        }

                        // -- CheckButton (completion toggle) --
                        let check_btn = gtk::CheckButton::new();
                        check_btn.set_valign(gtk::Align::Center);
                        {
                            let row_ref = row.clone();
                            let state_h = state_fc.clone();
                            let refresh_h = refresh_fc.clone();
                            check_btn.connect_toggled(move |btn| {
                                // Skip toggles triggered by programmatic set_active in
                                // connect_bind.
                                let initializing = unsafe { obj_data_bool(btn, "init") };
                                if initializing {
                                    return;
                                }
                                let vid = unsafe { obj_data_str(&row_ref, "video-id") };
                                if vid.is_empty() {
                                    return;
                                }
                                if let Ok(video_id) =
                                    vid.parse::<crate::domain::value_objects::VideoId>()
                                {
                                    let s = state_h.borrow();
                                    if let Some(ref ctx) = s.backend {
                                        let _ = ctx
                                            .video_repo
                                            .update_completion(&video_id, btn.is_active());
                                    }
                                    drop(s);
                                    refresh_h();
                                }
                            });
                        }
                        row.append(&check_btn);

                        // -- Title label --
                        let title_label = gtk::Label::new(None);
                        title_label.set_halign(gtk::Align::Start);
                        title_label.set_hexpand(true);
                        title_label.set_ellipsize(gtk::pango::EllipsizeMode::End);
                        row.append(&title_label);

                        // -- Source badge ("YT" / "Local") --
                        let source_badge = gtk::Label::new(None);
                        source_badge.add_css_class("caption");
                        source_badge.add_css_class("tag");
                        source_badge.set_valign(gtk::Align::Center);
                        row.append(&source_badge);

                        // -- Duration label --
                        let dur_label = gtk::Label::new(None);
                        dur_label.add_css_class("subtitle");
                        dur_label.set_valign(gtk::Align::Center);
                        row.append(&dur_label);

                        // -- Play button --
                        let play_btn = gtk::Button::with_label("Play");
                        play_btn.set_valign(gtk::Align::Center);
                        play_btn.add_css_class("circular");
                        {
                            let row_ref = row.clone();
                            let state_h = state_fc.clone();
                            let nav_h = nav_fc.clone();
                            let pages_h = pages_fc.clone();
                            play_btn.connect_clicked(move |_| {
                                let vid = unsafe { obj_data_str(&row_ref, "video-id") };
                                if vid.is_empty() {
                                    return;
                                }
                                state_h.borrow_mut().current_video_id = Some(vid);
                                if let Some(page) = pages_h.get(PAGE_VIDEO_PLAYER) {
                                    nav_h.push(page);
                                }
                            });
                        }
                        row.append(&play_btn);

                        // -- Menu button (summarize / quiz) --
                        let menu_model = gio::Menu::new();
                        if has_llm {
                            let summarize_section = gio::Menu::new();
                            summarize_section.append(Some("Summarize"), Some("video.summarize"));
                            menu_model.append_section(None::<&str>, &summarize_section);

                            let quiz_section = gio::Menu::new();
                            quiz_section.append(Some("Generate Quiz"), Some("video.quiz"));
                            menu_model.append_section(None::<&str>, &quiz_section);
                        } else {
                            let no_llm_section = gio::Menu::new();
                            no_llm_section
                                .append(Some("Summarize (needs API key)"), Some("video.noop"));
                            no_llm_section
                                .append(Some("Generate Quiz (needs API key)"), Some("video.noop"));
                            menu_model.append_section(None::<&str>, &no_llm_section);
                        }

                        let popover = gtk::PopoverMenu::from_model(Some(&menu_model));
                        let action_group = gio::SimpleActionGroup::new();

                        // Summarize action.
                        {
                            let action = gio::SimpleAction::new("summarize", None);
                            let row_ref = row.clone();
                            let s = state_fc.clone();
                            action.connect_activate(move |_, _| {
                                let vid = unsafe { obj_data_str(&row_ref, "video-id") };
                                if vid.is_empty() {
                                    return;
                                }
                                let video_id =
                                    match vid.parse::<crate::domain::value_objects::VideoId>() {
                                        Ok(id) => id,
                                        Err(_) => return,
                                    };
                                s.borrow_mut().current_video_id = Some(vid);
                                Toast::show("Summarization started. Check the AI Chat panel.");
                                let sb = s.borrow();
                                if let Some(ref ctx) = sb.backend
                                    && let Some(uc) = ServiceFactory::summarize_video(ctx)
                                {
                                    let input =
                                        crate::application::use_cases::SummarizeVideoInput {
                                            video_id,
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

                        // Quiz action.
                        {
                            let action = gio::SimpleAction::new("quiz", None);
                            let row_ref = row.clone();
                            let s = state_fc.clone();
                            action.connect_activate(move |_, _| {
                                let vid = unsafe { obj_data_str(&row_ref, "video-id") };
                                if vid.is_empty() {
                                    return;
                                }
                                let video_id =
                                    match vid.parse::<crate::domain::value_objects::VideoId>() {
                                        Ok(id) => id,
                                        Err(_) => return,
                                    };
                                s.borrow_mut().current_video_id = Some(vid);
                                Toast::show("Quiz generation started. Check the AI Chat panel.");
                                let sb = s.borrow();
                                if let Some(ref ctx) = sb.backend
                                    && let Some(uc) = ServiceFactory::take_exam(ctx)
                                {
                                    use crate::domain::value_objects::ExamDifficulty;
                                    let input = crate::application::use_cases::GenerateExamInput {
                                        video_id,
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

                        // Noop action (disabled placeholder).
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

                        // -- DragSource (initiate drag from any row) --
                        {
                            let drag_source = gtk::DragSource::new();
                            drag_source.set_actions(gtk::gdk::DragAction::MOVE);
                            let row_ref = row.clone();
                            drag_source.connect_prepare(move |_, _, _| {
                                let vid = unsafe { obj_data_str(&row_ref, "video-id") };
                                let mid = unsafe { obj_data_str(&row_ref, "module-id") };
                                if vid.is_empty() {
                                    return None;
                                }
                                let payload = format!("{}:{}", vid, mid);
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
                            row.add_controller(drag_source);
                        }

                        // -- DropTarget (receive drop on this row for reorder) --
                        {
                            let row_dt = gtk::DropTarget::new(
                                glib::Type::STRING,
                                gtk::gdk::DragAction::MOVE,
                            );

                            // Visual hover feedback (drop-above / drop-below).
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

                            // Drop handler: reorder within module or move across.
                            let drop_state = state_fc.clone();
                            let drop_cb = refresh_fc.clone();
                            let drop_expanded = expanded_fc.clone();
                            let drop_row = row.clone();
                            row_dt.connect_drop(move |row_widget, value, _, y| {
                                // Clear visual indicator.
                                drop_row.remove_css_class("drop-above");
                                drop_row.remove_css_class("drop-below");

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

                                // Read target row's identity from its data slots.
                                let target_vid_str =
                                    unsafe { obj_data_str(row_widget, "video-id") };
                                let target_mod_str =
                                    unsafe { obj_data_str(row_widget, "module-id") };
                                let target_sort = unsafe { obj_data_u32(row_widget, "sort-order") };

                                let Ok(target_video_id) =
                                    target_vid_str.parse::<crate::domain::value_objects::VideoId>()
                                else {
                                    return false;
                                };

                                // Never drop a row onto itself.
                                if moved_id == target_video_id {
                                    return false;
                                }

                                let row_height = drop_row.height() as f64;
                                let half = if row_height > 0.0 { row_height / 2.0 } else { 18.0 };
                                let insert_before = y < half;

                                let is_same_module = src_mod_str == target_mod_str;

                                let s = drop_state.borrow();
                                if let Some(ref ctx) = s.backend {
                                    let target_module_id = match target_mod_str.parse::<ModuleId>()
                                    {
                                        Ok(id) => id,
                                        Err(_) => return false,
                                    };

                                    if is_same_module {
                                        // Within-module reorder: rebuild contiguous
                                        // sort_orders with the dragged item inserted
                                        // at the computed position.
                                        let existing = ctx
                                            .video_repo
                                            .find_by_module(&target_module_id)
                                            .unwrap_or_default();
                                        let mut positions: Vec<(
                                            crate::domain::value_objects::VideoId,
                                            u32,
                                        )> = existing
                                            .iter()
                                            .filter(|v| v.id() != &moved_id)
                                            .map(|v| (*v.id(), v.sort_order()))
                                            .collect();

                                        let target_pos = positions
                                            .iter()
                                            .position(|(id, _)| id == &target_video_id);
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
                                        positions.insert(insert_at, (moved_id, 0));

                                        let mut had_error = false;
                                        for (new_order, (vid_id, old_order)) in
                                            positions.iter().enumerate()
                                        {
                                            let new_order = new_order as u32;
                                            if (new_order != *old_order || vid_id == &moved_id)
                                                && let Err(e) = ctx.video_repo.update_module(
                                                    vid_id,
                                                    &target_module_id,
                                                    new_order,
                                                )
                                            {
                                                Toast::show_error(&format!(
                                                    "Failed to reorder: {}",
                                                    e
                                                ));
                                                had_error = true;
                                                break;
                                            }
                                        }
                                        if had_error {
                                            return false;
                                        }
                                    } else {
                                        // Cross-module move.
                                        let uc = ServiceFactory::move_video_to_module(ctx);
                                        let new_order = if insert_before {
                                            target_sort
                                        } else {
                                            target_sort.saturating_add(1)
                                        };
                                        if let Err(e) = uc.execute(MoveVideoInput {
                                            video_id: moved_id,
                                            target_module_id,
                                            sort_order: new_order,
                                        }) {
                                            Toast::show_error(&format!("Failed to move: {}", e));
                                            return false;
                                        }
                                    }
                                }
                                drop(s);
                                drop_expanded.borrow_mut().insert(target_mod_str);
                                drop_cb();
                                true
                            });
                            row.add_controller(row_dt);
                        }

                        list_item.set_child(Some(&row));
                    });

                    // --- Factory: bind a VideoRowObject to the row widgets ---
                    factory.connect_bind(move |_factory, list_item| {
                        let Some(list_item) = list_item.downcast_ref::<gtk::ListItem>() else {
                            return;
                        };
                        let Some(item) =
                            list_item.item().and_then(|i| i.downcast::<VideoRowObject>().ok())
                        else {
                            return;
                        };
                        let Some(row) =
                            list_item.child().and_then(|c| c.downcast::<gtk::Box>().ok())
                        else {
                            return;
                        };

                        // Store identity data on the row for signal handlers / DnD.
                        // SAFETY: Same as the initialisation in connect_setup.
                        unsafe {
                            row.set_data::<String>("video-id", item.id());
                            row.set_data::<String>("module-id", item.module_id());
                            row.set_data::<u32>("sort-order", item.sort_order());
                        }

                        // Navigate the widget tree created in connect_setup.
                        // Layout: [CheckButton, Label, Label, Label, Button, MenuButton]
                        let Some(check_btn) =
                            row.first_child().and_then(|c| c.downcast::<gtk::CheckButton>().ok())
                        else {
                            return;
                        };
                        let Some(title_label) =
                            check_btn.next_sibling().and_then(|c| c.downcast::<gtk::Label>().ok())
                        else {
                            return;
                        };
                        let Some(source_badge) = title_label
                            .next_sibling()
                            .and_then(|c| c.downcast::<gtk::Label>().ok())
                        else {
                            return;
                        };
                        let Some(dur_label) = source_badge
                            .next_sibling()
                            .and_then(|c| c.downcast::<gtk::Label>().ok())
                        else {
                            return;
                        };

                        // Update CheckButton without firing the toggled handler.
                        // SAFETY: Storing a bool flag read by the toggled handler;
                        // the CheckButton outlives the handler.
                        unsafe {
                            check_btn.set_data::<bool>("init", true);
                        }
                        check_btn.set_active(item.is_completed());
                        unsafe {
                            check_btn.set_data::<bool>("init", false);
                        }

                        // Title.
                        let t = item.title();
                        title_label.set_text(&t);
                        title_label.set_tooltip_text(Some(&t));

                        // Source badge.
                        source_badge.set_text(match item.source_type().as_str() {
                            "YouTube" => "YT",
                            _ => "Local",
                        });

                        // Duration (H:MM:SS or MM:SS).
                        let total_secs = item.duration_secs();
                        let hours = total_secs / 3600;
                        let mins = (total_secs % 3600) / 60;
                        let secs = total_secs % 60;
                        let formatted_dur = if hours > 0 {
                            format!("{}:{:02}:{:02}", hours, mins, secs)
                        } else {
                            format!("{:02}:{:02}", mins, secs)
                        };
                        dur_label.set_text(&formatted_dur);
                    });

                    // Assemble the ListView.
                    let no_selection = gtk::NoSelection::new(Some(store.clone()));
                    let list_view = gtk::ListView::new(Some(no_selection), Some(factory));
                    list_view.set_margin_start(12);
                    list_view.set_margin_top(4);
                    list_view.set_margin_bottom(4);
                    list_view.set_vexpand(false);
                    list_view.set_single_click_activate(false);

                    // Populate the store with VideoRowObjects.
                    for video in &videos {
                        let source_type = match video.source() {
                            crate::domain::value_objects::VideoSource::YouTube(_) => "YouTube",
                            crate::domain::value_objects::VideoSource::LocalPath(_) => "Local",
                        };
                        let obj = VideoRowObject::new(
                            video.id().to_string(),
                            module.id().as_uuid().to_string(),
                            video.title().to_string(),
                            video.is_completed(),
                            video.duration_secs(),
                            source_type.to_string(),
                            video.sort_order(),
                        );
                        store.append(&obj);
                    }

                    // Container wrapping the ListView + fallback drop zone.
                    let video_container = gtk::Box::new(gtk::Orientation::Vertical, 0);
                    video_container.append(&list_view);

                    // Fallback drop zone (catches drops that miss per-row targets,
                    // e.g. on an empty module or past the last row).
                    {
                        let fb_dt =
                            gtk::DropTarget::new(glib::Type::STRING, gtk::gdk::DragAction::MOVE);
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
                                    // Within-module: move to end.
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
                                    // Cross-module: move to end of target.
                                    let uc = ServiceFactory::move_video_to_module(ctx);
                                    if let Err(e) = uc.execute(MoveVideoInput {
                                        video_id: moved_id,
                                        target_module_id: fb_module_id,
                                        sort_order: 0,
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
                        video_container.add_controller(fb_dt);
                    }

                    expander.set_child(Some(&video_container));
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
    }
}

// ---------------------------------------------------------------------------
// Module rename dialog
// ---------------------------------------------------------------------------

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
