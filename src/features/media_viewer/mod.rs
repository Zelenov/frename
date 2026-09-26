//! Unified media viewer feature: routes video and image files internally.
//!
//! FolderWorkspace holds one `MediaViewerState` and calls `open(file)` for every file.
//! Routing between video and image is hidden inside this module.

pub mod image;
mod messages;
mod state;
pub mod video;
pub mod view;

pub use messages::Message;
pub use state::MediaViewerState;
