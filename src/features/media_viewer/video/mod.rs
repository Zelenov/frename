//! Video player sub-feature of media_viewer.

mod frame_step;
mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub(crate) use state::check_decodes;
#[cfg(all(test, any(target_os = "linux", windows)))]
pub(crate) use state::open_for_test;
pub use state::{Overlay, VideoPlayerState};
