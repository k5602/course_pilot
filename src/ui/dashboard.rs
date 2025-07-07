use crate::ui::components::course_card::CourseCard;
use crate::ui::hooks::use_courses;
use dioxus::prelude::*;

/// Dashboard: Responsive grid of CourseCards, wired to AppState/backend
#[component]
pub fn Dashboard() -> Element {
    let courses = use_courses();

    // Simulate async loading state (replace with real async logic as needed)
    let is_loading = false; // Set to true to simulate loading
    let has_error = false; // Set to true to simulate error

    let courses_guard = courses.read();

    rsx! {
        section {
            class: "w-full max-w-7xl mx-auto px-4 py-8",
            h1 { class: "text-2xl font-bold mb-6", "Your Courses" }
            if has_error {
                div {
                    class: "flex flex-col items-center justify-center py-12 text-error",
                    "Failed to load courses. Please try again."
                }
            } else if is_loading {
                div {
                    class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
                    {(0..3).map(|_| rsx! {
                        div {
                            class: "card bg-base-200 shadow-xl animate-pulse",
                            div { class: "card-body pb-4",
                                div { class: "h-6 w-2/3 bg-base-300 rounded mb-2" }
                                div { class: "h-4 w-1/2 bg-base-300 rounded mb-2" }
                                div { class: "h-2 w-full bg-base-300 rounded mb-2" }
                                div { class: "h-8 w-1/d bg-base-300 rounded mt-4" }
                            }
                        }
                    })}
                }
            } else if courses_guard.is_empty() {
                div {
                    class: "flex flex-col items-center justify-center py-12 text-base-content/60",
                    "No courses found. Click 'Add New Course' to get started."
                }
            } else {
                div {
                    class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
                    {courses_guard.iter().map(|course| rsx! {
                        CourseCard {
                            id: 0, // Use a unique integer if available, or 0 as placeholder
                            title: course.name.clone(),
                            video_count: course.raw_titles.len(),
                            total_duration: course.structure.as_ref().map(|s| {
                                let secs = s.aggregate_total_duration().as_secs();
                                let hours = secs / 3600;
                                let mins = (secs % 3600) / 60;
                                format!("{}h {}m", hours, mins)
                            }).unwrap_or_else(|| "N/A".to_string()),
                            progress: 0.0, // TODO: wire up real progress
                        }
                    })}
                }
            }
        }
    }
}
