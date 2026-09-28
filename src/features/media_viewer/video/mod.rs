//! Video player sub-feature of media_viewer.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub(crate) use state::check_decodes;
pub use state::{Overlay, VideoPlayerState};
