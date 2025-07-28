use super::CourseCard;
use crate::types::Course;
use dioxus::prelude::*;

#[derive(Props, PartialEq, Clone)]
pub struct CourseGridProps {
    pub courses: Vec<Course>,
}

/// Clean course grid component
#[component]
pub fn CourseGrid(props: CourseGridProps) -> Element {
    rsx! {
        div {
            class: "grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6",
            {props.courses.iter().enumerate().map(|(idx, course)| {
                rsx! {
                    CourseCard {
                        key: "{course.id}",
                        course: course.clone(),
                        index: idx,
                    }
                }
            })}
        }
    }
}