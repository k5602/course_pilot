use dioxus::prelude::*;
use crate::ui::hooks::use_show_toast;
use crate::ui::hooks::ToastVariant;
use std::rc::Rc;

#[component]
pub fn ToastTest() -> Element {
    rsx! {
        div {
            class: "flex flex-col space-y-2 p-4",
            h2 { class: "text-xl font-bold mb-4", "Test Toast Notifications" }
            
            button {
                class: "btn btn-success",
                onclick: move |_| {
                    let show_toast = use_show_toast();
                    show_toast("This is a success message!", ToastVariant::Success);
                },
                "Show Success Toast"
            }
            
            button {
                class: "btn btn-error",
                onclick: move |_| {
                    let show_toast = use_show_toast();
                    show_toast("This is an error message!", ToastVariant::Error);
                },
                "Show Error Toast"
            }
            
            button {
                class: "btn btn-info",
                onclick: move |_| {
                    let show_toast = use_show_toast();
                    show_toast("This is an info message!", ToastVariant::Info);
                },
                "Show Info Toast"
            }
            
            button {
                class: "btn btn-warning",
                onclick: move |_| {
                    let show_toast = use_show_toast();
                    show_toast("This is a warning message!", ToastVariant::Warning);
                },
                "Show Warning Toast"
            }
        }
    }
}
