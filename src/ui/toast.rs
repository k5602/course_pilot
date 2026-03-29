use std::cell::RefCell;

thread_local! {
    static OVERLAY: RefCell<Option<adw::ToastOverlay>> = const { RefCell::new(None) };
}

pub struct Toast;

impl Toast {
    pub fn init(overlay: &adw::ToastOverlay) {
        OVERLAY.with(|o| *o.borrow_mut() = Some(overlay.clone()));
    }

    pub fn show(message: &str) {
        OVERLAY.with(|o| {
            if let Some(ref overlay) = *o.borrow() {
                let toast = adw::Toast::new(message);
                overlay.add_toast(toast);
            }
        });
    }

    pub fn show_error(message: &str) {
        OVERLAY.with(|o| {
            if let Some(ref overlay) = *o.borrow() {
                let toast = adw::Toast::new(message);
                toast.set_priority(adw::ToastPriority::High);
                overlay.add_toast(toast);
            }
        });
    }
}
