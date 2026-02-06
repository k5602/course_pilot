//! Application Routes

use dioxus::prelude::*;
use dioxus_motion::prelude::*;

use crate::ui::layouts::MainLayout;
use crate::ui::pages::*;

/// Application routes with typesafe navigation.
#[derive(Clone, Debug, PartialEq, Routable, MotionTransitions)]
#[rustfmt::skip]
pub enum Route {
    #[layout(MainLayout)]
    #[route("/")]
    #[transition(Fade)]
    Dashboard {},
    
    #[route("/courses")]
    #[transition(Fade)]
    CourseList {},
    
    #[route("/courses/:course_id")]
    #[transition(ZoomIn)]
    CourseView { course_id: String },
    
    #[route("/courses/:course_id/video/:video_id")]
    #[transition(ZoomIn)]
    VideoPlayer { course_id: String, video_id: String },
    
    #[route("/quizzes")]
    #[transition(Fade)]
    QuizList {},
    
    #[route("/quizzes/:exam_id")]
    #[transition(ZoomIn)]
    QuizView { exam_id: String },
    
    #[route("/settings")]
    #[transition(Fade)]
    Settings {},
}
