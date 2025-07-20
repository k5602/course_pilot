use dioxus::prelude::*;
use crate::types::Course;
use crate::ui::components::modal::Modal;
use crate::ui::components::modal_confirmation::ModalConfirmation;
use crate::ui::hooks::{use_form_manager, use_course_manager, use_modal_manager};

#[derive(Props, PartialEq, Clone)]
pub struct CourseActionsProps {
    pub course: Course,
}

/// Course actions component handling modals and interactions
#[component]
pub fn CourseActions(props: CourseActionsProps) -> Element {
    let course_manager = use_course_manager();
    let course_name_form = use_form_manager(props.course.name.clone());
    let edit_modal = use_modal_manager(false);
    let delete_modal = use_modal_manager(false);
    
    // Handle course update
    let handle_update_course = {
        let course_manager = course_manager.clone();
        let edit_modal = edit_modal.clone();
        let course_name_form = course_name_form.clone();
        let course_id = props.course.id;
        let original_name = props.course.name.clone();
        
        move |_| {
            let new_name = course_name_form.value.trim().to_string();
            if new_name.is_empty() {
                crate::ui::components::toast::toast::error("Course name cannot be empty");
                return;
            }
            
            if new_name != original_name {
                course_manager.update_course.call((course_id, new_name));
            }
            
            edit_modal.close.call(());
        }
    };
    
    // Handle course deletion
    let handle_delete_course = {
        let course_manager = course_manager.clone();
        let delete_modal = delete_modal.clone();
        let course_id = props.course.id;
        
        move |_| {
            course_manager.delete_course.call(course_id);
            delete_modal.close.call(());
        }
    };

    rsx! {
        // Edit Course Modal
        Modal {
            open: edit_modal.is_open,
            on_close: {
                let edit_modal = edit_modal.clone();
                let course_name_form = course_name_form.clone();
                move |_| {
                    edit_modal.close.call(());
                    course_name_form.reset.call(());
                }
            },
            title: "Edit Course".to_string(),
            actions: rsx! {
                button {
                    class: "btn btn-ghost",
                    onclick: {
                        let edit_modal = edit_modal.clone();
                        let course_name_form = course_name_form.clone();
                        move |_| {
                            edit_modal.close.call(());
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
            },
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
        
        // Delete Confirmation Modal
        ModalConfirmation {
            open: delete_modal.is_open,
            title: "Delete Course",
            message: format!(
                "Are you sure you want to delete '{}'? This will also delete all associated plans and notes. This action cannot be undone.", 
                props.course.name
            ),
            confirm_label: Some("Delete Course".to_string()),
            cancel_label: Some("Cancel".to_string()),
            confirm_color: Some("error".to_string()),
            on_confirm: handle_delete_course,
            on_cancel: {
                let delete_modal = delete_modal.clone();
                move |_| delete_modal.close.call(())
            },
        }
    }
}