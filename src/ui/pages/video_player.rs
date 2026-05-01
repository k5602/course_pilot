use std::cell::{Cell, RefCell};
use std::rc::Rc;

use adw::prelude::*;

use crate::domain::ports::VideoRepository;
use crate::infrastructure::video::VideoPlayer;
use crate::ui::state::SharedState;
use crate::ui::widgets::QualitySelector;

fn fmt_ns(ns: u64) -> String {
    let total_secs = ns / 1_000_000_000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}

pub struct VideoPlayerPage {
    widget: gtk::Box,
    state: SharedState,
    player: Rc<RefCell<Option<VideoPlayer>>>,
    play_btn: gtk::Button,
    seek_bar: gtk::Scale,
    pos_label: gtk::Label,
    dur_label: gtk::Label,
    vol_scale: gtk::Scale,
    quality_selector: QualitySelector,
    timer_source: RefCell<Option<glib::SourceId>>,
    video_title: gtk::Label,
    player_frame: gtk::Frame,
    status_page: adw::StatusPage,
    suppress_seek: Rc<Cell<bool>>,
    current_video_source: RefCell<Option<VideoSourceForQuality>>,
}

#[derive(Clone)]
enum VideoSourceForQuality {
    YouTube(String),
    Local,
}

impl VideoPlayerPage {
    pub fn new(state: SharedState) -> Self {
        let widget = gtk::Box::new(gtk::Orientation::Vertical, 8);
        widget.add_css_class("content-area");

        let video_title = gtk::Label::new(Some("Video Player"));
        video_title.add_css_class("heading");
        video_title.set_hexpand(true);
        video_title.set_halign(gtk::Align::Start);
        widget.append(&video_title);

        let player_frame = gtk::Frame::new(None);
        player_frame.set_hexpand(true);
        player_frame.set_vexpand(true);
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
        controls.set_margin_start(8);
        controls.set_margin_end(8);
        controls.set_margin_bottom(8);

        let seek_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);

        let pos_label = gtk::Label::new(Some("00:00"));
        seek_box.append(&pos_label);

        let seek_bar = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 100.0, 0.5);
        seek_bar.set_hexpand(true);
        seek_bar.set_draw_value(false);
        seek_box.append(&seek_bar);

        let dur_label = gtk::Label::new(Some("00:00"));
        seek_box.append(&dur_label);

        controls.append(&seek_box);

        let btn_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        btn_box.set_halign(gtk::Align::Center);

        let play_btn = gtk::Button::with_label("Play");
        play_btn.add_css_class("circular");
        btn_box.append(&play_btn);

        controls.append(&btn_box);

        let mid_box = gtk::Box::new(gtk::Orientation::Horizontal, 16);
        mid_box.set_halign(gtk::Align::Center);

        let quality_selector = QualitySelector::new("Quality");

        let vol_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
        let vol_label = gtk::Label::new(Some("Volume"));
        vol_box.append(&vol_label);

        let vol_scale = gtk::Scale::with_range(gtk::Orientation::Horizontal, 0.0, 1.0, 0.05);
        vol_scale.set_value(0.8);
        vol_scale.set_width_request(120);
        vol_box.append(&vol_scale);

        mid_box.append(quality_selector.widget());
        mid_box.append(&vol_box);
        controls.append(&mid_box);

        widget.append(&controls);

        let player_rc: Rc<RefCell<Option<VideoPlayer>>> = Rc::new(RefCell::new(None));

        let player_cl = player_rc.clone();
        vol_scale.connect_value_changed(move |s| {
            if let Some(ref p) = *player_cl.borrow() {
                p.set_volume(s.value());
            }
        });

        let suppress_seek = Rc::new(Cell::new(false));
        let player_cl = player_rc.clone();
        let seek_bar_cl = seek_bar.clone();
        let suppress_seek_cl = suppress_seek.clone();
        seek_bar.connect_value_changed(move |_| {
            if suppress_seek_cl.get() {
                return;
            }
            if let Some(ref p) = *player_cl.borrow() {
                let val_ns = seek_bar_cl.value() as u64;
                p.seek(val_ns);
            }
        });

        let player_cl = player_rc.clone();
        let play_btn_cl = play_btn.clone();
        play_btn.connect_clicked(move |_| {
            let p = player_cl.borrow_mut();
            if let Some(ref player) = *p {
                if play_btn_cl.label().as_deref() == Some("Play") {
                    player.resume();
                    play_btn_cl.set_label("Pause");
                } else {
                    player.pause();
                    play_btn_cl.set_label("Play");
                }
            }
        });

        let page = Self {
            widget,
            state: state.clone(),
            player: player_rc,
            play_btn,
            seek_bar,
            pos_label,
            dur_label,
            vol_scale,
            quality_selector,
            timer_source: RefCell::new(None),
            video_title,
            player_frame,
            status_page,
            suppress_seek,
            current_video_source: RefCell::new(None),
        };

        // Quality change handler: re-resolve YouTube stream URL
        let state_q = state.clone();
        let player_q = page.player.clone();
        let source_q = page.current_video_source.clone();
        page.quality_selector.connect_selected(move |quality| {
            let src = source_q.borrow().clone();
            match src {
                Some(VideoSourceForQuality::YouTube(yid)) => {
                    let saved_pos = {
                        let p = player_q.borrow();
                        p.as_ref().and_then(|pl| pl.position())
                    };
                    // Stop current playback
                    if let Some(ref p) = *player_q.borrow() {
                        p.stop();
                    }
                    // Save the new quality as session override
                    state_q.borrow_mut().session_quality = quality;
                    // Re-resolve and play
                    let yid_clone = yid.clone();
                    let player_cl = player_q.clone();
                    let (tx, rx) = std::sync::mpsc::channel::<String>();
                    crate::infrastructure::tokio_bridge::spawn(async move {
                        let result = crate::infrastructure::youtube::resolve_youtube_stream_inner(
                            &yid_clone, quality,
                        )
                        .await;
                        match result {
                            Ok(url) => {
                                let _ = tx.send(url);
                            },
                            Err(e) => log::error!("Failed to resolve stream: {e}"),
                        }
                    });
                    glib::idle_add_local(move || match rx.try_recv() {
                        Ok(stream_url) => {
                            if let Some(ref p) = *player_cl.borrow() {
                                p.play_uri(&stream_url);
                                // Seek to saved position
                                if let Some(pos) = saved_pos {
                                    p.seek(pos);
                                }
                            }
                            glib::ControlFlow::Break
                        },
                        Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                        Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                            glib::ControlFlow::Break
                        },
                    });
                },
                Some(VideoSourceForQuality::Local) | None => {},
            }
        });

        page.setup_keyboard_shortcuts();

        page
    }

    fn setup_keyboard_shortcuts(&self) {
        let controller = gtk::EventControllerKey::new();
        let player = self.player.clone();
        let seek_bar = self.seek_bar.clone();
        let play_btn = self.play_btn.clone();

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
            gtk::gdk::Key::Up | gtk::gdk::Key::KP_Up => {
                let val = seek_bar.value();
                let new_val = (val + 10_000_000_000.0).min(seek_bar.adjustment().upper());
                seek_bar.set_value(new_val);
                if let Some(ref p) = *player.borrow() {
                    p.seek(new_val as u64);
                }
                glib::Propagation::Stop
            },
            gtk::gdk::Key::Down | gtk::gdk::Key::KP_Down => {
                let val = seek_bar.value();
                let new_val = (val - 10_000_000_000.0).max(seek_bar.adjustment().lower());
                seek_bar.set_value(new_val);
                if let Some(ref p) = *player.borrow() {
                    p.seek(new_val as u64);
                }
                glib::Propagation::Stop
            },
            gtk::gdk::Key::space => {
                let p = player.borrow();
                if let Some(ref player) = *p {
                    if play_btn.label().as_deref() == Some("Play") {
                        player.resume();
                        play_btn.set_label("Pause");
                    } else {
                        player.pause();
                        play_btn.set_label("Play");
                    }
                }
                glib::Propagation::Stop
            },
            _ => glib::Propagation::Proceed,
        });
        self.widget.add_controller(controller);
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.widget
    }

    pub fn refresh(&self) {
        self.stop_timer();
        if let Some(ref p) = *self.player.borrow() {
            p.stop();
        }

        let state = self.state.borrow();
        let video_id_str = match state.current_video_id {
            Some(ref id) => id.clone(),
            None => {
                self.video_title.set_text("No video selected.");
                self.player_frame.set_child(Some(&self.status_page));
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
                            *self.current_video_source.borrow_mut() =
                                Some(VideoSourceForQuality::YouTube(yid.as_str().to_string()));
                            self.quality_selector.set_quality(quality);
                            *self.player.borrow_mut() = Some(player);
                            self.play_btn.set_label("Loading...");
                            self.play_btn.set_sensitive(false);
                            self.vol_scale.set_value(0.8);

                            let yid_str = yid.as_str().to_string();
                            let player_rc = self.player.clone();
                            let (tx, rx) = std::sync::mpsc::channel::<String>();

                            crate::infrastructure::tokio_bridge::spawn(async move {
                                match crate::infrastructure::youtube::resolve_youtube_stream_inner(
                                    &yid_str, quality,
                                )
                                .await
                                {
                                    Ok(url) => {
                                        let _ = tx.send(url);
                                    },
                                    Err(e) => log::error!("Failed to resolve YouTube stream: {e}"),
                                }
                            });

                            glib::idle_add_local(move || match rx.try_recv() {
                                Ok(stream_url) => {
                                    if let Some(ref p) = *player_rc.borrow() {
                                        p.play_uri(&stream_url);
                                    }
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
                            let uri = format!("file://{path}");
                            player.play_uri(&uri);
                            *self.player.borrow_mut() = Some(player);
                            self.play_btn.set_label("Pause");
                            self.vol_scale.set_value(0.8);
                        },
                    };

                    self.start_timer();
                },
                Ok(None) => {
                    self.video_title.set_text("Video not found.");
                    self.player_frame.set_child(Some(&self.status_page));
                },
                Err(e) => {
                    self.video_title.set_text(&format!("Error: {}", e));
                    self.player_frame.set_child(Some(&self.status_page));
                },
            }
        } else {
            self.video_title.set_text("No backend connected.");
            self.player_frame.set_child(Some(&self.status_page));
        }
    }

    pub fn stop(&self) {
        self.stop_timer();
        if let Some(ref p) = *self.player.borrow() {
            p.stop();
        }
        self.player_frame.set_child(Some(&self.status_page));
        self.play_btn.set_label("Play");
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
}
