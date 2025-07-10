//! Command Palette Modal Component
//! - DaisyUI modal pattern, Dioxus idioms
//! - Keyboard shortcut (Ctrl+K), ARIA/focus trap ready
//! - Action list, search/filter, arrow key navigation

// --- Command Palette implementation is commented out as per current plan ---
// To be enabled in Phase 3 (see implementation_checklist.md and plan.md)

// use dioxus::prelude::*;
// use dioxus_signals::*;
// use dioxus_free_icons::icons::fa_regular_icons::FaSearch;
// use dioxus_free_icons::Icon;
// use std::rc::Rc;

// /// Props for CommandPalette
// #[derive(PartialEq, Props, Clone)]
// pub struct CommandPaletteProps {
//     /// Whether the palette is open
//     pub open: bool,
//     /// List of actions (label, callback)
//     pub actions: Rc<Vec<CommandAction>>,
//     /// Optional: placeholder for search input
//     #[props(optional)]
//     pub placeholder: Option<String>,
//     /// Callback when closed (ESC or click outside)
//     pub on_close: EventHandler<()>,
// }

// /// Represents a command/action in the palette
// #[derive(Clone, PartialEq)]
// pub struct CommandAction {
//     pub label: String,
//     pub on_select: Rc<dyn Fn()>,
//     // Optionally: pub icon: Option<IconData>,
// }

// #[component]
// pub fn CommandPalette(
//     open: bool,
//     actions: Rc<Vec<CommandAction>>,
//     #[props(optional)] placeholder: Option<String>,
//     on_close: EventHandler<()>,
// ) -> Element {
//     // Local state: search query, selected index
//     let mut query = use_signal(String::new);
//     let mut selected = use_signal(|| 0);

//     // Filtered actions
//     let filtered: Vec<_> = actions
//         .iter()
//         .enumerate()
//         .filter(|(_, action)| {
//             let q = query().to_lowercase();
//             q.is_empty() || action.label.to_lowercase().contains(&q)
//         })
//         .collect();

//     // Keyboard navigation
//     use_effect(move || {
//         if !open {
//             return;
//         }
//         let handler = move |evt: KeyboardEvent| {
//             match evt.key().as_str() {
//                 "ArrowDown" => {
//                     let len = filtered.len();
//                     if len > 0 {
//                         selected.set((selected() + 1) % len);
//                     }
//                     evt.prevent_default();
//                 }
//                 "ArrowUp" => {
//                     let len = filtered.len();
//                     if len > 0 {
//                         selected.set((selected() + len - 1) % len);
//                     }
//                     evt.prevent_default();
//                 }
//                 "Enter" => {
//                     if let Some((_, action)) = filtered.get(selected()) {
//                         (action.on_select)();
//                         on_close.call(());
//                     }
//                     evt.prevent_default();
//                 }
//                 "Escape" => {
//                     on_close.call(());
//                     evt.prevent_default();
//                 }
//                 _ => {}
//             }
//         };
//         // Attach handler to window
//         let listener = dioxus::desktop::use_window_event("keydown", handler);
//         move || drop(listener)
//     });

//     // Focus trap: focus input on open
//     let input_ref = use_node_ref();
//     use_effect(move || {
//         if open {
//             if let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() {
//                 let _ = input.focus();
//             }
//         }
//     });

//     rsx! {
//         // DaisyUI modal pattern
//         if open {
//             dialog {
//                 id: "command_palette_modal",
//                 class: "modal modal-open",
//                 tabindex: 0,
//                 aria_label: "Command Palette",
//                 onkeydown: move |evt| {
//                     // Prevent propagation for handled keys
//                     match evt.key().as_str() {
//                         "ArrowDown" | "ArrowUp" | "Enter" | "Escape" => evt.stop_propagation(),
//                         _ => {}
//                     }
//                 },
//                 div {
//                     class: "modal-box p-0 max-w-xl w-full bg-base-100 shadow-xl",
//                     div {
//                         class: "flex items-center border-b px-4 py-2",
//                         Icon { icon: FaSearch, class: "w-5 h-5 text-base-content/70 mr-2" }
//                         input {
//                             r#ref: input_ref,
//                             class: "input input-bordered w-full bg-transparent focus:outline-none",
//                             r#type: "text",
//                             placeholder: placeholder.unwrap_or_else(|| "Type a command...".to_string()),
//                             value: "{query}",
//                             oninput: move |evt| query.set(evt.value()),
//                             autocomplete: "off",
//                             aria_label: "Search commands",
//                         }
//                     }
//                     ul {
//                         class: "menu menu-compact py-2 max-h-72 overflow-y-auto",
//                         if filtered.is_empty() {
//                             li { class: "text-base-content/60 px-4 py-2", "No commands found." }
//                         } else {
//                             for (idx, action) in filtered.iter() {
//                                 li {
//                                     class: format_args!(
//                                         "px-4 py-2 cursor-pointer {}",
//                                         if *selected() == idx { "bg-primary text-primary-content" } else { "" }
//                                     ),
//                                     tabindex: 0,
//                                     aria_selected: *selected() == idx,
//                                     onclick: move |_| {
//                                         (action.on_select)();
//                                         on_close.call(());
//                                     },
//                                     onmouseenter: move |_| selected.set(*idx),
//                                     "{action.label}"
//                                 }
//                             }
//                         }
//                     }
//                 }
//                 // Overlay to close
//                 form {
//                     method: "dialog",
//                     class: "modal-backdrop",
//                     onclick: move |_| on_close.call(()),
//                     // DaisyUI idiom: clicking backdrop closes modal
//                 }
//             }
//         }
//     }
// }
