//! Media viewer feature: shows the selected video, or a placeholder for anything else.
//!
//! FolderWorkspace holds one `MediaViewerState` and calls `open(file)` for every file.

mod messages;
mod placeholder;
mod state;
pub mod video;
pub mod view;

pub use messages::Message;
pub use state::MediaViewerState;
