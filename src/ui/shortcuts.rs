use std::rc::Rc;

use gtk::prelude::*;

use crate::ui::navigation::{
    PAGE_COURSE_LIST, PAGE_COURSE_VIEW, PAGE_QUIZ_LIST, PAGE_QUIZ_VIEW, PAGE_VIDEO_PLAYER,
};

pub fn setup_shortcuts(window: &gtk::Window, stack: Rc<gtk::Stack>) {
    let controller = gtk::EventControllerKey::new();

    controller.connect_key_pressed(move |_, keyval, _code, _state| match keyval {
        gtk::gdk::Key::Escape => {
            let current = stack.visible_child_name().unwrap_or_default();
            match current.as_str() {
                PAGE_COURSE_VIEW => stack.set_visible_child_name(PAGE_COURSE_LIST),
                PAGE_VIDEO_PLAYER => stack.set_visible_child_name(PAGE_COURSE_VIEW),
                PAGE_QUIZ_VIEW => stack.set_visible_child_name(PAGE_QUIZ_LIST),
                _ => {},
            }
            glib::Propagation::Stop
        },
        _ => glib::Propagation::Proceed,
    });

    window.add_controller(controller);
}
