#![allow(clippy::module_name_repetitions)]
//! Deprecated shim: video player state moved to `crate::state::video_player`.
//!
//! This module re-exports the new location to maintain backwards compatibility.
//! Prefer importing from `crate::state::video_player` going forward.

#[allow(unused_imports)]
pub use crate::state::video_player::*;
