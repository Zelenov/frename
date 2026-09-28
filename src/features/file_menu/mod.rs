//! The file context menu (#48): a right-click on a file in the list, or on the open file's name,
//! opens a small menu drawn by frename: show the file in Explorer (the file manager on Linux),
//! copy its full path, copy its name. Each item has a key that does the same on the open file
//! without the menu. The workspace runs the chosen action, after saving the file's pending
//! edits, so the path and name are the final ones.

mod action;
mod messages;
mod state;
pub mod system;
pub mod view;

pub use action::FileAction;
pub use messages::Message;
pub use state::FileMenuState;
