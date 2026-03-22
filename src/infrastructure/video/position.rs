use std::cell::RefCell;
use std::rc::Weak;

pub struct PositionTracker {
    _source_id: glib::SourceId,
}

impl PositionTracker {
    pub fn new(
        player: Weak<RefCell<Option<super::player::VideoPlayer>>>,
        save_interval_secs: u32,
        on_position: impl Fn(u64) + 'static,
    ) -> Self {
        let _source_id = glib::timeout_add_seconds_local(save_interval_secs, move || {
            let p = match player.upgrade() {
                Some(p) => p,
                None => return glib::ControlFlow::Break,
            };
            let p = p.borrow();
            if let Some(ref player) = *p
                && let Some(pos) = player.position()
            {
                on_position(pos);
            }
            glib::ControlFlow::Continue
        });

        Self { _source_id }
    }
}
