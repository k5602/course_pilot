use anyhow::Result;

pub mod controls;
pub mod hooks;
pub mod player;
pub mod protocol;

pub mod types;
pub mod utils;

pub use controls::*;
pub use hooks::*;
pub use player::*;
pub use protocol::*;

pub use crate::state::video_player::*;
pub use types::*;
pub use utils::*;

/// Initialize the video player subsystem
pub fn init() -> Result<()> {
    log::info!("Video player subsystem initialized");
    Ok(())
}
