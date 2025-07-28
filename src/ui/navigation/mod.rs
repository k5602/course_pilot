//! Navigation Module for Course Pilot
//!
//! This module provides navigation components including breadcrumbs,
//! route guards, deep linking support, and routing utilities.

pub mod breadcrumbs;
pub mod route_guards;
pub mod deep_linking;

// Re-export navigation components
pub use breadcrumbs::Breadcrumbs;
pub use route_guards::{RouteGuard, RouteGuardResult, RouteGuardManager, RouteGuardProvider, use_route_guard};
pub use deep_linking::{DeepLinkingHandler, DeepLinkingManager, use_deep_linking};

#[cfg(debug_assertions)]
pub use deep_linking::DeepLinkingTester;
