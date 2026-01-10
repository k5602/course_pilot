//! UI Module - Dioxus Desktop Application
//!
//! Three-panel layout: Sidebar | Main Content | Right Panel

pub mod actions;
pub mod app;
pub mod custom;
pub mod hooks;
pub mod layouts;
pub mod pages;
pub mod routes;
pub mod state;

pub use app::App;
pub use routes::Route;
pub use state::AppState;
