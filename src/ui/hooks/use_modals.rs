use dioxus::prelude::*;

/// Modal state manager for clean modal handling
#[derive(Clone)]
pub struct ModalManager {
    pub is_open: bool,
    pub open: Callback<()>,
    pub close: Callback<()>,
    pub toggle: Callback<()>,
}

pub fn use_modal_manager(initial_state: bool) -> ModalManager {
    let mut is_open = use_signal(|| initial_state);

    let open = use_callback(move |_| is_open.set(true));
    let close = use_callback(move |_| is_open.set(false));
    let toggle = use_callback(move |_| is_open.set(!is_open()));

    ModalManager { is_open: is_open(), open, close, toggle }
}

/// Form state manager for input handling
#[derive(Clone)]
pub struct FormManager<T: Clone + PartialEq + 'static> {
    pub value: T,
    pub set_value: Callback<T>,
    pub reset: Callback<()>,
    pub is_dirty: bool,
}

pub fn use_form_manager<T: Clone + PartialEq + 'static>(initial_value: T) -> FormManager<T> {
    let mut value = use_signal(|| initial_value.clone());
    let initial_ref = use_signal(|| initial_value);

    let set_value = use_callback(move |new_value: T| value.set(new_value));
    let reset = use_callback(move |_| value.set(initial_ref()));

    let is_dirty = value() != initial_ref();

    FormManager { value: value(), set_value, reset, is_dirty }
}
