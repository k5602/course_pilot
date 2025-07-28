use crate::types::Course;
use crate::ui::{
    ImportModal, ImportSource, ImportSettings, toast_helpers,
    use_course_manager, use_modal_manager, use_search_state,
    use_debounced_state,
};
use dioxus::prelude::*;
use dioxus_free_icons::Icon;
use dioxus_free_icons::icons::fa_solid_icons::{
    FaPlus, FaMagnifyingGlass, FaFilter, FaSort, FaCircleExclamation,
};

use super::CourseGrid;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CourseFilter {
    All,
    NotStarted,
    InProgress,
    Completed,
    Structured,
    Unstructured,
}

impl CourseFilter {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseFilter::All => "All Courses",
            CourseFilter::NotStarted => "Not Started",
            CourseFilter::InProgress => "In Progress", 
            CourseFilter::Completed => "Completed",
            CourseFilter::Structured => "Structured",
            CourseFilter::Unstructured => "Unstructured",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CourseSortBy {
    Name,
    DateCreated,
    Progress,
    LastAccessed,
}

impl CourseSortBy {
    pub fn as_str(&self) -> &'static str {
        match self {
            CourseSortBy::Name => "Name",
            CourseSortBy::DateCreated => "Date Created",
            CourseSortBy::Progress => "Progress",
            CourseSortBy::LastAccessed => "Last Accessed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Comprehensive All Courses view with filtering, searching, and sorting
#[component]
pub fn AllCoursesView() -> Element {
    let course_manager = use_course_manager();
    let import_modal = use_modal_manager(false);
    
    // Search and filter state
    let (search_query, set_search_query) = use_search_state();
    let (_, debounced_search, _) = use_debounced_state(search_query(), 300);
    let mut current_filter = use_signal(|| CourseFilter::All);
    let sort_by = use_signal(|| CourseSortBy::Name);
    let sort_order = use_signal(|| SortOrder::Ascending);


    // Get courses from course manager
    let courses = course_manager.courses.clone();
    let is_loading = course_manager.is_loading;
    let error = course_manager.error.clone();

    // Filter and sort courses
    let filtered_and_sorted_courses = {
        let mut filtered_courses: Vec<Course> = courses
            .iter()
            .filter(|course| {
                // Apply search filter
                let search_query_text = debounced_search().to_lowercase();
                let matches_search = if search_query_text.is_empty() {
                    true
                } else {
                    course.name.to_lowercase().contains(&search_query_text)
                        || course.raw_titles.iter().any(|title| 
                            title.to_lowercase().contains(&search_query_text)
                        )
                };

                if !matches_search {
                    return false;
                }

                // Apply status filter
                match current_filter() {
                    CourseFilter::All => true,
                    CourseFilter::NotStarted => {
                        // For now, consider courses without structure as not started
                        course.structure.is_none()
                    },
                    CourseFilter::InProgress => {
                        // For now, consider structured courses as in progress
                        course.structure.is_some()
                    },
                    CourseFilter::Completed => {
                        // For now, no courses are considered completed
                        // This would need actual progress tracking
                        false
                    },
                    CourseFilter::Structured => course.structure.is_some(),
                    CourseFilter::Unstructured => course.structure.is_none(),
                }
            })
            .cloned()
            .collect();

        // Sort courses
        match sort_by() {
            CourseSortBy::Name => {
                filtered_courses.sort_by(|a, b| {
                    let cmp = a.name.cmp(&b.name);
                    match sort_order() {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            },
            CourseSortBy::DateCreated => {
                filtered_courses.sort_by(|a, b| {
                    let cmp = a.created_at.cmp(&b.created_at);
                    match sort_order() {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            },
            CourseSortBy::Progress => {
                // Sort by structure status as a proxy for progress
                // Structured courses are considered more progressed
                filtered_courses.sort_by(|a, b| {
                    let a_progress = if a.structure.is_some() { 1 } else { 0 };
                    let b_progress = if b.structure.is_some() { 1 } else { 0 };
                    let cmp = a_progress.cmp(&b_progress);
                    match sort_order() {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            },
            CourseSortBy::LastAccessed => {
                // Sort by creation date as fallback for last accessed
                filtered_courses.sort_by(|a, b| {
                    let cmp = a.created_at.cmp(&b.created_at);
                    match sort_order() {
                        SortOrder::Ascending => cmp,
                        SortOrder::Descending => cmp.reverse(),
                    }
                });
            },
        }

        filtered_courses
    };

    // Handle import completion
    let handle_import_complete = {
        let course_manager = course_manager.clone();
        let import_modal = import_modal.clone();
        
        EventHandler::new(move |_| {
            course_manager.refresh.call(());
            import_modal.close.call(());
            toast_helpers::success("Course imported successfully!");
        })
    };

    // Toggle sort order when clicking same sort option
    let mut handle_sort_change = {
        let mut sort_by = sort_by;
        let mut sort_order = sort_order;
        
        move |new_sort: CourseSortBy| {
            if sort_by() == new_sort {
                // Toggle order if same sort option
                sort_order.set(match sort_order() {
                    SortOrder::Ascending => SortOrder::Descending,
                    SortOrder::Descending => SortOrder::Ascending,
                });
            } else {
                // Set new sort option with ascending order
                sort_by.set(new_sort);
                sort_order.set(SortOrder::Ascending);
            }
        }
    };

    rsx! {
        div { class: "w-full max-w-7xl mx-auto px-4 py-8",
            // Header with title and import button
            div { class: "flex items-center justify-between mb-8",
                div {
                    h1 { class: "text-3xl font-bold", "All Courses" }
                    p { class: "text-sm text-base-content/70 mt-1",
                        "Manage your course collection with filtering, search, and sorting"
                    }
                }
                
                button {
                    class: "btn btn-primary gap-2",
                    onclick: move |_| import_modal.open.call(()),
                    Icon { icon: FaPlus, class: "w-4 h-4" }
                    "Import Course"
                }
            }

            // Search and filter controls
            div { class: "bg-base-200 rounded-lg p-6 mb-6",
                div { class: "grid grid-cols-1 lg:grid-cols-3 gap-4",
                    // Search input
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Search Courses" }
                        }
                        div { class: "relative",
                            input {
                                r#type: "text",
                                placeholder: "Search by name or content...",
                                class: "input input-bordered w-full pl-10",
                                value: search_query(),
                                oninput: move |evt| set_search_query.call(evt.value()),
                            }
                            Icon { 
                                icon: FaMagnifyingGlass, 
                                class: "absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-base-content/50" 
                            }
                        }
                    }

                    // Filter dropdown
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Filter by Status" }
                        }
                        div { class: "dropdown dropdown-bottom w-full",
                            div { 
                                tabindex: "0",
                                role: "button",
                                class: "btn btn-outline w-full justify-between",
                                Icon { icon: FaFilter, class: "w-4 h-4" }
                                span { "{current_filter().as_str()}" }
                                span { class: "text-xs", "▼" }
                            }
                            ul { 
                                tabindex: "0",
                                class: "dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full",
                                for filter in [
                                    CourseFilter::All,
                                    CourseFilter::NotStarted,
                                    CourseFilter::InProgress,
                                    CourseFilter::Completed,
                                    CourseFilter::Structured,
                                    CourseFilter::Unstructured,
                                ] {
                                    {
                                        let is_active = current_filter() == filter;
                                        rsx! {
                                            li { key: "{filter:?}",
                                                a {
                                                    class: if is_active { "active" } else { "" },
                                                    onclick: move |_| current_filter.set(filter),
                                                    "{filter.as_str()}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Sort dropdown
                    div { class: "form-control",
                        label { class: "label",
                            span { class: "label-text font-medium", "Sort by" }
                        }
                        div { class: "dropdown dropdown-bottom w-full",
                            div { 
                                tabindex: "0",
                                role: "button",
                                class: "btn btn-outline w-full justify-between",
                                Icon { icon: FaSort, class: "w-4 h-4" }
                                span { 
                                    "{sort_by().as_str()}"
                                    if sort_order() == SortOrder::Descending { " ↓" } else { " ↑" }
                                }
                                span { class: "text-xs", "▼" }
                            }
                            ul { 
                                tabindex: "0",
                                class: "dropdown-content z-[1] menu p-2 shadow bg-base-100 rounded-box w-full",
                                for sort_option in [
                                    CourseSortBy::Name,
                                    CourseSortBy::DateCreated,
                                    CourseSortBy::Progress,
                                    CourseSortBy::LastAccessed,
                                ] {
                                    {
                                        let is_active = sort_by() == sort_option;
                                        rsx! {
                                            li { key: "{sort_option:?}",
                                                a {
                                                    class: if is_active { "active" } else { "" },
                                                    onclick: move |_| handle_sort_change(sort_option),
                                                    "{sort_option.as_str()}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Results summary
                div { class: "flex items-center justify-between mt-4 pt-4 border-t border-base-300",
                    div { class: "text-sm text-base-content/70",
                        "Showing {filtered_and_sorted_courses.len()} of {courses.len()} courses"
                    }
                    
                    if !search_query().is_empty() {
                        button {
                            class: "btn btn-ghost btn-xs",
                            onclick: move |_| set_search_query.call(String::new()),
                            "Clear search"
                        }
                    }
                }
            }

            // Course content
            if is_loading {
                div { class: "flex justify-center items-center py-12",
                    span { class: "loading loading-spinner loading-lg" }
                    span { class: "ml-3 text-base-content/70", "Loading courses..." }
                }
            } else if let Some(error_msg) = error {
                div { class: "alert alert-error",
                    Icon { icon: FaCircleExclamation, class: "w-5 h-5" }
                    span { "Error loading courses: {error_msg}" }
                    button {
                        class: "btn btn-sm btn-ghost",
                        onclick: move |_| course_manager.refresh.call(()),
                        "Retry"
                    }
                }
            } else if filtered_and_sorted_courses.is_empty() {
                div { class: "text-center py-12",
                    {if courses.is_empty() {
                        // No courses at all
                        rsx! {
                            div { class: "max-w-md mx-auto",
                                Icon { icon: FaPlus, class: "w-16 h-16 mx-auto text-base-content/30 mb-4" }
                                h3 { class: "text-xl font-semibold mb-2", "No courses yet" }
                                p { class: "text-base-content/70 mb-6",
                                    "Get started by importing your first course from YouTube or a local folder."
                                }
                                button {
                                    class: "btn btn-primary gap-2",
                                    onclick: move |_| import_modal.open.call(()),
                                    Icon { icon: FaPlus, class: "w-4 h-4" }
                                    "Import Your First Course"
                                }
                            }
                        }
                    } else {
                        // No courses match current filter/search
                        rsx! {
                            div { class: "max-w-md mx-auto",
                                Icon { icon: FaMagnifyingGlass, class: "w-16 h-16 mx-auto text-base-content/30 mb-4" }
                                h3 { class: "text-xl font-semibold mb-2", "No courses found" }
                                p { class: "text-base-content/70 mb-6",
                                    "Try adjusting your search terms or filters to find what you're looking for."
                                }
                                div { class: "flex gap-2 justify-center",
                                    {if !search_query().is_empty() {
                                        rsx! {
                                            button {
                                                class: "btn btn-outline btn-sm",
                                                onclick: move |_| set_search_query.call(String::new()),
                                                "Clear search"
                                            }
                                        }
                                    } else {
                                        rsx! { }
                                    }}
                                    {if current_filter() != CourseFilter::All {
                                        rsx! {
                                            button {
                                                class: "btn btn-outline btn-sm",
                                                onclick: move |_| current_filter.set(CourseFilter::All),
                                                "Show all courses"
                                            }
                                        }
                                    } else {
                                        rsx! { }
                                    }}
                                }
                            }
                        }
                    }}
                }
            } else {
                // Course grid
                CourseGrid {
                    courses: filtered_and_sorted_courses,
                }
            }

            // Import Modal
            ImportModal {
                open: import_modal.is_open,
                on_close: move |_| import_modal.close.call(()),
                on_import: move |(source, _input, _settings): (ImportSource, String, ImportSettings)| {
                    // Handle import logic here
                    toast_helpers::info(format!("Starting import from {}", source.as_str()));
                    // The actual import logic would be handled by the import manager
                },
                on_course_imported: Some(handle_import_complete),
            }
        }
    }
}