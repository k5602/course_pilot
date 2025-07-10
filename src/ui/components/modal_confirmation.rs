use dioxus::prelude::*;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaCheck, FaCow, FaExclamation, FaMagnifyingGlass, FaXmark,
};
use dioxus_free_icons::Icon;

// =======================
// Modal Confirmation
// =======================

#[derive(Props, PartialEq, Clone)]
pub struct ModalConfirmationProps {
    pub open: bool,
    pub title: String,
    pub message: String,
    #[props(optional)]
    pub confirm_label: Option<String>,
    #[props(optional)]
    pub cancel_label: Option<String>,
    #[props(optional)]
    pub confirm_color: Option<String>,
    #[props(optional)]
    pub on_confirm: Option<EventHandler<()>>,
    #[props(optional)]
    pub on_cancel: Option<EventHandler<()>>,
}

#[component]
pub fn ModalConfirmation(props: ModalConfirmationProps) -> Element {
    if !props.open {
        return rsx! {};
    }
    let confirm_label = props.confirm_label.clone().unwrap_or("Confirm".to_string());
    let cancel_label = props.cancel_label.clone().unwrap_or("Cancel".to_string());
    let confirm_color = props.confirm_color.clone().unwrap_or("primary".to_string());

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/40",
            tabindex: "-1",
            onclick: move |_| {
                if let Some(cb) = &props.on_cancel {
                    cb.call(());
                }
            },
            div {
                class: "modal-box bg-base-100 shadow-xl relative max-w-md w-full mx-4",
                onclick: move |evt| evt.stop_propagation(),
                h3 { class: "font-bold text-lg flex items-center gap-2 mb-2",
                    Icon { icon: FaExclamation, class: "text-warning w-5 h-5" }
                    "{props.title}"
                }
                p { class: "mb-4 text-base-content/80", "{props.message}" }
                div { class: "modal-action flex gap-2 justify-end",
                    button {
                        class: "btn btn-sm btn-ghost",
                        onclick: move |_| {
                            if let Some(cb) = &props.on_cancel {
                                cb.call(());
                            }
                        },
                        Icon { icon: FaXmark, class: "w-4 h-4" }
                        "{cancel_label}"
                    }
                    button {
                        class: format!("btn btn-sm btn-{}", confirm_color),
                        onclick: move |_| {
                            if let Some(cb) = &props.on_confirm {
                                cb.call(());
                            }
                        },
                        Icon { icon: FaCheck, class: "w-4 h-4" }
                        "{confirm_label}"
                    }
                }
            }
        }
    }
}

// =======================
// DropdownItem for ActionMenu
// =======================

#[derive(Props, PartialEq, Clone)]
pub struct DropdownItem {
    pub label: String,
    pub icon: Option<Element>,
    #[props(optional)]
    pub on_select: Option<EventHandler<()>>,
    pub children: Option<Vec<DropdownItem>>, // Must always be Option<Vec<DropdownItem>>, never RSX or function call
    #[props(optional)]
    pub disabled: bool,
}

// =======================
// ActionMenu (uses EnhancedDropdown pattern)
// =======================

#[component]
pub fn ActionMenu(actions: Vec<DropdownItem>, #[props(optional)] class: Option<String>) -> Element {
    let class = class.as_deref().unwrap_or("");
    let mut is_open = use_signal(|| false);
    let open_submenu = use_signal(|| None::<usize>);
    let selected = use_signal(|| 0);

    // Clone actions for closure use to avoid move issues
    let actions_for_closure = actions.clone();
    let actions_len = actions_for_closure.len();
    let onkeydown = {
        let mut selected = selected.clone();
        let mut open_submenu = open_submenu.clone();
        move |evt: dioxus::events::KeyboardEvent| {
            match evt.key().to_string().as_str() {
                "ArrowDown" => {
                    if actions_len > 0 {
                        selected.set((selected() + 1) % actions_len);
                    }
                }
                "ArrowUp" => {
                    if actions_len > 0 {
                        selected.set((selected() + actions_len - 1) % actions_len);
                    }
                }
                "ArrowRight" => {
                    open_submenu.set(Some(selected()));
                }
                "ArrowLeft" | "Escape" => {
                    open_submenu.set(None);
                }
                "Enter" => {
                    // If submenu, open it, else trigger
                    if let Some(item) = actions_for_closure.get(selected()) {
                        if let Some(children) = &item.children {
                            if !children.is_empty() {
                                open_submenu.set(Some(selected()));
                            }
                        } else if let Some(cb) = &item.on_select {
                            cb.call(());
                        }
                    }
                }
                _ => {}
            }
        }
    };

    rsx! {
        div { class: format!("relative inline-block {}", class),
            button {
                class: "btn btn-sm btn-outline flex items-center gap-2",
                onclick: move |_| is_open.set(!is_open()),
                Icon { icon: FaCow, class: "w-5 h-5" }
                span { class: "ml-1", "▼" }
            }
            if is_open() {
                div {
                    tabindex: "0",
                    onkeydown: onkeydown,
                    style: "outline: none;",
                    {
                        // Move render_menu inside the component so it can close over selected
                        fn render_menu(
                            items: &Vec<DropdownItem>,
                            selected: Signal<usize>,
                            open_submenu: Option<usize>,
                            open_submenu_signal: &Signal<Option<usize>>,
                            depth: usize,
                        ) -> Element {
                            let selected_idx = selected();
                            rsx! {
                                ul {
                                    class: format!(
                                        "menu menu-compact bg-base-200 rounded shadow-lg z-50 min-w-[10rem] absolute {} mt-2",
                                        if depth == 0 { "right-0" } else { "left-full top-0 ml-2" }
                                    ),
                                    {
                                        items.iter().enumerate().map(|(idx, item)| {
                                            let has_children = item.children.as_ref().map_or(false, |c| !c.is_empty());
                                            let is_selected = idx == selected_idx;
                                            let is_open = open_submenu == Some(idx);
                                            rsx! {
                                                li {
                                                    class: format!(
                                                        "flex items-center gap-2 px-2 py-2 rounded cursor-pointer group relative {} {}",
                                                        if item.disabled { "opacity-50 pointer-events-none" } else { "hover:bg-base-300" },
                                                        if is_selected { "bg-primary text-primary-content" } else { "" }
                                                    ),
                                                    tabindex: "0",
                                                    onclick: {
                                                        let cb = item.on_select.clone();
                                                        let mut open_submenu_signal = open_submenu_signal.clone();
                                                        move |_| {
                                                            if has_children {
                                                                open_submenu_signal.set(Some(idx));
                                                            } else if let Some(f) = cb {
                                                                f.call(());
                                                            }
                                                        }
                                                    },
                                                    if let Some(icon) = &item.icon {
                                                        {icon.clone()}
                                                    }
                                                    span { class: "font-medium", "{item.label}" }
                                                    if has_children {
                                                        span { class: "ml-auto text-xs opacity-60", "▶" }
                                                        if is_open {
                                                            if let Some(children) = &item.children {
                                                                {render_menu(children, selected.clone(), None, open_submenu_signal, depth + 1)}
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        })
                                    }
                                }
                            }
                        }
                        render_menu(&actions, selected.clone(), open_submenu(), &open_submenu, 0)
                    }
                }
            }
        }
    }
}

// =======================
// Command Palette
// =======================

#[derive(Props, PartialEq, Clone)]
/// Command Palette Action for modal
pub struct CommandAction {
    pub label: String,
    #[props(optional)]
    pub on_select: Option<EventHandler<()>>,
    #[props(optional)]
    pub description: Option<String>,
    #[props(optional)]
    pub icon: Option<Element>,
}

/// Command Palette Modal (Dioxus idioms, DaisyUI modal, ARIA/keyboard/focus)
#[component]
pub fn CommandPalette(
    open: bool,
    actions: Vec<CommandAction>,
    #[props(optional)] placeholder: Option<String>,
    #[props(optional)] on_close: Option<EventHandler<()>>,
) -> Element {
    let mut query = use_signal(|| String::new());
    let selected = use_signal(|| 0);

    let actions_for_filter = actions.clone();
    let query_for_filter = query.clone();
    let filtered: Vec<_> = actions_for_filter
        .iter()
        .enumerate()
        .filter(|(_, action)| {
            let q = query_for_filter().to_lowercase();
            q.is_empty() || action.label.to_lowercase().contains(&q)
        })
        .collect();

    let onkeydown = {
        let selected = selected.clone();
        let actions = actions.clone();
        let query = query.clone();
        let on_close = on_close.clone();
        move |evt: dioxus::events::KeyboardEvent| {
            let mut selected = selected.clone();
            // Recompute filtered inside closure to avoid lifetime issues
            let filtered: Vec<_> = actions
                .iter()
                .enumerate()
                .filter(|(_, action)| {
                    let q = query().to_lowercase();
                    q.is_empty() || action.label.to_lowercase().contains(&q)
                })
                .collect();
            match evt.key().to_string().as_str() {
                "ArrowDown" => {
                    if filtered.len() > 0 {
                        selected.set((selected() + 1) % filtered.len());
                    }
                }
                "ArrowUp" => {
                    if filtered.len() > 0 {
                        selected.set((selected() + filtered.len() - 1) % filtered.len());
                    }
                }
                "Enter" => {
                    if let Some((_, action)) = filtered.get(selected()) {
                        if let Some(cb) = &action.on_select {
                            cb.call(());
                        }
                        if let Some(close) = &on_close {
                            close.call(());
                        }
                    }
                }
                "Escape" => {
                    if let Some(close) = &on_close {
                        close.call(());
                    }
                }
                _ => {}
            }
        }
    };

    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/40",
            tabindex: "-1",
            onclick: move |_| {
                if let Some(cb) = &on_close {
                    cb.call(());
                }
            },
            div {
                class: "modal-box bg-base-100 shadow-xl relative max-w-lg w-full mx-4 p-0",
                onclick: move |evt| evt.stop_propagation(),
                div { class: "flex items-center gap-2 p-4 border-b border-base-300",
                    Icon { icon: FaMagnifyingGlass, class: "w-5 h-5 opacity-60" }
                    input {
                        class: "input input-sm flex-1 bg-base-200",
                        r#type: "text",
                        placeholder: placeholder.clone().unwrap_or_else(|| "Type a command...".to_string()),
                        value: "{query()}",
                        oninput: move |evt| query.set(evt.value().clone()),
                        onkeydown: onkeydown,
                    }
                }
                ul { class: "menu menu-compact w-full max-h-64 overflow-y-auto p-2",
                    if filtered.is_empty() {
                        li { class: "text-base-content/60 px-2 py-1", "No commands found." }
                    } else {
                        {
                            filtered.iter().enumerate().map(|(idx, (_, action))| {
                                let is_selected = idx == selected();
                                rsx! {
                                    li {
                                        class: format!(
                                            "flex items-center gap-2 px-2 py-2 rounded cursor-pointer {}",
                                            if is_selected { "bg-primary text-primary-content" } else { "hover:bg-base-200" }
                                        ),
                                        tabindex: "0",
                                        onclick: {
                                            let cb = action.on_select.clone();
                                            let on_close = on_close.clone();
                                            move |_| {
                                                if let Some(f) = cb {
                                                    f.call(());
                                                }
                                                if let Some(close) = &on_close {
                                                    close.call(());
                                                }
                                            }
                                        },
                                        if let Some(icon) = &action.icon {
                                            {icon.clone()}
                                        }
                                        span { class: "font-medium", "{action.label}" }
                                        if let Some(desc) = &action.description {
                                            span { class: "ml-2 text-xs opacity-60", "{desc}" }
                                        }
                                    }
                                }
                            })
                        }
                    }
                }
            }
        }
    }
}

// =======================
// Advanced Tabs (closeable, dynamic)
// =======================

#[derive(Props, PartialEq, Clone)]
pub struct TabData {
    pub label: String,
    pub content: Element,
    #[props(optional)]
    pub closable: bool,
}

use std::rc::Rc;
#[component]
pub fn AdvancedTabs(
    tabs: Vec<TabData>,
    selected: usize,
    on_select: EventHandler<usize>,
    #[props(optional)] on_close: Option<EventHandler<usize>>,
    #[props(optional)] class: Option<String>,
) -> Element {
    use std::collections::HashSet;
    let class = class.as_deref().unwrap_or("tabs tabs-boxed");
    let mut mounted_tabs = use_signal(|| HashSet::new());

    // Mark the selected tab as mounted
    {
        let mut set = mounted_tabs.write();
        set.insert(selected);
    }

    let tabs_rc = Rc::new(tabs);
    let on_select_clone = on_select.clone();
    let on_close_clone = on_close.clone();
    let tabs_for_keydown = tabs_rc.clone();
    // Keyboard navigation handler for tabs
    let onkeydown = move |evt: dioxus::events::KeyboardEvent| {
        let tabs = tabs_for_keydown.clone();
        let on_select = on_select_clone.clone();
        let on_close = on_close_clone.clone();
        let tabs_len = tabs.len();
        match evt.key().to_string().as_str() {
            "ArrowRight" => {
                if tabs_len > 0 {
                    on_select.call((selected + 1) % tabs_len);
                }
            }
            "ArrowLeft" => {
                if tabs_len > 0 {
                    on_select.call((selected + tabs_len - 1) % tabs_len);
                }
            }
            "Delete" | "Backspace" => {
                if let Some(cb) = &on_close {
                    if tabs.get(selected).map(|t| t.closable).unwrap_or(false) {
                        cb.call(selected);
                    }
                }
            }
            "w" | "W" => {
                if evt.modifiers().contains(dioxus::events::Modifiers::CONTROL) {
                    if let Some(cb) = &on_close {
                        if tabs.get(selected).map(|t| t.closable).unwrap_or(false) {
                            cb.call(selected);
                        }
                    }
                }
            }
            "Enter" | " " => {
                on_select.call(selected);
            }
            _ => {}
        }
    };

    rsx! {
        div {
            class: "{class}",
            tabindex: "0",
            onkeydown: onkeydown,
            {
                tabs_rc.iter().enumerate().map(|(idx, tab)| {
                    let tab_class = if idx == selected { "tab tab-active flex items-center gap-2" } else { "tab flex items-center gap-2" };
                    rsx! {
                        div { class: "relative flex items-center",
                            button {
                                key: "{idx}",
                                class: "{tab_class}",
                                onclick: move |_| on_select.call(idx),
                                "{tab.label}"
                                if tab.closable {
                                    button {
                                        class: "ml-1 btn btn-xs btn-circle btn-ghost absolute -right-2 top-1",
                                        onclick: move |evt| {
                                            evt.stop_propagation();
                                            if let Some(cb) = &on_close {
                                                cb.call(idx);
                                            }
                                        },
                                        Icon { icon: FaXmark, class: "w-3 h-3" }
                                    }
                                }
                            }
                        }
                    }
                })
            }
        }
        div { class: "mt-4",
            for (idx, tab) in tabs_rc.iter().enumerate() {
                if mounted_tabs.read().contains(&idx) {
                    if idx == selected {
                        {tab.content.clone()}
                    } else {
                        div { style: "display: none;", {tab.content.clone()} }
                    }
                }
            }
        }
    }
}

// =======================
// Circular/Ring Progress Indicator
// =======================

#[component]
pub fn CircularProgress(
    value: u8,
    #[props(optional)] size: Option<u32>,
    #[props(optional)] color: Option<String>,
    #[props(optional)] label: Option<String>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let value = value.clamp(0, 100);
    let size = size.unwrap_or(48);
    let color = color.as_deref().unwrap_or("primary");
    let extra_class = class.as_deref().unwrap_or("");
    let radius = (size as f32) / 2.0 - 6.0;
    let circumference = 2.0 * std::f32::consts::PI * radius;
    let offset = circumference * (1.0 - (value as f32 / 100.0));

    rsx! {
        div { class: "flex flex-col items-center gap-1 {extra_class}",
            svg {
                width: "{size}",
                height: "{size}",
                view_box: format!("0 0 {} {}", size, size),
                class: "block",
                circle {
                    cx: size/2,
                    cy: size/2,
                    r: radius,
                    fill: "none",
                    stroke: "#e5e7eb",
                    stroke_width: "6",
                }
                circle {
                    cx: size/2,
                    cy: size/2,
                    r: radius,
                    fill: "none",
                    stroke: "var(--color-primary, #2563eb)",
                    stroke_width: "6",
                    stroke_dasharray: format!("{}", circumference),
                    stroke_dashoffset: format!("{}", offset),
                    stroke_linecap: "round",
                    style: "transition: stroke-dashoffset 0.4s cubic-bezier(.4,2,.6,1);"
                }
                text {
                    x: "50%",
                    y: "50%",
                    text_anchor: "middle",
                    dy: ".3em",
                    font_size: (size as f32 * 0.32).round(),
                    fill: "#374151",
                    "{value}%"
                }
            }
            if let Some(label) = &label {
                span { class: "text-xs opacity-70", "{label}" }
            }
        }
    }
}

// =======================
// Badge component
// =======================

#[component]
pub fn Badge(
    label: String,
    #[props(optional)] color: Option<String>,
    #[props(optional)] class: Option<String>,
) -> Element {
    let color = color.as_deref().unwrap_or("primary");
    let class = class.as_deref().unwrap_or("");
    rsx! {
        span {
            class: format!("badge badge-{} {}", color, class),
            "{label}"
        }
    }
}
