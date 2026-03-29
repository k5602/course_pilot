use std::cell::RefCell;
use std::rc::Rc;

use gtk::prelude::*;

use crate::application::ServiceFactory;
use crate::application::use_cases::UpdatePresenceInput;
use crate::domain::entities::SearchResultType;
use crate::domain::ports::Activity;
use crate::domain::ports::SearchRepository;
use crate::ui::dialogs;
use crate::ui::navigation::{
    PAGE_COURSE_LIST, PAGE_COURSE_VIEW, PAGE_DASHBOARD, PAGE_QUIZ_LIST, PAGE_QUIZ_VIEW,
    PAGE_SETTINGS, PAGE_VIDEO_PLAYER,
};
use crate::ui::pages;
use crate::ui::right_panel;
use crate::ui::shortcuts::setup_shortcuts;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;

fn build_sidebar(
    state: SharedState,
    stack: Rc<gtk::Stack>,
    _right_panel_widget: &gtk::Box,
    parent_window: Option<gtk::Window>,
    overlay: &adw::OverlaySplitView,
    on_import_refresh: Rc<dyn Fn()>,
) -> gtk::Box {
    let sidebar = gtk::Box::new(gtk::Orientation::Vertical, 4);
    sidebar.set_width_request(220);
    sidebar.add_css_class("sidebar");

    let title = gtk::Label::new(Some("Course Pilot"));
    title.add_css_class("heading");
    title.set_margin_start(12);
    title.set_margin_end(12);
    title.set_margin_top(16);
    title.set_margin_bottom(8);
    sidebar.append(&title);

    let search = gtk::SearchEntry::new();
    search.set_placeholder_text(Some("Search courses..."));
    search.set_margin_start(8);
    search.set_margin_end(8);
    search.set_margin_bottom(8);
    sidebar.append(&search);

    let search_popover = gtk::Popover::new();
    search_popover.set_parent(&search);
    search_popover.set_position(gtk::PositionType::Bottom);
    search_popover.set_width_request(280);

    let state_se = state.clone();
    let stack_se = stack.clone();
    search.connect_search_changed(move |entry| {
        let query = entry.text().to_string();
        if query.trim().is_empty() {
            search_popover.popdown();
            return;
        }

        let results: Vec<crate::domain::entities::SearchResult> = {
            let s = state_se.borrow();
            s.backend
                .as_ref()
                .and_then(|ctx| ctx.search_repo.search(&query, 20).ok())
                .unwrap_or_default()
        };

        if results.is_empty() {
            search_popover.popdown();
            return;
        }

        let box_ = gtk::Box::new(gtk::Orientation::Vertical, 4);
        box_.set_margin_start(8);
        box_.set_margin_end(8);
        box_.set_margin_top(8);
        box_.set_margin_bottom(8);

        for result in &results {
            let row = gtk::Box::new(gtk::Orientation::Vertical, 2);

            let type_label = gtk::Label::new(Some(&result.entity_type.to_string()));
            type_label.add_css_class("caption");
            type_label.set_halign(gtk::Align::Start);

            let title_label = gtk::Label::new(Some(&result.title));
            title_label.set_halign(gtk::Align::Start);
            title_label.set_wrap(true);
            title_label.set_max_width_chars(40);

            row.append(&type_label);
            row.append(&title_label);

            let r = result.clone();
            let s_cl = state_se.clone();
            let st_cl = stack_se.clone();
            let popover_cl = search_popover.clone();
            let gesture = gtk::GestureClick::new();
            gesture.connect_released(move |_, _, _, _| {
                popover_cl.popdown();
                let mut s = s_cl.borrow_mut();
                match r.entity_type {
                    SearchResultType::Course => {
                        s.current_course_id = Some(r.course_id.as_uuid().to_string());
                        st_cl.set_visible_child_name(PAGE_COURSE_VIEW);
                    },
                    SearchResultType::Video => {
                        s.current_course_id = Some(r.course_id.as_uuid().to_string());
                        s.current_video_id = Some(r.entity_id.clone());
                        st_cl.set_visible_child_name(PAGE_VIDEO_PLAYER);
                    },
                    SearchResultType::Note => {
                        s.current_course_id = Some(r.course_id.as_uuid().to_string());
                        st_cl.set_visible_child_name(PAGE_COURSE_VIEW);
                    },
                }
            });
            row.add_controller(gesture);

            let separator = gtk::Separator::new(gtk::Orientation::Horizontal);
            box_.append(&row);
            box_.append(&separator);
        }

        search_popover.set_child(Some(&box_));
        if !search_popover.is_visible() {
            search_popover.popup();
        }
    });

    let nav_box = gtk::Box::new(gtk::Orientation::Vertical, 2);
    nav_box.set_vexpand(true);
    nav_box.set_margin_start(8);
    nav_box.set_margin_end(8);

    let nav_items: [(&str, &str); 4] = [
        (PAGE_DASHBOARD, "Dashboard"),
        (PAGE_COURSE_LIST, "Courses"),
        (PAGE_QUIZ_LIST, "Quizzes"),
        (PAGE_SETTINGS, "Settings"),
    ];

    let buttons: Rc<RefCell<Vec<gtk::ToggleButton>>> = Rc::new(RefCell::new(Vec::new()));

    for (page_id, label) in &nav_items {
        let btn = gtk::ToggleButton::new();
        btn.add_css_class("flat");
        btn.set_halign(gtk::Align::Fill);
        let btn_label = gtk::Label::new(Some(label));
        btn_label.set_halign(gtk::Align::Start);
        btn.set_child(Some(&btn_label));

        let buttons_clone = buttons.clone();
        let stack_clone = stack.clone();
        let nav_page_id = page_id.to_string();
        btn.connect_toggled(move |btn| {
            if btn.is_active() {
                for other in buttons_clone.borrow().iter() {
                    if other != btn {
                        other.set_active(false);
                    }
                }
                stack_clone.set_visible_child_name(&nav_page_id);
            } else {
                let has_active = buttons_clone.borrow().iter().any(|b| b.is_active());
                if !has_active {
                    btn.set_active(true);
                }
            }
        });
        buttons.borrow_mut().push(btn.clone());
        nav_box.append(&btn);
    }

    sidebar.append(&nav_box);

    let import_section = gtk::Box::new(gtk::Orientation::Vertical, 2);
    import_section.set_margin_start(8);
    import_section.set_margin_end(8);
    import_section.set_margin_bottom(8);

    let pw_yt = parent_window.clone();
    let state_cl_yt = state.clone();
    let stack_cl_yt = stack.clone();
    let refresh_yt = on_import_refresh.clone();
    let yt_btn = gtk::Button::with_label("Import YouTube Playlist");
    yt_btn.add_css_class("flat");
    yt_btn.set_halign(gtk::Align::Fill);
    yt_btn.connect_clicked(move |_| {
        dialogs::import_dialog::show_import_playlist_dialog(
            state_cl_yt.clone(),
            stack_cl_yt.clone(),
            pw_yt.as_ref(),
            Some(refresh_yt.clone()),
        );
    });
    import_section.append(&yt_btn);

    let pw_local = parent_window.clone();
    let state_cl_local = state.clone();
    let stack_cl_local = stack.clone();
    let refresh_local = on_import_refresh;
    let local_btn = gtk::Button::with_label("Import Local Media");
    local_btn.add_css_class("flat");
    local_btn.set_halign(gtk::Align::Fill);
    local_btn.connect_clicked(move |_| {
        dialogs::import_local_dialog::show_import_local_dialog(
            state_cl_local.clone(),
            stack_cl_local.clone(),
            pw_local.as_ref(),
            Some(refresh_local.clone()),
        );
    });
    import_section.append(&local_btn);

    sidebar.append(&import_section);

    let notes_toggle = gtk::ToggleButton::new();
    notes_toggle.add_css_class("flat");
    notes_toggle.set_halign(gtk::Align::Fill);
    let toggle_label = gtk::Label::new(Some("Notes"));
    toggle_label.set_halign(gtk::Align::Start);
    notes_toggle.set_child(Some(&toggle_label));
    notes_toggle.set_active({
        let s = state.borrow();
        s.right_panel_visible
    });

    let state_clone = state.clone();
    let overlay_clone = overlay.clone();
    notes_toggle.connect_toggled(move |btn| {
        let visible = btn.is_active();
        {
            let mut s = state_clone.borrow_mut();
            s.right_panel_visible = visible;
        }
        overlay_clone.set_show_sidebar(visible);
    });

    sidebar.append(&notes_toggle);

    sidebar
}

pub struct MainLayout;

impl MainLayout {
    pub fn build(state: SharedState, parent_window: &gtk::Window) -> adw::ToastOverlay {
        let navigation_stack = gtk::Stack::new();
        navigation_stack.set_hexpand(true);
        navigation_stack.set_vexpand(true);

        let nav_stack_rc = Rc::new(navigation_stack);

        let dashboard =
            Rc::new(pages::dashboard::DashboardPage::new(state.clone(), nav_stack_rc.clone()));
        nav_stack_rc.add_named(dashboard.widget(), Some(PAGE_DASHBOARD));

        let course_list =
            Rc::new(pages::course_list::CourseListPage::new(state.clone(), nav_stack_rc.clone()));
        nav_stack_rc.add_named(course_list.widget(), Some(PAGE_COURSE_LIST));

        let course_view =
            Rc::new(pages::course_view::CourseViewPage::new(state.clone(), nav_stack_rc.clone()));
        {
            let weak = Rc::downgrade(&course_view);
            course_view.set_refresh_cb(Rc::new(move || {
                if let Some(cv) = weak.upgrade() {
                    cv.refresh();
                }
            }));
        }
        nav_stack_rc.add_named(course_view.widget(), Some(PAGE_COURSE_VIEW));

        let quiz_list =
            Rc::new(pages::quiz_list::QuizListPage::new(state.clone(), nav_stack_rc.clone()));
        nav_stack_rc.add_named(quiz_list.widget(), Some(PAGE_QUIZ_LIST));

        let quiz_view =
            Rc::new(pages::quiz_view::QuizViewPage::new(state.clone(), nav_stack_rc.clone()));
        nav_stack_rc.add_named(quiz_view.widget(), Some(PAGE_QUIZ_VIEW));

        let settings =
            Rc::new(pages::settings::SettingsPage::new(state.clone(), nav_stack_rc.clone()));
        nav_stack_rc.add_named(settings.widget(), Some(PAGE_SETTINGS));

        let video_player =
            Rc::new(pages::video_player::VideoPlayerPage::new(state.clone(), nav_stack_rc.clone()));
        nav_stack_rc.add_named(video_player.widget(), Some(PAGE_VIDEO_PLAYER));

        nav_stack_rc.set_visible_child_name(PAGE_DASHBOARD);

        let right_panel = Rc::new(right_panel::RightPanel::new(state.clone()));

        let outer_split = adw::OverlaySplitView::new();
        outer_split.set_sidebar(Some(right_panel.widget()));
        outer_split.set_sidebar_position(gtk::PackType::End);
        outer_split.set_show_sidebar({
            let s = state.borrow();
            s.right_panel_visible
        });

        let on_import_refresh = {
            let cl = course_list.clone();
            Rc::new(move || cl.refresh())
        };

        let sidebar_box = build_sidebar(
            state.clone(),
            nav_stack_rc.clone(),
            right_panel.widget(),
            Some(parent_window.clone()),
            &outer_split,
            on_import_refresh,
        );

        let sidebar_page = adw::NavigationPage::new(&sidebar_box, "Sidebar");
        let content_page = adw::NavigationPage::new(&*nav_stack_rc, "Content");
        let inner_split = adw::NavigationSplitView::new();
        inner_split.set_sidebar(Some(&sidebar_page));
        inner_split.set_content(Some(&content_page));
        inner_split.set_show_content(true);
        inner_split.set_collapsed(false);
        inner_split.set_max_sidebar_width(220.0);
        inner_split.set_min_sidebar_width(180.0);
        outer_split.set_content(Some(&inner_split));

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&outer_split));
        Toast::init(&toast_overlay);

        if let Some(ref ctx) = state.borrow().backend {
            let presence = ServiceFactory::update_presence(ctx);
            presence.execute(UpdatePresenceInput { activity: Activity::Dashboard });
        }

        let db = dashboard.clone();
        let cl = course_list.clone();
        let cv = course_view.clone();
        let vp = video_player.clone();
        let ql = quiz_list.clone();
        let qv = quiz_view.clone();
        let st = settings.clone();
        let rp = right_panel.clone();
        let old_page: Rc<RefCell<String>> = Rc::new(RefCell::new(PAGE_DASHBOARD.to_string()));
        let nav_state = state.clone();

        nav_stack_rc.connect_visible_child_name_notify(move |stack| {
            let new_name = stack.visible_child_name().unwrap_or_default().to_string();
            let old = old_page.borrow().clone();

            if old == PAGE_VIDEO_PLAYER && new_name != PAGE_VIDEO_PLAYER {
                vp.stop();
            }

            *old_page.borrow_mut() = new_name.clone();

            let s = nav_state.borrow();
            if let Some(ref ctx) = s.backend {
                let presence = ServiceFactory::update_presence(ctx);
                let activity = match new_name.as_str() {
                    PAGE_DASHBOARD => Activity::Dashboard,
                    PAGE_COURSE_LIST => Activity::BrowsingCourses,
                    PAGE_COURSE_VIEW => Activity::BrowsingCourses,
                    PAGE_VIDEO_PLAYER => Activity::Watching {
                        course_title: String::new(),
                        video_title: String::new(),
                    },
                    PAGE_QUIZ_LIST => Activity::Idle,
                    PAGE_QUIZ_VIEW => Activity::TakingExam {
                        course_title: String::new(),
                        exam_title: String::new(),
                    },
                    PAGE_SETTINGS => Activity::Settings,
                    _ => Activity::Idle,
                };
                presence.execute(UpdatePresenceInput { activity });
            }
            drop(s);

            match new_name.as_str() {
                PAGE_DASHBOARD => db.refresh(),
                PAGE_COURSE_LIST => cl.refresh(),
                PAGE_COURSE_VIEW => cv.refresh(),
                PAGE_VIDEO_PLAYER => {
                    vp.refresh();
                    rp.refresh();
                },
                PAGE_QUIZ_LIST => ql.refresh(),
                PAGE_QUIZ_VIEW => qv.refresh(),
                PAGE_SETTINGS => st.refresh(),
                _ => {},
            }
        });

        setup_shortcuts(parent_window, nav_stack_rc.clone());

        toast_overlay
    }
}
