use dioxus::prelude::*;

/// Modal state manager for clean modal handling
#[derive(Clone)]
pub struct ModalManager {
    pub is_open: bool,
    pub open: EventHandler<()>,
    pub close: EventHandler<()>,
    pub toggle: EventHandler<()>,
}

pub fn use_modal_manager(initial_state: bool) -> ModalManager {
    let is_open = use_signal(|| initial_state);

    let open = EventHandler::new({
        let mut is_open = is_open;
        move |_| is_open.set(true)
    });

    let close = EventHandler::new({
        let mut is_open = is_open;
        move |_| is_open.set(false)
    });

    let toggle = EventHandler::new({
        let mut is_open = is_open;
        move |_| is_open.set(!is_open())
    });

    ModalManager {
        is_open: is_open(),
        open,
        close,
        toggle,
    }
}

/// Form state manager for input handling
#[derive(Clone)]
pub struct FormManager<T: Clone + PartialEq + 'static> {
    pub value: T,
    pub set_value: EventHandler<T>,
    pub reset: EventHandler<()>,
    pub is_dirty: bool,
}

pub fn use_form_manager<T: Clone + PartialEq + 'static>(initial_value: T) -> FormManager<T> {
    let value = use_signal(|| initial_value.clone());
    let initial_ref = use_signal(|| initial_value);

    let set_value = EventHandler::new({
        let mut value = value;
        move |new_value: T| value.set(new_value)
    });

    let reset = EventHandler::new({
        let mut value = value;
        let initial_ref = initial_ref;
        move |_| value.set(initial_ref())
    });

    let is_dirty = value() != initial_ref();

    FormManager {
        value: value(),
        set_value,
        reset,
        is_dirty,
    }
}
