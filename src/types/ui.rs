use dioxus::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[cfg(debug_assertions)]
use crate::ui::routes::ToastTest;
use crate::ui::routes::{AddCourse, AllCourses, Dashboard, Home, PlanView, Settings};

use super::course::{Course, Note};
use super::import::ImportJob;
use super::plan::{Plan, PlanViewState};

#[derive(Clone, Debug, PartialEq, Routable)]
pub enum Route {
    #[route("/")]
    Home {},

    #[route("/dashboard")]
    Dashboard {},

    #[route("/courses")]
    AllCourses {},

    #[route("/plan/:course_id")]
    PlanView { course_id: String },

    #[route("/settings")]
    Settings {},

    #[route("/import")]
    AddCourse {},

    #[cfg(debug_assertions)]
    #[route("/toast-test")]
    ToastTest {},
}

#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub courses: Vec<Course>,
    pub plans: Vec<Plan>,
    pub notes: Vec<Note>,
    pub active_import: Option<ImportJob>,
    pub contextual_panel: ContextualPanelState,
    pub sidebar_open_mobile: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CourseStatus {
    Structured,
    Unstructured,
    Pending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextualPanelTab {
    Notes,
    Chatbot,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct VideoContext {
    pub course_id: Uuid,
    pub video_index: usize,
    pub video_title: String,
    pub module_title: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContextualPanelState {
    pub is_open: bool,
    pub active_tab: ContextualPanelTab,
    pub video_context: Option<VideoContext>,
}

impl Default for ContextualPanelState {
    fn default() -> Self {
        Self { is_open: false, active_tab: ContextualPanelTab::Notes, video_context: None }
    }
}
