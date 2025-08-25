use anyhow::Result;

pub mod controls;
pub mod ipc;
pub mod player;
pub mod protocol;

pub mod types;
pub mod utils;

pub use crate::state::video_player::{
    VideoPlayerContext, VideoPlayerProvider, VideoPlayerProviderProps, use_video_player,
};
pub use controls::*;

pub use ipc::*;
pub use player::*;
pub use protocol::*;

pub use types::*;
pub use utils::*;

/// Initialize the video player subsystem
pub fn init() -> Result<()> {
    log::info!("Video player subsystem initialized (IPC ready)");
    Ok(())
}
