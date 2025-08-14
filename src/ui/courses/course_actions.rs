use crate::types::Course;
use crate::ui::{Modal, confirmation_modal, toast_helpers, use_course_manager, use_form_manager};
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct CourseActionsProps {
    pub course: Course,
    pub edit_modal_open: bool,
    pub delete_modal_open: bool,
    pub on_edit_close: EventHandler<()>,
    pub on_delete_close: EventHandler<()>,
}

/// Course actions component handling modals and interactions
#[component]
pub fn CourseActions(props: CourseActionsProps) -> Element {
    let course_manager = use_course_manager();
    let course_name_form = use_form_manager(props.course.name.clone());

    // Handle course update
    let handle_update_course = {
        let course_manager = course_manager.clone();
        let on_edit_close = props.on_edit_close;
        let course_name_form = course_name_form.clone();
        let course_id = props.course.id;
        let original_name = props.course.name.clone();

        move |_| {
            let new_name = course_name_form.value.trim().to_string();
            if new_name.is_empty() {
                toast_helpers::error("Course name cannot be empty");
                return;
            }

            if new_name != original_name {
                course_manager.update_course.call((course_id, new_name));
            }

            on_edit_close.call(());
        }
    };

    // Handle course deletion
    let handle_delete_course = {
        let course_manager = course_manager.clone();
        let on_delete_close = props.on_delete_close;
        let course_id = props.course.id;

        Callback::new(move |_| {
            course_manager.delete_course.call(course_id);
            course_manager.refresh.call(());
            on_delete_close.call(());
        })
    };

    rsx! {
        // Edit Course Modal
        Modal {
            variant: crate::ui::components::modal::form_modal(rsx! {
                button {
                    class: "btn btn-ghost",
                    onclick: {
                        let on_edit_close = props.on_edit_close;
                        let course_name_form = course_name_form.clone();
                        move |_| {
                            on_edit_close.call(());
                            course_name_form.reset.call(());
                        }
                    },
                    "Cancel"
                }
                button {
                    class: "btn btn-primary",
                    onclick: handle_update_course,
                    disabled: !course_name_form.is_dirty || course_name_form.value.trim().is_empty(),
                    "Save Changes"
                }
            }),
            open: props.edit_modal_open,
            on_close: {
                let on_edit_close = props.on_edit_close;
                let course_name_form = course_name_form.clone();
                move |_| {
                    on_edit_close.call(());
                    course_name_form.reset.call(());
                }
            },
            title: "Edit Course".to_string(),
            div {
                class: "form-control w-full",
                label {
                    class: "label",
                    span { class: "label-text", "Course Name" }
                }
                input {
                    r#type: "text",
                    placeholder: "Enter course name",
                    class: "input input-bordered w-full",
                    value: course_name_form.value,
                    oninput: move |evt| course_name_form.set_value.call(evt.value()),
                }
            }
        }

        // Delete confirmation modal using unified Modal
        Modal {
            variant: confirmation_modal(
                format!("Are you sure you want to delete the course '{}'? This action cannot be undone.", props.course.name),
                "Delete Course",
                "Cancel",
                "error",
                Some(handle_delete_course),
                Some(Callback::new({
                    let on_delete_close = props.on_delete_close;
                    move |_| on_delete_close.call(())
                }))
            ),
            open: props.delete_modal_open,
            title: Some("Delete Course".to_string()),
            on_close: Some(props.on_delete_close),
        }
    }
}
#[derive(Props, PartialEq, Clone)]
pub struct ContentReorganizationModalsProps {
    pub course: Course,
    pub recluster_modal_open: bool,
    pub restore_order_modal_open: bool,
    pub manual_reorder_modal_open: bool,
    pub on_recluster_close: EventHandler<()>,
    pub on_restore_order_close: EventHandler<()>,
    pub on_manual_reorder_close: EventHandler<()>,
}

/// Content reorganization modals component
#[component]
pub fn ContentReorganizationModals(props: ContentReorganizationModalsProps) -> Element {
    let analytics_manager = crate::ui::hooks::use_analytics_manager();
    let course_manager = use_course_manager();

    // Handle re-clustering
    let handle_recluster = {
        let analytics_manager = analytics_manager.clone();
        let course_manager = course_manager.clone();
        let on_close = props.on_recluster_close;
        let course_id = props.course.id;

        Callback::new(move |_| {
            let analytics_manager = analytics_manager.clone();
            let course_manager = course_manager.clone();
            
            spawn(async move {
                toast_helpers::info("Re-clustering course content...");
                
                // Call the analytics manager to re-cluster the course
                analytics_manager.structure_course.call(course_id);
                course_manager.refresh.call(());
                
                toast_helpers::success("Course content has been re-clustered successfully!");
            });
            
            on_close.call(());
        })
    };

    // Handle restore original order
    let handle_restore_order = {
        let course_manager = course_manager.clone();
        let on_close = props.on_restore_order_close;
        let _course_id = props.course.id;

        Callback::new(move |_| {
            let _course_manager = course_manager.clone();
            
            spawn(async move {
                toast_helpers::info("Restoring original video order...");
                
                // This would need to be implemented in the backend
                // For now, we'll show a placeholder message
                toast_helpers::info("Restore original order functionality will be implemented in a future update");
            });
            
            on_close.call(());
        })
    };

    // Handle manual reorder
    let handle_manual_reorder = {
        let on_close = props.on_manual_reorder_close;

        Callback::new(move |_| {
            toast_helpers::info("Manual reorder functionality will be implemented in a future update");
            on_close.call(());
        })
    };

    rsx! {
        // Re-cluster confirmation modal
        Modal {
            variant: confirmation_modal(
                "Re-clustering will analyze your course content again and may change the current module organization. This action cannot be undone.".to_string(),
                "Re-cluster Content",
                "Cancel",
                "warning",
                Some(handle_recluster),
                Some(Callback::new({
                    let on_close = props.on_recluster_close;
                    move |_| on_close.call(())
                }))
            ),
            open: props.recluster_modal_open,
            title: Some("Re-cluster Course Content".to_string()),
            on_close: Some(props.on_recluster_close),
        }

        // Restore original order confirmation modal
        Modal {
            variant: confirmation_modal(
                "This will restore the course to its original video order, removing any clustering organization. This action cannot be undone.".to_string(),
                "Restore Original Order",
                "Cancel",
                "info",
                Some(handle_restore_order),
                Some(Callback::new({
                    let on_close = props.on_restore_order_close;
                    move |_| on_close.call(())
                }))
            ),
            open: props.restore_order_modal_open,
            title: Some("Restore Original Order".to_string()),
            on_close: Some(props.on_restore_order_close),
        }

        // Manual reorder modal (placeholder for future implementation)
        Modal {
            variant: crate::ui::components::modal::ModalVariant::Standard,
            open: props.manual_reorder_modal_open,
            title: Some("Manual Reorder".to_string()),
            on_close: Some(props.on_manual_reorder_close),
            div {
                class: "space-y-4",
                div {
                    class: "alert alert-info",
                    div {
                        class: "flex items-center gap-3",
                        span { class: "text-lg", "🚧" }
                        div {
                            span { class: "font-semibold", "Feature Coming Soon" }
                            p {
                                class: "text-sm opacity-80",
                                "Manual reorder functionality will allow you to drag and drop videos to customize your course structure."
                            }
                        }
                    }
                }
                div {
                    class: "flex justify-end gap-2",
                    button {
                        class: "btn btn-primary",
                        onclick: handle_manual_reorder,
                        "Got it"
                    }
                }
            }
        }
    }
}