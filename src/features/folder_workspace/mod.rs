//! Folder workspace: owns the loaded folder, selected file, and selection logic.
//!
//! The app only sees update, view, subscription, and current_file (for title).
//! How the workspace looks (splitters, regions) is only in this crate's view.
//! Each feature (folder list, video, rename) receives only data and owns its own UI.

mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::FolderWorkspace;
