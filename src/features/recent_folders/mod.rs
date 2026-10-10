//! Recently opened folders (#63): a list of the last ten folders worked in, so switching between
//! shoots does not need the folder picker. It shows as a dropdown next to the open button
//! (`Ctrl+R` opens it) and on the empty screen. A click opens the folder with the file that was
//! open in it last time; a folder that is gone is shown as "not found" and is only removed when
//! the user says so. The list itself, its limit and its order are `frename_core::recent_folders`.

mod display;
mod messages;
mod state;
pub mod view;

pub use messages::Message;
pub use state::{Effect, RecentFoldersState};

/// Scrollable ID of the list on the empty screen (shared by the view and the keyboard scroll).
pub const RECENT_LIST_SCROLLABLE_ID: &str = "recent-folders-list";
