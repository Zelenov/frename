//! Video player sub-feature of media_viewer.

mod frame_step;
mod messages;
mod side_search;
mod state;
pub mod view;

pub use messages::Message;
pub use side_search::{step, step_cue, SearchMessage, SideSearch, SIDE_SEARCH_INPUT_ID};
pub(crate) use state::check_decodes;
#[cfg(all(test, any(target_os = "linux", windows)))]
pub(crate) use state::{real_clip, video_loaded};
pub use state::{Overlay, VideoPlayerState};
