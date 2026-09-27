//! Dragging files out of frename into other programs (Premiere Pro, Explorer): a real operating
//! system file drag started from a row of the file list. See `docs/design/drag-to-premiere.md`.
//!
//! The press, the move past the threshold and the decision whether the drag may start are pure
//! state here; the drag itself is platform code run on the window's thread ([`start`]).

mod messages;
mod state;
#[cfg(windows)]
mod win32;

use std::path::PathBuf;

use iced::window::raw_window_handle::HasWindowHandle;

pub use messages::Message;
pub use state::{files_to_drag, readiness, DragOutState, Readiness};

/// How a drag out of the window ended.
#[derive(Debug, Clone, PartialEq, Eq)]
// Each platform ends a drag in only some of these ways.
#[allow(dead_code)]
pub enum Outcome {
    /// Dropped on a program that took the files.
    Dropped,
    /// Cancelled (Esc, or dropped where files are not taken).
    Cancelled,
    /// Not started: the mouse button was already up, so a drag would have dropped at once.
    NotStarted,
    /// This platform cannot start a file drag yet.
    Unsupported,
    /// The drag could not be set up.
    Failed(String),
}

/// Drag `paths` out of the window: runs the operating system's drag loop and returns when the
/// files are dropped or the drag is cancelled. Must run on the window's thread (iced's
/// `window::run`). Only copy and link are offered, never move.
pub fn start(window: &dyn HasWindowHandle, paths: &[PathBuf]) -> Outcome {
    if paths.is_empty() {
        return Outcome::NotStarted;
    }
    platform_start(window, paths)
}

#[cfg(windows)]
fn platform_start(window: &dyn HasWindowHandle, paths: &[PathBuf]) -> Outcome {
    win32::start(window, paths)
}

#[cfg(not(windows))]
fn platform_start(_window: &dyn HasWindowHandle, paths: &[PathBuf]) -> Outcome {
    log::info!(
        "drag out: not supported on this platform ({} file(s))",
        paths.len()
    );
    Outcome::Unsupported
}
