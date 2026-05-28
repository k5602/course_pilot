use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

use adw::NavigationPage;
use adw::prelude::*;

use crate::domain::ports::{ExamRepository, StreamResolver, VideoRepository};
use crate::infrastructure::video::VideoPlayer;
use crate::ui::state::SharedState;
use crate::ui::toast::Toast;
use crate::ui::widgets::QualityDropDown;

fn fmt_ns(ns: u64) -> String {
    let total_secs = ns / 1_000_000_000;
    let hours = total_secs / 3600;
    let mins = (total_secs % 3600) / 60;
    let secs = total_secs % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, mins, secs)
    } else {
        format!("{:02}:{:02}", mins, secs)
    }
}

fn find_parent<T: IsA<gtk::Widget>>(start: &gtk::Widget) -> Option<T> {
    let mut p = start.parent();
    while let Some(w) = p {
        if let Ok(target) = w.clone().downcast::<T>() {
            return Some(target);
        }
        p = w.parent();
    }
    None
}

pub struct VideoPlayerPage {
    widget: gtk::ScrolledWindow,
    state: SharedState,
    nav_pages: Rc<RefCell<Rc<HashMap<&'static str, NavigationPage>>>>,
    player: Rc<RefCell<Option<VideoPlayer>>>,
    play_btn: gtk::Button,
    seek_bar: gtk::Scale,
    pos_label: gtk::Label,
    dur_label: gtk::Label,
    quality_selector: QualityDropDown,
    timer_source: RefCell<Option<glib::SourceId>>,
    video_title: gtk::Label,
    player_frame: gtk::Frame,
    status_page: adw::StatusPage,
    suppress_seek: Rc<Cell<bool>>,
    current_video_source: RefCell<Option<VideoSourceForQuality>>,
    suppress_quality: Rc<Cell<bool>>,
    is_playing: Rc<Cell<bool>>,
    summarize_btn: gtk::Button,
    quiz_btn: gtk::Button,
    quizzes_container: gtk::Box,
    transcript_lbl: gtk::Label,
    details_box: gtk::Box,
    is_fullscreen: Rc<Cell<bool>>,
    fullscreen_btn: gtk::Button,
}

#[derive(Clone)]
enum VideoSourceForQuality {
    YouTube(String),
    Local,
}

impl VideoPlayerPage {
    pub fn new(state: SharedState) -> Self {
        let scroll = gtk::ScrolledWindow::new();
        scroll.set_vexpand(true);
        scroll.set_hexpand(true);

        let widget = gtk::Box::new(gtk::Orientation::Vertical, 16);
        widget.add_css_class("content-area");
        scroll.set_child(Some(&widget));

        let video_title = gtk::Label::new(Some("Video Player"));
        video_title.add_css_class("heading");
        video_title.set_hexpand(true);
        video_title.set_halign(gtk::Align::Start);
        widget.append(&video_title);

        let player_frame = gtk::Frame::new(None);
        player_frame.set_hexpand(true);
        player_frame.set_vexpand(true);
        player_frame.set_height_request(400);
        player_frame.add_css_class("card");
        player_frame.set_margin_start(8);
        player_frame.set_margin_end(8);

        let status_page = adw::StatusPage::new();
        status_page.set_title("No Video Loaded");
        status_page.set_description(Some("Select a video to start watching."));
        status_page.set_icon_name(Some("video-x-generic-symbolic"));
        player_frame.set_child(Some(&status_page));
        widget.append(&player_frame);

        let controls = gtk::Box::new(gtk::Orientation::Vertical, 4);
        controls.set_margin_start(16);
        controls.set_margin_end(16);
        controls.set_margin_bottom(16);

        // Modern Seekbar at the very top of controls
        let seek_bar = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 0.5);
        seek_bar.set_hexpand(true);
        seek_bar.set_draw_value(false);
        seek_bar.add_css_class("player-slider");
        controls.append(&seek_bar);

        // Control bar layout below the seekbar
        let controls_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        controls_row.set_hexpand(true);

        // Left section: Playback controls and time display
        let left_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        left_box.set_halign(gtk::Align::Start);
        left_box.set_valign(gtk::Align::Center);

        let play_btn = gtk::Button::new();
        play_btn.set_icon_name("media-playback-start-symbolic");
        play_btn.add_css_class("circular");
        play_btn.add_css_class("flat");
        play_btn.set_tooltip_text(Some("Play / Pause"));
        left_box.append(&play_btn);

        let skip_back_btn = gtk::Button::new();
        skip_back_btn.set_icon_name("media-seek-backward-symbolic");
        skip_back_btn.add_css_class("circular");
        skip_back_btn.add_css_class("flat");
        skip_back_btn.set_tooltip_text(Some("Skip Back 10s"));
        left_box.append(&skip_back_btn);

        let skip_fwd_btn = gtk::Button::new();
        skip_fwd_btn.set_icon_name("media-seek-forward-symbolic");
        skip_fwd_btn.add_css_class("circular");
        skip_fwd_btn.add_css_class("flat");
        skip_fwd_btn.set_tooltip_text(Some("Skip Forward 10s"));
        left_box.append(&skip_fwd_btn);

        let pos_label = gtk::Label::new(Some("00:00"));
        let dur_label = gtk::Label::new(Some("00:00"));
        pos_label.add_css_class("subtitle");
        dur_label.add_css_class("subtitle");

        let time_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        time_box.set_valign(gtk::Align::Center);
        time_box.set_margin_start(8);
        let slash_label = gtk::Label::new(Some("/"));
        slash_label.add_css_class("subtitle");
        time_box.append(&pos_label);
        time_box.append(&slash_label);
        time_box.append(&dur_label);
        left_box.append(&time_box);

        controls_row.append(&left_box);

        // Center space separator
        let spacer = gtk::Box::new(gtk::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        controls_row.append(&spacer);

        // Right section: Summarize, Quiz, Notes, Volume, and Quality
        let right_box = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        right_box.set_halign(gtk::Align::End);
        right_box.set_valign(gtk::Align::Center);

        // Summarize button
        let summarize_btn = gtk::Button::with_label("Summarize");
        summarize_btn.add_css_class("pill");
        summarize_btn.add_css_class("flat");
        summarize_btn.set_has_tooltip(true);
        right_box.append(&summarize_btn);

        // Quiz button
        let quiz_btn = gtk::Button::with_label("Generate Quiz");
        quiz_btn.add_css_class("pill");
        quiz_btn.add_css_class("flat");
        quiz_btn.set_has_tooltip(true);
        right_box.append(&quiz_btn);

        // Dynamic notes popup button
        let notes_btn = gtk::Button::with_label("Edit Notes (Ctrl+N)");
        notes_btn.add_css_class("pill");
        notes_btn.add_css_class("flat");
        let notes_state = state.clone();
        notes_btn.connect_clicked(move |_| {
            crate::ui::notes_window::open_notes_window(notes_state.clone());
        });
        right_box.append(&notes_btn);

        // Volume control box
        let vol_box = gtk::Box::new(gtk::Orientation::Horizontal, 4);
        vol_box.set_valign(gtk::Align::Center);
        vol_box.set_margin_start(8);

        let vol_btn = gtk::Button::new();
        vol_btn.set_icon_name("audio-volume-high-symbolic");
        vol_btn.add_css_class("circular");
        vol_btn.add_css_class("flat");
        vol_btn.set_tooltip_text(Some("Mute / Unmute"));
        vol_box.append(&vol_btn);

        let vol_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.05);
        vol_scale.set_value(0.8);
        vol_scale.set_width_request(80);
        vol_scale.add_css_class("player-slider");
        vol_box.append(&vol_scale);
        right_box.append(&vol_box);

        // Quality dropdown
        let quality_selector = QualityDropDown::new();
        quality_selector.widget().add_css_class("flat");
        quality_selector.widget().set_tooltip_text(Some("Select quality"));
        right_box.append(quality_selector.widget());

        // Fullscreen button
        let fullscreen_btn = gtk::Button::new();
        fullscreen_btn.set_icon_name("view-fullscreen-symbolic");
        fullscreen_btn.add_css_class("circular");
        fullscreen_btn.add_css_class("flat");
        fullscreen_btn.set_tooltip_text(Some("Toggle Fullscreen (F)"));
        right_box.append(&fullscreen_btn);

        controls_row.append(&right_box);
        controls.append(&controls_row);
        widget.append(&controls);

        let player_rc: Rc<RefCell<Option<VideoPlayer>>> = Rc::new(RefCell::new(None));

        // Skip Back 10s click handler
        let player_sk = player_rc.clone();
        let seek_bar_sk = seek_bar.clone();
        skip_back_btn.connect_clicked(move |_| {
            let p = player_sk.borrow();
            if let Some(ref player) = *p
                && let Some(pos) = player.position()
            {
                let new_pos = pos.saturating_sub(10_000_000_000); // 10s in ns
                seek_bar_sk.set_value(new_pos as f64);
                player.seek(new_pos);
            }
        });

        // Skip Forward 10s click handler
        let player_sf = player_rc.clone();
        let seek_bar_sf = seek_bar.clone();
        skip_fwd_btn.connect_clicked(move |_| {
            let p = player_sf.borrow();
            if let Some(ref player) = *p
                && let Some(pos) = player.position()
                && let Some(dur) = player.duration()
            {
                let new_pos = (pos + 10_000_000_000).min(dur);
                seek_bar_sf.set_value(new_pos as f64);
                player.seek(new_pos);
            }
        });

        // Volume scale change handler: set volume and dynamically update volume icon
        let vol_btn_change = vol_btn.clone();
        let player_vol_scale = player_rc.clone();
        vol_scale.connect_value_changed(move |s| {
            let val = s.value();
            if let Some(ref p) = *player_vol_scale.borrow() {
                p.set_volume(val);
            }
            if val == 0.0 {
                vol_btn_change.set_icon_name("audio-volume-muted-symbolic");
            } else if val < 0.3 {
                vol_btn_change.set_icon_name("audio-volume-low-symbolic");
            } else if val < 0.7 {
                vol_btn_change.set_icon_name("audio-volume-medium-symbolic");
            } else {
                vol_btn_change.set_icon_name("audio-volume-high-symbolic");
            }
        });

        // Mute / Unmute toggle button handler
        let player_mute = player_rc.clone();
        let vol_scale_cl = vol_scale.clone();
        let vol_btn_cl = vol_btn.clone();
        let saved_volume = Rc::new(Cell::new(0.8));
        vol_btn.connect_clicked(move |_| {
            let current = vol_scale_cl.value();
            if current > 0.0 {
                saved_volume.set(current);
                vol_scale_cl.set_value(0.0);
                if let Some(ref p) = *player_mute.borrow() {
                    p.set_volume(0.0);
                }
                vol_btn_cl.set_icon_name("audio-volume-muted-symbolic");
            } else {
                let saved = saved_volume.get();
                vol_scale_cl.set_value(saved);
                if let Some(ref p) = *player_mute.borrow() {
                    p.set_volume(saved);
                }
                if saved < 0.3 {
                    vol_btn_cl.set_icon_name("audio-volume-low-symbolic");
                } else if saved < 0.7 {
                    vol_btn_cl.set_icon_name("audio-volume-medium-symbolic");
                } else {
                    vol_btn_cl.set_icon_name("audio-volume-high-symbolic");
                }
            }
        });

        let details_box = gtk::Box::new(gtk::Orientation::Vertical, 16);

        // Separator between Player and Scroll-Down Content
        let sep = gtk::Separator::new(gtk::Orientation::Horizontal);
        sep.set_margin_top(16);
        sep.set_margin_bottom(16);
        details_box.append(&sep);

        // 5. Related/Associated Quizzes Section
        let quizzes_title = gtk::Label::new(Some("Associated Challenges & Quizzes"));
        quizzes_title.add_css_class("heading");
        quizzes_title.set_halign(gtk::Align::Start);
        details_box.append(&quizzes_title);

        let quizzes_container = gtk::Box::new(gtk::Orientation::Horizontal, 12);
        quizzes_container.set_margin_top(8);
        quizzes_container.set_margin_bottom(16);
        details_box.append(&quizzes_container);

        // 6. Video Summary Section
        let transcript_title = gtk::Label::new(Some("Video Summary"));
        transcript_title.add_css_class("heading");
        transcript_title.set_halign(gtk::Align::Start);
        details_box.append(&transcript_title);

        let transcript_frame = gtk::Frame::new(None);
        transcript_frame.add_css_class("card");
        transcript_frame.set_margin_top(8);
        transcript_frame.set_margin_bottom(16);

        let transcript_lbl = gtk::Label::new(Some(
            "No summary generated yet. Click 'Summarize' above to generate an AI summary.",
        ));
        transcript_lbl.set_wrap(true);
        transcript_lbl.set_halign(gtk::Align::Start);
        transcript_lbl.set_valign(gtk::Align::Start);
        transcript_lbl.add_css_class("subtitle");
        transcript_lbl.set_margin_start(16);
        transcript_lbl.set_margin_end(16);
        transcript_lbl.set_margin_top(16);
        transcript_lbl.set_margin_bottom(16);
        transcript_frame.set_child(Some(&transcript_lbl));
        details_box.append(&transcript_frame);

        widget.append(&details_box);

        let nav_pages = Rc::new(RefCell::new(Rc::new(HashMap::new())));
        let is_playing = Rc::new(Cell::new(false));

        let sum_state = state.clone();
        let lbl_clone = transcript_lbl.clone();
        summarize_btn.connect_clicked(move |_| {
            let s = sum_state.borrow();
            let video_id_str = match s.current_video_id {
                Some(ref id) => id.clone(),
                None => return,
            };
            if let Some(ref ctx) = s.backend
                && let Some(uc) = crate::application::ServiceFactory::summarize_video(ctx)
            {
                let input = crate::application::use_cases::SummarizeVideoInput {
                    video_id: video_id_str.clone().parse().unwrap(),
                    force_refresh: true,
                };
                Toast::show("Summarization started...");
                let (tx, rx) = std::sync::mpsc::channel::<Result<String, String>>();
                crate::infrastructure::tokio_bridge::spawn(async move {
                    let res = match uc.execute(input).await {
                        Ok(out) => Ok(out.summary),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = tx.send(res);
                });

                let lbl = lbl_clone.clone();
                glib::idle_add_local(move || match rx.try_recv() {
                    Ok(Ok(summary)) => {
                        lbl.set_text(&summary);
                        Toast::show("Summary generated successfully!");
                        glib::ControlFlow::Break
                    },
                    Ok(Err(e)) => {
                        lbl.set_text(&format!("Error generating summary: {}", e));
                        Toast::show_error(&format!("Summarization failed: {}", e));
                        glib::ControlFlow::Break
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                });
            }
        });

        let quiz_state = state.clone();
        let quizzes_container_cl = quizzes_container.clone();
        let nav_pages_cl = nav_pages.clone();
        let scroll_cl = scroll.clone();
        quiz_btn.connect_clicked(move |_| {
            let s = quiz_state.borrow();
            let video_id_str = match s.current_video_id {
                Some(ref id) => id.clone(),
                None => return,
            };
            if let Some(ref ctx) = s.backend
                && let Some(uc) = crate::application::ServiceFactory::take_exam(ctx)
            {
                use crate::domain::value_objects::ExamDifficulty;
                let input = crate::application::use_cases::GenerateExamInput {
                    video_id: video_id_str.clone().parse().unwrap(),
                    num_questions: 5,
                    difficulty: ExamDifficulty::Medium,
                };
                Toast::show("Quiz generation started...");
                let (tx, rx) = std::sync::mpsc::channel::<Result<(), String>>();
                crate::infrastructure::tokio_bridge::spawn(async move {
                    let res = match uc.generate(input).await {
                        Ok(_) => Ok(()),
                        Err(e) => Err(e.to_string()),
                    };
                    let _ = tx.send(res);
                });

                let state_cl2 = quiz_state.clone();
                let quizzes_container_cl2 = quizzes_container_cl.clone();
                let nav_pages_cl2 = nav_pages_cl.clone();
                let scroll_cl2 = scroll_cl.clone();
                let video_id_cl = video_id_str.clone();

                glib::idle_add_local(move || match rx.try_recv() {
                    Ok(Ok(())) => {
                        Toast::show("Quiz generated successfully!");
                        while let Some(child) = quizzes_container_cl2.first_child() {
                            quizzes_container_cl2.remove(&child);
                        }

                        let s = state_cl2.borrow();
                        if let Some(ref ctx) = s.backend
                            && let Ok(quizzes) =
                                ctx.exam_repo.find_by_video(&video_id_cl.parse().unwrap())
                            {
                                if quizzes.is_empty() {
                                    let empty_q_lbl = gtk::Label::new(Some(
                                        "No quizzes generated for this video yet. Click 'Generate Quiz' to start!",
                                    ));
                                    empty_q_lbl.add_css_class("subtitle");
                                    empty_q_lbl.set_halign(gtk::Align::Start);
                                    quizzes_container_cl2.append(&empty_q_lbl);
                                } else {
                                    for quiz in quizzes {
                                        let quiz_btn = gtk::Button::with_label(&format!(
                                            "Challenge: Quiz #{}",
                                            quiz.id()
                                        ));
                                        quiz_btn.add_css_class("suggested-action");
                                        quiz_btn.set_halign(gtk::Align::Start);

                                        let state_cl = state_cl2.clone();
                                        let nav_cl = nav_pages_cl2.clone();
                                        let nav_view_cl = scroll_cl2
                                            .upcast_ref::<gtk::Widget>()
                                            .parent()
                                            .and_then(|w| w.parent())
                                            .and_then(|w| w.downcast::<adw::NavigationView>().ok());
                                        let quiz_id_str = quiz.id().to_string();

                                        quiz_btn.connect_clicked(move |_| {
                                            state_cl.borrow_mut().current_quiz_id =
                                                Some(quiz_id_str.clone());
                                            let pages = nav_cl.borrow();
                                            if let Some(page) =
                                                pages.get(crate::ui::navigation::PAGE_QUIZ_VIEW)
                                                && let Some(ref nv) = nav_view_cl {
                                                    nv.push(page);
                                                }
                                        });

                                        quizzes_container_cl2.append(&quiz_btn);
                                    }
                                }
                            }
                        glib::ControlFlow::Break
                    },
                    Ok(Err(e)) => {
                        Toast::show_error(&format!("Quiz generation failed: {}", e));
                        glib::ControlFlow::Break
                    },
                    Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
                });
            }
        });

        let page = Self {
            widget: scroll,
            state,
            nav_pages,
            player: player_rc,
            play_btn,
            seek_bar: seek_bar.clone(),
            pos_label,
            dur_label,
            quality_selector: quality_selector.clone(),
            timer_source: RefCell::new(None),
            video_title,
            player_frame,
            status_page,
            suppress_seek: Rc::new(Cell::new(false)),
            current_video_source: RefCell::new(None),
            suppress_quality: Rc::new(Cell::new(false)),
            is_playing,
            summarize_btn,
            quiz_btn,
            quizzes_container,
            transcript_lbl,
            details_box,
            is_fullscreen: Rc::new(Cell::new(false)),
            fullscreen_btn,
        };

        // Wire GStreamer seek scale gestures
        let player_seek = page.player.clone();
        let suppress_seek_gest = page.suppress_seek.clone();
        seek_bar.connect_value_changed(move |scale| {
            if !suppress_seek_gest.get()
                && let Some(ref p) = *player_seek.borrow()
            {
                p.seek(scale.value() as u64);
            }
        });

        // Wire Play/Pause button click
        let player_play = page.player.clone();
        let is_playing_gest = page.is_playing.clone();
        let play_btn_gest = page.play_btn.clone();
        page.play_btn.connect_clicked(move |_| {
            if let Some(ref p) = *player_play.borrow() {
                let current = is_playing_gest.get();
                if current {
                    p.pause();
                    play_btn_gest.set_icon_name("media-playback-start-symbolic");
                } else {
                    p.resume();
                    play_btn_gest.set_icon_name("media-playback-pause-symbolic");
                }
                is_playing_gest.set(!current);
            }
        });

        page.setup_quality_handling();
        page.setup_keyboard_shortcuts();
        page.setup_fullscreen();

        page
    }

    fn setup_quality_handling(&self) {
        let player = self.player.clone();
        let state = self.state.clone();
        let is_playing = self.is_playing.clone();
        let play_btn = self.play_btn.clone();
        let current_source = self.current_video_source.clone();
        let suppress = self.suppress_quality.clone();

        self.quality_selector.connect_selected(move |quality| {
            if suppress.get() {
                return;
            }
            let yid = match *current_source.borrow() {
                Some(VideoSourceForQuality::YouTube(ref id)) => id.clone(),
                _ => return,
            };

            let pos = player.borrow().as_ref().and_then(|p| p.position()).unwrap_or(0);

            if let Some(ref p) = *player.borrow() {
                p.stop();
            }

            play_btn.set_icon_name("process-working-symbolic");
            play_btn.set_sensitive(false);

            let (tx, rx) = std::sync::mpsc::channel::<String>();
            let yid_clone = yid.clone();
            let backend_opt = state.borrow().backend.clone();

            if let Some(ctx) = backend_opt {
                crate::infrastructure::tokio_bridge::spawn(async move {
                    match ctx.youtube.resolve_youtube_stream(&yid_clone, quality).await {
                        Ok(url) => {
                            let _ = tx.send(url);
                        },
                        Err(e) => {
                            let _ = tx.send(format!("ERROR:{}", e));
                        },
                    }
                });
            }

            let player_rc = player.clone();
            let play_btn_cl = play_btn.clone();
            let is_playing_cl = is_playing.clone();

            glib::idle_add_local(move || match rx.try_recv() {
                Ok(msg) if msg.starts_with("ERROR:") => {
                    let detail = msg.strip_prefix("ERROR:").unwrap_or("Stream resolution failed");
                    Toast::show_error(detail);
                    is_playing_cl.set(false);
                    play_btn_cl.set_icon_name("media-playback-start-symbolic");
                    play_btn_cl.set_sensitive(true);
                    glib::ControlFlow::Break
                },
                Ok(stream_url) => {
                    if let Some(ref p) = *player_rc.borrow() {
                        p.play_uri(&stream_url);
                        p.seek(pos);
                    }
                    is_playing_cl.set(true);
                    play_btn_cl.set_icon_name("media-playback-pause-symbolic");
                    play_btn_cl.set_sensitive(true);
                    glib::ControlFlow::Break
                },
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            });
        });
    }

    pub fn set_nav_pages(&self, pages: Rc<HashMap<&'static str, NavigationPage>>) {
        *self.nav_pages.borrow_mut() = pages;
    }

    fn setup_keyboard_shortcuts(&self) {
        let controller = gtk::EventControllerKey::new();
        let player = self.player.clone();
        let seek_bar = self.seek_bar.clone();
        let play_btn = self.play_btn.clone();
        let is_playing = self.is_playing.clone();

        let is_fs = self.is_fullscreen.clone();
        let widget = self.widget.clone();
        let video_title = self.video_title.clone();
        let details_box = self.details_box.clone();
        let state = self.state.clone();

        let toggle_shortcut = move || {
            let is_fs_val = is_fs.get();
            let new_fs = !is_fs_val;
            is_fs.set(new_fs);

            let root_window = widget.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            if let Some(win) = root_window {
                if new_fs {
                    win.fullscreen();
                } else {
                    win.unfullscreen();
                }
            }

            let w_ref = widget.upcast_ref::<gtk::Widget>();
            let inner_split = find_parent::<adw::NavigationSplitView>(w_ref);
            let outer_split = find_parent::<adw::OverlaySplitView>(w_ref);
            let toolbar = find_parent::<adw::ToolbarView>(w_ref);

            if new_fs {
                if let Some(split) = inner_split {
                    split.set_collapsed(true);
                }
                if let Some(split) = outer_split {
                    split.set_show_sidebar(false);
                }
                if let Some(tb) = toolbar {
                    tb.set_reveal_top_bars(false);
                }
                video_title.set_visible(false);
                details_box.set_visible(false);
            } else {
                if let Some(split) = inner_split {
                    split.set_collapsed(false);
                }
                if let Some(split) = outer_split {
                    let show_chat = state.borrow().right_panel_visible;
                    split.set_show_sidebar(show_chat);
                }
                if let Some(tb) = toolbar {
                    tb.set_reveal_top_bars(true);
                }
                video_title.set_visible(true);
                details_box.set_visible(true);
            }
        };

        controller.connect_key_pressed(move |_, keyval, _code, _state| match keyval {
            gtk::gdk::Key::Left | gtk::gdk::Key::KP_Left => {
                let val = seek_bar.value();
                let new_val = (val - 5_000_000_000.0).max(seek_bar.adjustment().lower());
                seek_bar.set_value(new_val);
                if let Some(ref p) = *player.borrow() {
                    p.seek(new_val as u64);
                }
                glib::Propagation::Stop
            },
            gtk::gdk::Key::Right | gtk::gdk::Key::KP_Right => {
                let val = seek_bar.value();
                let new_val = (val + 5_000_000_000.0).min(seek_bar.adjustment().upper());
                seek_bar.set_value(new_val);
                if let Some(ref p) = *player.borrow() {
                    p.seek(new_val as u64);
                }
                glib::Propagation::Stop
            },
            gtk::gdk::Key::Up | gtk::gdk::Key::KP_Up => glib::Propagation::Proceed,
            gtk::gdk::Key::Down | gtk::gdk::Key::KP_Down => glib::Propagation::Proceed,
            gtk::gdk::Key::space => {
                if let Some(ref p) = *player.borrow() {
                    let current = is_playing.get();
                    if current {
                        p.pause();
                        play_btn.set_icon_name("media-playback-start-symbolic");
                    } else {
                        p.resume();
                        play_btn.set_icon_name("media-playback-pause-symbolic");
                    }
                    is_playing.set(!current);
                }
                glib::Propagation::Stop
            },
            gtk::gdk::Key::f | gtk::gdk::Key::F | gtk::gdk::Key::F11 => {
                toggle_shortcut();
                glib::Propagation::Stop
            },
            _ => glib::Propagation::Proceed,
        });

        self.widget.add_controller(controller);
    }

    pub fn widget(&self) -> &gtk::ScrolledWindow {
        &self.widget
    }

    pub fn refresh(&self) {
        self.stop_timer();
        if let Some(ref p) = *self.player.borrow() {
            p.stop();
        }
        *self.player.borrow_mut() = None;
        self.is_playing.set(false);
        self.play_btn.set_icon_name("media-playback-start-symbolic");

        // Clear associated quizzes
        while let Some(child) = self.quizzes_container.first_child() {
            self.quizzes_container.remove(&child);
        }

        // Dynamically check LLM availability so buttons update after key is added in Settings
        let has_llm = self
            .state
            .borrow()
            .backend
            .as_ref()
            .and_then(|ctx| crate::application::ServiceFactory::summarize_video(ctx))
            .is_some();
        self.summarize_btn.set_sensitive(has_llm);
        self.quiz_btn.set_sensitive(has_llm);
        if has_llm {
            self.summarize_btn.set_tooltip_text(Some("Generate an AI summary of this video"));
            self.quiz_btn.set_tooltip_text(Some("Generate a quiz based on this video"));
        } else {
            self.summarize_btn
                .set_tooltip_text(Some("Requires a Gemini API key — add one in Settings"));
            self.quiz_btn.set_tooltip_text(Some("Requires a Gemini API key — add one in Settings"));
        }

        let state = self.state.borrow();
        let video_id_str = match state.current_video_id {
            Some(ref id) => id.clone(),
            None => {
                self.video_title.set_text("No video selected.");
                self.player_frame.set_child(Some(&self.status_page));
                self.transcript_lbl.set_text("No transcript loaded.");
                return;
            },
        };

        if let Some(ref ctx) = state.backend {
            let video_id = match video_id_str.parse::<crate::domain::value_objects::VideoId>() {
                Ok(id) => id,
                Err(_) => {
                    self.video_title.set_text("Invalid video ID.");
                    self.player_frame.set_child(Some(&self.status_page));
                    return;
                },
            };

            match ctx.video_repo.find_by_id(&video_id) {
                Ok(Some(video)) => {
                    self.video_title.set_text(video.title());

                    let player = match VideoPlayer::new() {
                        Ok(p) => p,
                        Err(e) => {
                            self.video_title.set_text(&format!("Player error: {}", e));
                            self.player_frame.set_child(Some(&self.status_page));
                            return;
                        },
                    };

                    let picture = player.widget();
                    picture.set_vexpand(true);
                    picture.set_hexpand(true);

                    self.player_frame.set_child(Some(picture));

                    let dur_ns = (video.duration_secs() as u64) * 1_000_000_000;
                    self.seek_bar.set_range(0.0, dur_ns as f64);
                    self.seek_bar.set_value(0.0);
                    self.pos_label.set_text("00:00");
                    self.dur_label.set_text(&fmt_ns(dur_ns));

                    let quality = state.session_quality;
                    match video.source() {
                        crate::domain::value_objects::VideoSource::YouTube(yid) => {
                            player.set_volume(0.8);
                            *self.current_video_source.borrow_mut() =
                                Some(VideoSourceForQuality::YouTube(yid.as_str().to_string()));
                            *self.player.borrow_mut() = Some(player);
                            self.play_btn.set_icon_name("process-working-symbolic");
                            self.play_btn.set_sensitive(false);

                            self.suppress_quality.set(true);
                            self.quality_selector.set_quality(quality);
                            self.suppress_quality.set(false);

                            let yid_str = yid.as_str().to_string();
                            let backend_opt = self.state.borrow().backend.clone();
                            let (tx, rx) = std::sync::mpsc::channel::<String>();

                            if let Some(ctx) = backend_opt {
                                crate::infrastructure::tokio_bridge::spawn(async move {
                                    match ctx
                                        .youtube
                                        .resolve_youtube_stream(&yid_str, quality)
                                        .await
                                    {
                                        Ok(url) => {
                                            let _ = tx.send(url);
                                        },
                                        Err(e) => {
                                            let _ = tx.send(format!("ERROR:{}", e));
                                        },
                                    }
                                });
                            }

                            let player_rc = self.player.clone();
                            let play_btn_cl = self.play_btn.clone();
                            let is_playing_cl = self.is_playing.clone();

                            glib::idle_add_local(move || match rx.try_recv() {
                                Ok(msg) if msg.starts_with("ERROR:") => {
                                    let detail = msg
                                        .strip_prefix("ERROR:")
                                        .unwrap_or("Stream resolution failed");
                                    Toast::show_error(detail);
                                    is_playing_cl.set(false);
                                    play_btn_cl.set_icon_name("media-playback-start-symbolic");
                                    play_btn_cl.set_sensitive(true);
                                    glib::ControlFlow::Break
                                },
                                Ok(stream_url) => {
                                    if let Some(ref p) = *player_rc.borrow() {
                                        p.play_uri(&stream_url);
                                    }
                                    is_playing_cl.set(true);
                                    play_btn_cl.set_icon_name("media-playback-pause-symbolic");
                                    play_btn_cl.set_sensitive(true);
                                    glib::ControlFlow::Break
                                },
                                Err(std::sync::mpsc::TryRecvError::Empty) => {
                                    glib::ControlFlow::Continue
                                },
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                                    glib::ControlFlow::Break
                                },
                            });
                        },
                        crate::domain::value_objects::VideoSource::LocalPath(path) => {
                            *self.current_video_source.borrow_mut() =
                                Some(VideoSourceForQuality::Local);
                            player.set_volume(0.8);
                            let uri = format!("file://{path}");
                            player.play_uri(&uri);
                            *self.player.borrow_mut() = Some(player);
                            self.is_playing.set(true);
                            self.play_btn.set_icon_name("media-playback-pause-symbolic");
                        },
                    };

                    self.start_timer();

                    // Load Summary
                    if let Some(s) = video.summary() {
                        self.transcript_lbl.set_text(s);
                    } else {
                        self.transcript_lbl.set_text("No summary generated yet. Click 'Summarize' above to generate an AI summary.");
                    }

                    // Load quizzes related to this video dynamically
                    while let Some(child) = self.quizzes_container.first_child() {
                        self.quizzes_container.remove(&child);
                    }
                    if let Ok(quizzes) = ctx.exam_repo.find_by_video(&video_id) {
                        if quizzes.is_empty() {
                            let empty_q_lbl = gtk::Label::new(Some(
                                "No quizzes generated for this video yet. Click 'Generate Quiz' to start!",
                            ));
                            empty_q_lbl.add_css_class("subtitle");
                            empty_q_lbl.set_halign(gtk::Align::Start);
                            self.quizzes_container.append(&empty_q_lbl);
                        } else {
                            for quiz in quizzes {
                                let quiz_btn = gtk::Button::with_label(&format!(
                                    "Challenge: Quiz #{}",
                                    quiz.id()
                                ));
                                quiz_btn.add_css_class("suggested-action");
                                quiz_btn.set_halign(gtk::Align::Start);

                                let state_cl = self.state.clone();
                                let nav_cl = self.nav_pages.clone();
                                let nav_view_cl = self
                                    .widget
                                    .parent()
                                    .and_then(|w| w.parent())
                                    .and_then(|w| w.downcast::<adw::NavigationView>().ok());
                                let quiz_id_str = quiz.id().to_string();

                                quiz_btn.connect_clicked(move |_| {
                                    state_cl.borrow_mut().current_quiz_id =
                                        Some(quiz_id_str.clone());
                                    let pages = nav_cl.borrow();
                                    if let Some(page) =
                                        pages.get(crate::ui::navigation::PAGE_QUIZ_VIEW)
                                    {
                                        // Dynamically navigate to quiz page
                                        if let Some(ref nv) = nav_view_cl {
                                            nv.push(page);
                                        }
                                    }
                                });

                                self.quizzes_container.append(&quiz_btn);
                            }
                        }
                    }
                },
                Ok(None) => {
                    self.video_title.set_text("Video not found.");
                    self.player_frame.set_child(Some(&self.status_page));
                    self.transcript_lbl.set_text("No summary loaded.");
                },
                Err(e) => {
                    self.video_title.set_text(&format!("Error: {}", e));
                    self.player_frame.set_child(Some(&self.status_page));
                    self.transcript_lbl.set_text("Error loading summary.");
                },
            }
        } else {
            self.video_title.set_text("No backend connected.");
            self.player_frame.set_child(Some(&self.status_page));
            self.transcript_lbl.set_text("No backend connected.");
        }
    }

    pub fn stop(&self) {
        self.stop_timer();
        if let Some(ref p) = *self.player.borrow() {
            p.stop();
        }
        *self.player.borrow_mut() = None;
        self.is_playing.set(false);
        self.player_frame.set_child(Some(&self.status_page));
        self.play_btn.set_icon_name("media-playback-start-symbolic");
    }

    fn start_timer(&self) {
        self.stop_timer();

        let player = self.player.clone();
        let seek_bar = self.seek_bar.clone();
        let pos_label = self.pos_label.clone();
        let dur_label = self.dur_label.clone();
        let suppress = self.suppress_seek.clone();

        let source_id = glib::timeout_add_local(std::time::Duration::from_millis(250), move || {
            let p = player.borrow();
            if let Some(ref player) = *p {
                if let Some(pos) = player.position() {
                    suppress.set(true);
                    seek_bar.set_value(pos as f64);
                    suppress.set(false);
                    pos_label.set_text(&fmt_ns(pos));
                }
                if let Some(dur) = player.duration() {
                    dur_label.set_text(&fmt_ns(dur));
                }
            }
            glib::ControlFlow::Continue
        });

        *self.timer_source.borrow_mut() = Some(source_id);
    }

    fn stop_timer(&self) {
        if let Some(source_id) = self.timer_source.borrow_mut().take() {
            source_id.remove();
        }
    }

    fn setup_fullscreen(&self) {
        let is_fs = self.is_fullscreen.clone();
        let widget = self.widget.clone();
        let video_title = self.video_title.clone();
        let details_box = self.details_box.clone();
        let state = self.state.clone();
        let fullscreen_btn = self.fullscreen_btn.clone();
        let player_frame = self.player_frame.clone();

        let toggle = move || {
            let is_fs_val = is_fs.get();
            let new_fs = !is_fs_val;
            is_fs.set(new_fs);

            let root_window = widget.root().and_then(|r| r.downcast::<gtk::Window>().ok());
            if let Some(win) = root_window {
                if new_fs {
                    win.fullscreen();
                } else {
                    win.unfullscreen();
                }
            }

            let w_ref = widget.upcast_ref::<gtk::Widget>();
            let inner_split = find_parent::<adw::NavigationSplitView>(w_ref);
            let outer_split = find_parent::<adw::OverlaySplitView>(w_ref);
            let toolbar = find_parent::<adw::ToolbarView>(w_ref);

            if new_fs {
                if let Some(split) = inner_split {
                    split.set_collapsed(true);
                }
                if let Some(split) = outer_split {
                    split.set_show_sidebar(false);
                }
                if let Some(tb) = toolbar {
                    tb.set_reveal_top_bars(false);
                }
                video_title.set_visible(false);
                details_box.set_visible(false);
            } else {
                if let Some(split) = inner_split {
                    split.set_collapsed(false);
                }
                if let Some(split) = outer_split {
                    let show_chat = state.borrow().right_panel_visible;
                    split.set_show_sidebar(show_chat);
                }
                if let Some(tb) = toolbar {
                    tb.set_reveal_top_bars(true);
                }
                video_title.set_visible(true);
                details_box.set_visible(true);
            }
        };

        let toggle_btn = toggle.clone();
        fullscreen_btn.connect_clicked(move |_| {
            toggle_btn();
        });

        let toggle_gest = toggle.clone();
        let gesture = gtk::GestureClick::new();
        gesture.set_button(1);
        gesture.connect_pressed(move |gest, n_press, _, _| {
            if n_press == 2 {
                gest.set_state(gtk::EventSequenceState::Claimed);
                toggle_gest();
            }
        });
        player_frame.add_controller(gesture);
    }
}
