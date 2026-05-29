use gst::glib::prelude::Cast;
use gst::prelude::{ElementExt, ElementExtManual, GstBinExt, GstObjectExt, ObjectExt};
use gtk::prelude::WidgetExt;
use std::sync::mpsc;

struct FrameData {
    mapped: gst::MappedBuffer<gst::buffer::Readable>,
    width: i32,
    height: i32,
    stride: usize,
}

pub struct VideoPlayer {
    pipeline: gst::Pipeline,
    playbin: gst::Element,
    picture: gtk::Picture,
    _bus_guard: gst::bus::BusWatchGuard,
    _frame_tx: mpsc::Sender<FrameData>,
}

impl VideoPlayer {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        gst::init()?;

        let pipeline = gst::Pipeline::new();
        let playbin = gst::ElementFactory::make("playbin")
            .build()
            .map_err(|e| format!("Failed to create playbin: {e}"))?;
        pipeline.add(&playbin)?;

        let picture = gtk::Picture::new();
        picture.set_hexpand(true);
        picture.set_vexpand(true);
        picture.set_can_shrink(false);
        picture.set_content_fit(gtk::ContentFit::Contain);

        // --- Video frame rendering via appsink ---
        let appsink_elem = gst::ElementFactory::make("appsink")
            .name("video_sink")
            .build()
            .map_err(|e| format!("Failed to create appsink: {e}"))?;

        let caps = gst_video::VideoCapsBuilder::new().format(gst_video::VideoFormat::Rgba).build();
        appsink_elem.set_property("caps", &caps);
        appsink_elem.set_property("sync", true);
        appsink_elem.set_property("max-buffers", 1u32);
        appsink_elem.set_property("drop", true);

        playbin.set_property("video-sink", &appsink_elem);

        let (frame_tx, frame_rx) = mpsc::channel::<FrameData>();
        let frame_tx_cb = frame_tx.clone();

        let appsink = appsink_elem
            .downcast::<gst_app::AppSink>()
            .map_err(|_| "Failed to cast appsink to AppSink")?;
        let callbacks = gst_app::AppSinkCallbacks::builder()
            .new_sample(move |appsink| {
                let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Error)?;
                let buffer = sample.buffer().ok_or(gst::FlowError::Error)?.to_owned();
                let caps = sample.caps().ok_or(gst::FlowError::Error)?;
                let info =
                    gst_video::VideoInfo::from_caps(caps).map_err(|_| gst::FlowError::Error)?;

                let mapped =
                    buffer.into_mapped_buffer_readable().map_err(|_| gst::FlowError::Error)?;

                let data = FrameData {
                    mapped,
                    width: info.width() as i32,
                    height: info.height() as i32,
                    stride: (info.width() as usize) * 4,
                };
                let _ = frame_tx_cb.send(data);
                Ok(gst::FlowSuccess::Ok)
            })
            .build();
        appsink.set_callbacks(callbacks);
        // --- End video frame rendering setup ---

        // Idle callback on main thread: render most recent frame only
        let picture_idle = picture.clone();
        glib::idle_add_local(move || {
            let mut latest: Option<FrameData> = None;
            while let Ok(data) = frame_rx.try_recv() {
                latest = Some(data);
            }
            if let Some(data) = latest {
                let bytes = glib::Bytes::from_owned(data.mapped);
                let texture = gtk::gdk::MemoryTexture::new(
                    data.width,
                    data.height,
                    gtk::gdk::MemoryFormat::R8g8b8a8,
                    &bytes,
                    data.stride,
                );
                picture_idle.set_paintable(Some(&texture));
            }
            glib::ControlFlow::Continue
        });

        let bus = pipeline.bus().ok_or("Pipeline has no bus")?;
        let _bus_guard = bus.add_watch_local(move |_, msg| {
            match msg.view() {
                gst::MessageView::Error(err) => {
                    log::error!(
                        "GStreamer error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                },
                gst::MessageView::Warning(warn) => {
                    log::warn!("GStreamer warning: {} ({:?})", warn.error(), warn.debug());
                },
                gst::MessageView::Eos(..) => {
                    log::info!("GStreamer end of stream");
                },
                _ => {},
            }
            gst::glib::ControlFlow::Continue
        })?;

        Ok(Self { pipeline, playbin, picture, _bus_guard, _frame_tx: frame_tx })
    }

    pub fn widget(&self) -> &gtk::Picture {
        &self.picture
    }

    pub fn play_file(&self, path: &str) {
        let uri = url::Url::from_file_path(path)
            .map(|u| u.to_string())
            .unwrap_or_else(|_| format!("file://{}", path));
        self.playbin.set_property("uri", &uri);
        if let Err(e) = self.pipeline.set_state(gst::State::Playing) {
            log::warn!("GStreamer state change to Playing failed: {:?}", e);
        }
    }

    pub fn play_uri(&self, uri: &str) {
        self.playbin.set_property("uri", uri);
        if let Err(e) = self.pipeline.set_state(gst::State::Playing) {
            log::warn!("GStreamer state change to Playing failed: {:?}", e);
        }
    }

    pub fn pause(&self) {
        if let Err(e) = self.pipeline.set_state(gst::State::Paused) {
            log::warn!("GStreamer state change to Paused failed: {:?}", e);
        }
    }

    pub fn resume(&self) {
        if let Err(e) = self.pipeline.set_state(gst::State::Playing) {
            log::warn!("GStreamer state change to Playing failed: {:?}", e);
        }
    }

    pub fn stop(&self) {
        if let Err(e) = self.pipeline.set_state(gst::State::Null) {
            log::warn!("GStreamer state change to Null failed: {:?}", e);
        }
    }

    pub fn seek(&self, pos_ns: u64) {
        if pos_ns == u64::MAX {
            return;
        }
        if let Err(e) =
            self.pipeline.seek_simple(gst::SeekFlags::FLUSH, gst::ClockTime::from_nseconds(pos_ns))
        {
            log::warn!("GStreamer seek failed: {:?}", e);
        }
    }

    pub fn position(&self) -> Option<u64> {
        self.pipeline
            .query_position::<gst::ClockTime>()
            .map(|t| t.nseconds())
            .filter(|&ns| ns != u64::MAX)
    }

    pub fn duration(&self) -> Option<u64> {
        self.pipeline
            .query_duration::<gst::ClockTime>()
            .map(|t| t.nseconds())
            .filter(|&ns| ns != u64::MAX)
    }

    pub fn set_volume(&self, vol: f64) {
        self.playbin.set_property("volume", vol.clamp(0.0, 1.0));
    }

    pub fn set_rate(&self, rate: f64) {
        self.playbin.set_property("rate", rate);
    }

    pub fn set_suburi(&self, uri: Option<&str>) {
        self.playbin.set_property("suburi", uri);
    }
}

impl Drop for VideoPlayer {
    fn drop(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}
