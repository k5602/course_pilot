use crate::types::Course;
use crate::ui::components::modal::{Modal, confirmation_modal};
use crate::ui::hooks::{use_course_manager, use_form_manager};
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
                crate::ui::components::toast::toast::error("Course name cannot be empty");
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
