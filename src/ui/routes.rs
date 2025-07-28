use dioxus::prelude::*;
use uuid::Uuid;

use crate::types::Route;
use crate::ui::layout::LayoutWrapper;

// Route components - these will be rendered by the router
#[component]
pub fn Home() -> Element {
    let navigator = use_navigator();

    // Redirect to dashboard
    use_effect(move || {
        navigator.push(Route::Dashboard {});
    });

    rsx! {
        LayoutWrapper {
            div { class: "p-8",
                "Redirecting to dashboard..."
            }
        }
    }
}

#[component]
pub fn Dashboard() -> Element {
    rsx! { 
        LayoutWrapper {
            crate::ui::dashboard::Dashboard {}
        }
    }
}

#[component]
pub fn PlanView(course_id: String) -> Element {
    // Enhanced route parameter handling with validation and fallbacks
    let navigator = use_navigator();
    let course_manager = crate::ui::hooks::use_course_manager();
    
    // Convert string to UUID with better error handling
    let course_uuid = match Uuid::parse_str(&course_id) {
        Ok(uuid) => uuid,
        Err(_) => {
            return render_invalid_course_id(&course_id, navigator);
        }
    };

    // Validate that the course exists
    let course_exists = course_manager.courses.iter().any(|c| c.id == course_uuid);
    
    if !course_exists {
        return render_course_not_found(&course_id, navigator);
    }

    rsx! { 
        LayoutWrapper {
            crate::ui::plan_view::PlanView { course_id: course_uuid }
        }
    }
}

/// Render invalid course ID error with navigation options
fn render_invalid_course_id(course_id: &str, navigator: Navigator) -> Element {
    let handle_go_back = move |_| {
        navigator.go_back();
    };
    
    let handle_go_dashboard = move |_| {
        navigator.push(Route::Dashboard {});
    };

    rsx! {
        LayoutWrapper {
            div { class: "min-h-screen flex items-center justify-center bg-base-100",
                div { class: "max-w-md mx-auto text-center p-8",
                    div { class: "mb-6",
                        div { class: "text-6xl mb-4", "🔗" }
                        h1 { class: "text-3xl font-bold mb-2 text-error", "Invalid Course Link" }
                        p { class: "text-base-content/70 mb-4", 
                            "The course ID '{course_id}' is not a valid format."
                        }
                        p { class: "text-sm text-base-content/50", 
                            "Course IDs should be in UUID format (e.g., 123e4567-e89b-12d3-a456-426614174000)"
                        }
                    }
                    
                    div { class: "flex flex-col sm:flex-row gap-3 justify-center",
                        button {
                            class: "btn btn-primary",
                            onclick: handle_go_dashboard,
                            "Go to Dashboard"
                        }
                        button {
                            class: "btn btn-outline",
                            onclick: handle_go_back,
                            "Go Back"
                        }
                    }
                }
            }
        }
    }
}

/// Render course not found error with navigation options
fn render_course_not_found(course_id: &str, navigator: Navigator) -> Element {
    let handle_go_back = move |_| {
        navigator.go_back();
    };
    
    let handle_go_dashboard = move |_| {
        navigator.push(Route::Dashboard {});
    };
    
    let handle_go_courses = move |_| {
        navigator.push(Route::AllCourses {});
    };

    rsx! {
        LayoutWrapper {
            div { class: "min-h-screen flex items-center justify-center bg-base-100",
                div { class: "max-w-md mx-auto text-center p-8",
                    div { class: "mb-6",
                        div { class: "text-6xl mb-4", "📚" }
                        h1 { class: "text-3xl font-bold mb-2 text-warning", "Course Not Found" }
                        p { class: "text-base-content/70 mb-4", 
                            "The course with ID '{course_id}' could not be found."
                        }
                        p { class: "text-sm text-base-content/50", 
                            "It may have been deleted or you may not have access to it."
                        }
                    }
                    
                    div { class: "flex flex-col sm:flex-row gap-3 justify-center",
                        button {
                            class: "btn btn-primary",
                            onclick: handle_go_courses,
                            "Browse All Courses"
                        }
                        button {
                            class: "btn btn-outline",
                            onclick: handle_go_dashboard,
                            "Go to Dashboard"
                        }
                        button {
                            class: "btn btn-ghost",
                            onclick: handle_go_back,
                            "Go Back"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AllCourses() -> Element {
    rsx! { 
        LayoutWrapper {
            crate::ui::courses::AllCoursesView {}
        }
    }
}

#[component]
pub fn Settings() -> Element {
    rsx! {
        LayoutWrapper {
            crate::ui::settings::SettingsView {}
        }
    }
}

#[component]
pub fn AddCourse() -> Element {
    rsx! {
        LayoutWrapper {
            div {
                class: "p-8",
                h1 { class: "text-3xl font-bold mb-4", "Add Course" }
                p { class: "text-base-content/70", "Add a new course to your collection." }
            }
        }
    }
}

#[cfg(debug_assertions)]
#[component]
pub fn ToastTest() -> Element {
    rsx! {
        LayoutWrapper {
            div {
                class: "p-8",
                h1 { class: "text-3xl font-bold mb-4", "Toast Test" }
                p { class: "text-base-content/70", "Test toast notifications." }
            }
        }
    }
}