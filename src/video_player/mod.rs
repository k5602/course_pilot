use anyhow::Result;

pub mod types;
pub mod state;
pub mod player;
pub mod controls;
pub mod hooks;
pub mod utils;

pub use types::*;
pub use state::*;
pub use player::*;
pub use controls::*;
pub use hooks::*;
pub use utils::*;

/// Initialize the video player subsystem
pub fn init() -> Result<()> {
    log::info!("Video player subsystem initialized");
    Ok(())
}
