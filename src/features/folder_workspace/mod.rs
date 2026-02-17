//! Folder workspace: owns the loaded folder, selected file, and selection logic.
//!
//! The app only sees update, view, subscription, and current_file (for title).
//! How the workspace looks (splitters, regions) is only in this crate's view.
//! Each feature (folder list, video, rename) receives only data and owns its own UI.

mod messages;
mod state;
pub mod view;

/// Directory type used in this workspace (generic over AppDatabase).
pub type Directory = frename_core::Directory<frename_core::AppDatabase>;

pub use messages::Message;
pub use state::FolderWorkspace;
