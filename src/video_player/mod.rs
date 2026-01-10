//! video player module for Course Pilot
//!
//! Inspired by distraction-free approach:
//! - Simple YouTube iframe with onStateChange callback
//! - Basic HTML5 video for local files
//! - Progress tracking (position + completed)

pub mod local;
pub mod progress;
pub mod protocol;
pub mod types;
pub mod youtube;

pub use local::*;
pub use progress::*;
pub use protocol::handle_video_request;
pub use types::*;
pub use youtube::*;

use anyhow::Result;

/// Initialize the video player subsystem
pub fn init() -> Result<()> {
    log::info!("Video player subsystem initialized");
    Ok(())
}
