use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use gtk::prelude::*;

use adw::NavigationView;
use adw::prelude::{AdwApplicationWindowExt, BreakpointBinExt, NavigationPageExt};

use crate::application::ServiceFactory;
use crate::application::use_cases::UpdatePresenceInput;
use crate::domain::entities::SearchResultType;
use std::str::FromStr;

use crate::domain::ports::Activity;
use crate::ui::dialogs;
use crate::ui::navigation::{
    PAGE_COURSE_LIST, PAGE_COURSE_VIEW, PAGE_DASHBOARD, PAGE_QUIZ_LIST, PAGE_QUIZ_VIEW,
    PAGE_SETTINGS, PAGE_VIDEO_PLAYER,
};
use crate::ui::pages;
use crate::ui::right_panel;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;
use crate::ui::types::RefreshCallback;

fn wrap_page(
    widget: &impl IsA<gtk::Widget>,
    title: &str,
    tag: &'static str,
    state: SharedState,
    parent_window: adw::ApplicationWindow,
    import_refresh_cb: RefreshCallback,
) -> adw::NavigationPage {
    let toolbar = adw::ToolbarView::new();
    let header = adw::HeaderBar::new();

    // Create a beautiful "+" MenuButton in the header bar for direct ingest options
    let plus_btn = gtk::MenuButton::new();
    plus_btn.set_icon_name("list-add-symbolic");
    plus_btn.add_css_class("flat");
    plus_btn.set_tooltip_text(Some("Import Course"));

    let popover = gtk::Popover::new();
    let popover_box = gtk::Box::new(gtk::Orientation::Vertical, 6);
    popover_box.set_margin_start(8);
    popover_box.set_margin_end(8);
    popover_box.set_margin_top(8);
    popover_box.set_margin_bottom(8);

    let yt_btn = gtk::Button::with_label("Import YouTube Playlist");
    yt_btn.set_icon_name("network-wired-symbolic");
    yt_btn.add_css_class("flat");
    yt_btn.set_halign(gtk::Align::Fill);

    let state_yt = state.clone();
    let pw_yt = parent_window.clone();
    let refresh_yt = import_refresh_cb.clone();
    let popover_yt = popover.clone();
    yt_btn.connect_clicked(move |_| {
        popover_yt.popdown();
        let cb = refresh_yt.borrow().clone();
        dialogs::import_dialog::show_import_playlist_dialog(
            state_yt.clone(),
            Some(pw_yt.upcast_ref::<gtk::Window>()),
            cb,
        );
    });

    let local_btn = gtk::Button::with_label("Import Local Folder");
    local_btn.set_icon_name("folder-open-symbolic");
    local_btn.add_css_class("flat");
    local_btn.set_halign(gtk::Align::Fill);

    let state_local = state;
    let pw_local = parent_window;
    let refresh_local = import_refresh_cb.clone();
    let popover_local = popover.clone();
    local_btn.connect_clicked(move |_| {
        popover_local.popdown();
        let cb = refresh_local.borrow().clone();
        dialogs::import_local_dialog::show_import_local_dialog(
            state_local.clone(),
            Some(pw_local.upcast_ref::<gtk::Window>()),
            cb,
        );
    });

    popover_box.append(&yt_btn);
    popover_box.append(&local_btn);
    popover.set_child(Some(&popover_box));
    plus_btn.set_popover(Some(&popover));

    header.pack_end(&plus_btn);

    toolbar.add_top_bar(&header);
    toolbar.set_content(Some(widget));
    let page = adw::NavigationPage::new(&toolbar, title);
    page.set_tag(Some(tag));
    page
}

#[allow(clippy::too_many_arguments)]
fn build_sidebar(
    state: SharedState,
    nav_view: Rc<NavigationView>,
    nav_pages: Rc<HashMap<&'static str, adw::NavigationPage>>,
    tag_to_button: Rc<RefCell<HashMap<&'static str, gtk::ToggleButton>>>,
    _right_panel_widget: &gtk::Box,
    _parent_window: Option<adw::ApplicationWindow>,
    overlay: &adw::OverlaySplitView,
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

    let debounce_source: Rc<RefCell<Option<glib::SourceId>>> = Rc::new(RefCell::new(None));
    let state_se = state.clone();
    let nav_se = nav_view.clone();
    let pages_se = nav_pages.clone();
    search.connect_search_changed(move |entry| {
        if let Some(id) = debounce_source.borrow_mut().take() {
            id.remove();
        }
        let query = entry.text().to_string();
        if query.trim().is_empty() {
            search_popover.popdown();
            return;
        }
        let state_se_cl = state_se.clone();
        let nav_se_cl = nav_se.clone();
        let pages_se_cl = pages_se.clone();
        let search_popover_cl = search_popover.clone();
        let debounce = debounce_source.clone();
        let id = glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
            perform_search(&state_se_cl, &nav_se_cl, &pages_se_cl, &search_popover_cl, &query);
            *debounce.borrow_mut() = None;
            glib::ControlFlow::Break
        });
        *debounce_source.borrow_mut() = Some(id);
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
        let tag_map = tag_to_button.clone();
        let nav_cl = nav_view.clone();
        let pages_cl = nav_pages.clone();
        let nav_page_id = page_id.to_string();
        btn.connect_toggled(move |btn| {
            if btn.is_active() {
                for other in buttons_clone.borrow().iter() {
                    if other != btn {
                        other.set_active(false);
                    }
                }
                if nav_page_id == PAGE_DASHBOARD {
                    nav_cl.pop_to_tag(PAGE_DASHBOARD);
                } else if let Some(target) = pages_cl.get(nav_page_id.as_str()) {
                    nav_cl.pop_to_tag(PAGE_DASHBOARD);
                    nav_cl.push(target);
                }
            } else {
                let has_active = buttons_clone.borrow().iter().any(|b| b.is_active());
                if !has_active {
                    btn.set_active(true);
                }
            }
        });
        tag_map.borrow_mut().insert(page_id, btn.clone());
        buttons.borrow_mut().push(btn.clone());
        nav_box.append(&btn);
    }

    sidebar.append(&nav_box);

    let chat_toggle = gtk::ToggleButton::new();
    chat_toggle.add_css_class("flat");
    chat_toggle.set_halign(gtk::Align::Fill);
    let toggle_label = gtk::Label::new(Some("AI Chat"));
    toggle_label.set_halign(gtk::Align::Start);
    chat_toggle.set_child(Some(&toggle_label));
    chat_toggle.set_tooltip_text(Some("Toggle AI Chat companion panel"));
    chat_toggle.set_active({
        let s = state.borrow();
        s.right_panel_visible
    });

    let state_clone = state.clone();
    let overlay_clone = overlay.clone();
    chat_toggle.connect_toggled(move |btn| {
        let visible = btn.is_active();
        {
            let mut s = state_clone.borrow_mut();
            s.right_panel_visible = visible;
        }
        overlay_clone.set_show_sidebar(visible);
    });

    sidebar.append(&chat_toggle);

    // Dedicated Course Notes Popup editor button
    let notes_btn = gtk::Button::new();
    notes_btn.add_css_class("flat");
    notes_btn.set_halign(gtk::Align::Fill);
    let notes_lbl = gtk::Label::new(Some("Course Notes (Ctrl+N)"));
    notes_lbl.set_halign(gtk::Align::Start);
    notes_btn.set_child(Some(&notes_lbl));
    notes_btn.set_tooltip_text(Some("Open dynamic floating Notes editor"));

    let state_notes = state;
    notes_btn.connect_clicked(move |_| {
        crate::ui::notes_window::open_notes_window(state_notes.clone());
    });

    sidebar.append(&notes_btn);

    sidebar
}

pub struct MainLayout;

impl MainLayout {
    pub fn build(state: SharedState, parent_window: &adw::ApplicationWindow) -> adw::ToastOverlay {
        let key_controller = gtk::EventControllerKey::new();
        let state_key = state.clone();
        key_controller.connect_key_pressed(move |_, keyval, _code, state_flag| {
            if state_flag.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                && (keyval == gtk::gdk::Key::n || keyval == gtk::gdk::Key::N)
            {
                crate::ui::notes_window::open_notes_window(state_key.clone());
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        parent_window.add_controller(key_controller);

        let nav_view = adw::NavigationView::new();
        nav_view.set_hexpand(true);
        nav_view.set_vexpand(true);
        nav_view.set_pop_on_escape(true);

        let nav_view_rc = Rc::new(nav_view);

        let import_refresh_cb: RefreshCallback = Rc::new(RefCell::new(None));

        // Step 1: Create all page widgets first (they don't need nav_pages yet)
        let dashboard =
            Rc::new(pages::dashboard::DashboardPage::new(state.clone(), nav_view_rc.clone()));
        let video_player = Rc::new(pages::video_player::VideoPlayerPage::new(state.clone()));
        let quiz_view =
            Rc::new(pages::quiz_view::QuizViewPage::new(state.clone(), nav_view_rc.clone()));
        let settings =
            Rc::new(pages::settings::SettingsPage::new(state.clone(), nav_view_rc.clone()));

        // Step 2: Create NavigationPages from widgets of pages that don't depend on nav_pages
        let dash_nav = Rc::new(wrap_page(
            dashboard.widget(),
            "Dashboard",
            PAGE_DASHBOARD,
            state.clone(),
            parent_window.clone(),
            import_refresh_cb.clone(),
        ));
        let vp_nav = Rc::new(wrap_page(
            video_player.widget(),
            "Video Player",
            PAGE_VIDEO_PLAYER,
            state.clone(),
            parent_window.clone(),
            import_refresh_cb.clone(),
        ));
        let qv_nav = Rc::new(wrap_page(
            quiz_view.widget(),
            "Quiz",
            PAGE_QUIZ_VIEW,
            state.clone(),
            parent_window.clone(),
            import_refresh_cb.clone(),
        ));
        let st_nav = Rc::new(wrap_page(
            settings.widget(),
            "Settings",
            PAGE_SETTINGS,
            state.clone(),
            parent_window.clone(),
            import_refresh_cb.clone(),
        ));

        // Step 3: Create pages that need nav_pages (constructor accepts empty placeholder)
        let course_list =
            Rc::new(pages::course_list::CourseListPage::new(state.clone(), nav_view_rc.clone()));
        let course_view =
            Rc::new(pages::course_view::CourseViewPage::new(state.clone(), nav_view_rc.clone()));
        let quiz_list =
            Rc::new(pages::quiz_list::QuizListPage::new(state.clone(), nav_view_rc.clone()));

        // Step 4: Create their NavigationPages
        let cl_nav = Rc::new(wrap_page(
            course_list.widget(),
            "Courses",
            PAGE_COURSE_LIST,
            state.clone(),
            parent_window.clone(),
            import_refresh_cb.clone(),
        ));
        let cv_nav = Rc::new(wrap_page(
            course_view.widget(),
            "Course",
            PAGE_COURSE_VIEW,
            state.clone(),
            parent_window.clone(),
            import_refresh_cb.clone(),
        ));
        let ql_nav = Rc::new(wrap_page(
            quiz_list.widget(),
            "Quizzes",
            PAGE_QUIZ_LIST,
            state.clone(),
            parent_window.clone(),
            import_refresh_cb.clone(),
        ));

        // Step 5: Build the full nav_pages map and inject into pages that need it
        let mut pages_map: HashMap<&'static str, adw::NavigationPage> = HashMap::new();
        pages_map.insert(PAGE_DASHBOARD, dash_nav.as_ref().clone());
        pages_map.insert(PAGE_COURSE_LIST, cl_nav.as_ref().clone());
        pages_map.insert(PAGE_COURSE_VIEW, cv_nav.as_ref().clone());
        pages_map.insert(PAGE_QUIZ_LIST, ql_nav.as_ref().clone());
        pages_map.insert(PAGE_QUIZ_VIEW, qv_nav.as_ref().clone());
        pages_map.insert(PAGE_SETTINGS, st_nav.as_ref().clone());
        pages_map.insert(PAGE_VIDEO_PLAYER, vp_nav.as_ref().clone());
        let nav_pages = Rc::new(pages_map);

        // Inject nav_pages into pages that need it
        dashboard.set_nav_pages(nav_pages.clone());
        course_list.set_nav_pages(nav_pages.clone());
        course_view.set_nav_pages(nav_pages.clone());
        quiz_list.set_nav_pages(nav_pages.clone());
        video_player.set_nav_pages(nav_pages.clone());

        // Step 6: Initialize nav view with dashboard as the root
        nav_view_rc.push(dash_nav.as_ref());

        // Step 7: Set up course_view refresh callback
        {
            let weak = Rc::downgrade(&course_view);
            course_view.set_refresh_cb(Rc::new(move || {
                if let Some(cv) = weak.upgrade() {
                    cv.refresh();
                }
            }));
        }

        let right_panel = Rc::new(right_panel::RightPanel::new(state.clone()));

        let outer_split = adw::OverlaySplitView::new();
        outer_split.set_sidebar(Some(right_panel.widget()));
        outer_split.set_sidebar_position(gtk::PackType::End);
        outer_split.set_show_sidebar({
            let s = state.borrow();
            s.right_panel_visible
        });

        let tag_to_button: Rc<RefCell<HashMap<&'static str, gtk::ToggleButton>>> =
            Rc::new(RefCell::new(HashMap::new()));

        let on_import_refresh = {
            let nav = nav_view_rc.clone();
            let cl_page = cl_nav;
            let cl = course_list.clone();
            Rc::new(move || {
                nav.pop_to_tag(PAGE_DASHBOARD);
                nav.push(cl_page.as_ref());
                cl.refresh();
            })
        };
        *import_refresh_cb.borrow_mut() = Some(on_import_refresh);

        let sidebar_box = build_sidebar(
            state.clone(),
            nav_view_rc.clone(),
            nav_pages.clone(),
            tag_to_button.clone(),
            right_panel.widget(),
            Some(parent_window.clone()),
            &outer_split,
        );

        let sidebar_nav_page = adw::NavigationPage::new(&sidebar_box, "Sidebar");
        let content_nav_page = adw::NavigationPage::new(nav_view_rc.as_ref(), "Content");
        let inner_split = adw::NavigationSplitView::new();
        inner_split.set_sidebar(Some(&sidebar_nav_page));
        inner_split.set_content(Some(&content_nav_page));
        inner_split.set_show_content(true);
        inner_split.set_collapsed(false);
        inner_split.set_max_sidebar_width(220.0);
        inner_split.set_min_sidebar_width(180.0);

        let breakpoint_bin = adw::BreakpointBin::new();
        breakpoint_bin.set_width_request(200);
        breakpoint_bin.set_height_request(200);
        breakpoint_bin.set_child(Some(&inner_split));
        {
            let condition = adw::BreakpointCondition::new_length(
                adw::BreakpointConditionLengthType::MaxWidth,
                600.0,
                adw::LengthUnit::Sp,
            );
            let bp = adw::Breakpoint::new(condition);
            let val: glib::Value = true.into();
            bp.add_setter(inner_split.upcast_ref::<glib::Object>(), "collapsed", Some(&val));
            breakpoint_bin.add_breakpoint(bp);
        }
        outer_split.set_content(Some(&breakpoint_bin));

        // Responsive breakpoint: auto-hide right panel on narrow windows
        {
            let cond = adw::BreakpointCondition::new_length(
                adw::BreakpointConditionLengthType::MaxWidth,
                900.0,
                adw::LengthUnit::Px,
            );
            let bp = adw::Breakpoint::new(cond);
            let val: glib::Value = false.into();
            bp.add_setter(outer_split.upcast_ref::<glib::Object>(), "show-sidebar", Some(&val));
            parent_window.add_breakpoint(bp);
        }

        let toast_overlay = adw::ToastOverlay::new();
        toast_overlay.set_child(Some(&outer_split));
        Toast::init(&toast_overlay);

        // Step 8: Set up presence and page change tracking
        if let Some(ref ctx) = state.borrow().backend {
            let presence = ServiceFactory::update_presence(ctx);
            presence.execute(UpdatePresenceInput { activity: Activity::Dashboard });
        }

        let db = dashboard;
        let cl = course_list;
        let cv = course_view;
        let vp = video_player;
        let ql = quiz_list;
        let qv = quiz_view;
        let st = settings;
        let rp = right_panel;
        let old_page: Rc<RefCell<String>> = Rc::new(RefCell::new(PAGE_DASHBOARD.to_string()));
        let nav_state = state;
        let tag_btn = tag_to_button.clone();

        nav_view_rc.connect_visible_page_notify(move |nav| {
            let new_tag =
                nav.visible_page().and_then(|p| p.tag()).map(|t| t.to_string()).unwrap_or_default();
            let old = old_page.borrow().clone();

            if old == PAGE_VIDEO_PLAYER && new_tag != PAGE_VIDEO_PLAYER {
                vp.stop();
            }

            // When navigating away from Settings, refresh pages that depend on LLM state
            // so they re-evaluate has_llm (e.g. video player action buttons).
            if old == PAGE_SETTINGS && new_tag != PAGE_SETTINGS {
                cv.refresh();
                cl.refresh();
            }

            *old_page.borrow_mut() = new_tag.clone();

            // Update sidebar toggle buttons to match current page
            if !new_tag.is_empty() {
                let tag_btns = tag_btn.borrow();
                if let Some(btn) = tag_btns.get(new_tag.as_str()) {
                    btn.set_active(true);
                }
            }

            let backend = nav_state.borrow().backend.clone();
            if let Some(ref ctx) = backend {
                let presence = ServiceFactory::update_presence(ctx);
                let activity = match new_tag.as_str() {
                    PAGE_DASHBOARD => Activity::Dashboard,
                    PAGE_COURSE_LIST => Activity::BrowsingCourses,
                    PAGE_COURSE_VIEW => Activity::BrowsingCourses,
                    PAGE_VIDEO_PLAYER => {
                        let state_ref = nav_state.borrow();
                        let video_title = state_ref
                            .current_video_id
                            .as_ref()
                            .and_then(|id_str| {
                                let vid_id =
                                    crate::domain::value_objects::VideoId::from_str(id_str).ok()?;
                                ctx.video_repo.find_by_id(&vid_id).ok().flatten()
                            })
                            .map(|v| v.title().to_string())
                            .unwrap_or_else(|| "Watching video".to_string());
                        let course_title = state_ref
                            .current_course_id
                            .as_ref()
                            .and_then(|id_str| {
                                let c_id = crate::domain::value_objects::CourseId::from_str(id_str)
                                    .ok()?;
                                ctx.course_repo.find_by_id(&c_id).ok().flatten()
                            })
                            .map(|c| c.name().to_string())
                            .unwrap_or_default();
                        drop(state_ref);
                        Activity::Watching { course_title, video_title }
                    },
                    PAGE_QUIZ_LIST => Activity::Idle,
                    PAGE_QUIZ_VIEW => {
                        let state_ref = nav_state.borrow();
                        let exam_title = state_ref
                            .current_quiz_id
                            .as_ref()
                            .and_then(|id_str| {
                                let exam_id =
                                    crate::domain::value_objects::ExamId::from_str(id_str).ok()?;
                                let exam = ctx.exam_repo.find_by_id(&exam_id).ok().flatten()?;
                                let vid_id = exam.video_id();
                                ctx.video_repo
                                    .find_by_id(vid_id)
                                    .ok()
                                    .flatten()
                                    .map(|v| v.title().to_string())
                            })
                            .unwrap_or_else(|| "Taking quiz".to_string());
                        let course_title = state_ref
                            .current_course_id
                            .as_ref()
                            .and_then(|id_str| {
                                let c_id = crate::domain::value_objects::CourseId::from_str(id_str)
                                    .ok()?;
                                ctx.course_repo.find_by_id(&c_id).ok().flatten()
                            })
                            .map(|c| c.name().to_string())
                            .unwrap_or_default();
                        drop(state_ref);
                        Activity::TakingExam { course_title, exam_title }
                    },
                    PAGE_SETTINGS => Activity::Settings,
                    _ => Activity::Idle,
                };
                presence.execute(UpdatePresenceInput { activity });
            }

            match new_tag.as_str() {
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

        toast_overlay
    }
}

#[allow(clippy::too_many_arguments)]
fn perform_search(
    state: &SharedState,
    nav: &adw::NavigationView,
    nav_pages: &Rc<HashMap<&'static str, adw::NavigationPage>>,
    search_popover: &gtk::Popover,
    query: &str,
) {
    let query_str = query.to_string();
    let backend_opt = state.borrow().backend.clone();
    let (tx, rx) = std::sync::mpsc::channel::<Vec<crate::domain::entities::SearchResult>>();

    crate::infrastructure::tokio_bridge::spawn(async move {
        let results = if let Some(ctx) = backend_opt {
            ctx.search_repo.search(&query_str, 20).unwrap_or_default()
        } else {
            vec![]
        };
        let _ = tx.send(results);
    });

    let state = state.clone();
    let nav = nav.clone();
    let nav_pages = nav_pages.clone();
    let search_popover = search_popover.clone();

    glib::idle_add_local(move || match rx.try_recv() {
        Ok(results) => {
            if results.is_empty() {
                search_popover.popdown();
                return glib::ControlFlow::Break;
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
                let s_cl = state.clone();
                let nav_cl = nav.clone();
                let pages_cl = nav_pages.clone();
                let popover_cl = search_popover.clone();
                let gesture = gtk::GestureClick::new();
                gesture.connect_released(move |_, _, _, _| {
                    popover_cl.popdown();
                    let mut s = s_cl.borrow_mut();
                    match r.entity_type {
                        SearchResultType::Course => {
                            s.current_course_id = Some(r.course_id.as_uuid().to_string());
                            if let Some(cv_page) = pages_cl.get(PAGE_COURSE_VIEW) {
                                nav_cl.push(cv_page);
                            }
                        },
                        SearchResultType::Video => {
                            s.current_course_id = Some(r.course_id.as_uuid().to_string());
                            s.current_video_id = Some(r.entity_id.clone());
                            if let Some(vp_page) = pages_cl.get(PAGE_VIDEO_PLAYER) {
                                nav_cl.push(vp_page);
                            }
                        },
                        SearchResultType::Note => {
                            s.current_course_id = Some(r.course_id.as_uuid().to_string());
                            if let Some(cv_page) = pages_cl.get(PAGE_COURSE_VIEW) {
                                nav_cl.push(cv_page);
                            }
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

            glib::ControlFlow::Break
        },
        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
        Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
    });
}
