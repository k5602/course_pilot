//! Course card for dashboard

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::ui::Route;

/// A card displaying course info on the dashboard.
#[component]
pub fn CourseCard(
    id: String,
    name: String,
    module_count: usize,
    completed_modules: usize,
) -> Element {
    let opacity = use_motion(0.0f32);
    let y_offset = use_motion(12.0f32);
    let mut opacity_for_effect = opacity;
    let mut y_offset_for_effect = y_offset;

    use_effect(move || {
        let config = AnimationConfig::new(AnimationMode::Spring(Spring {
            stiffness: 120.0,
            damping: 16.0,
            mass: 0.8,
            velocity: 0.0,
        }));
        opacity_for_effect.animate_to(1.0, config.clone());
        y_offset_for_effect.animate_to(0.0, config);
    });

    let progress = if module_count > 0 {
        (completed_modules as f64 / module_count as f64 * 100.0) as u8
    } else {
        0
    };

    rsx! {
        Link {
            to: Route::CourseView { course_id: id },
            class: "card bg-base-200 hover:bg-base-300 transition-colors cursor-pointer",
            style: "opacity: {opacity.get_value()}; transform: translateY({y_offset.get_value()}px); will-change: transform, opacity;",

            div { class: "card-body",

                h3 { class: "card-title text-lg", "{name}" }

                p { class: "text-sm text-base-content/70",
                    "{completed_modules} / {module_count} modules"
                }

                // Progress bar
                div { class: "w-full bg-base-300 rounded-full h-2 mt-2",
                    div {
                        class: "bg-primary h-2 rounded-full transition-all",
                        style: "width: {progress}%",
                    }
                }
            }
        }
    }
}
