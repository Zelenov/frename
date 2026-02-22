//! Unified media viewer feature: routes video and image files internally.
//!
//! FolderWorkspace holds one `MediaViewerState` and calls `open(file)` for every file.
//! Routing between video and image is hidden inside this module.

pub mod image;
pub mod video;
pub mod view;
mod messages;
mod state;

pub use messages::Message;
pub use state::MediaViewerState;
