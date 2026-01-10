//! Application Routes

use dioxus::prelude::*;

use crate::ui::layouts::MainLayout;
use crate::ui::pages::*;

/// Application routes with typesafe navigation.
#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    Dashboard {},
    
    #[route("/courses")]
    CourseList {},
    
    #[route("/courses/:course_id")]
    CourseView { course_id: String },
    
    #[route("/courses/:course_id/video/:video_id")]
    VideoPlayer { course_id: String, video_id: String },
    
    #[route("/quizzes")]
    QuizList {},
    
    #[route("/quizzes/:exam_id")]
    QuizView { exam_id: String },
    
    #[route("/settings")]
    Settings {},
}
