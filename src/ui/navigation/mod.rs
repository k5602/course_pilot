//! Navigation Module for Course Pilot
//!
//! This module provides navigation components including breadcrumbs,
//! route guards, deep linking support, and routing utilities.

pub mod breadcrumbs;
pub mod deep_linking;
pub mod route_guards;

// Re-export navigation components
pub use breadcrumbs::Breadcrumbs;
pub use deep_linking::{DeepLinkingHandler, DeepLinkingManager, use_deep_linking};
pub use route_guards::{
    RouteGuard, RouteGuardManager, RouteGuardProvider, RouteGuardResult, use_route_guard,
};

#[cfg(debug_assertions)]
pub use deep_linking::DeepLinkingTester;
