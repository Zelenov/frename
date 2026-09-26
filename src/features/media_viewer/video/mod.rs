//! Video player sub-feature of media_viewer.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::VideoPlayerState;
pub(crate) use state::{description as pipeline_description, sinks as pipeline_sinks};
