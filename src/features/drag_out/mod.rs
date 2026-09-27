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
pub use state::{files_to_drag, readiness, save_failed, DragCheck, DragOutState, Readiness, Save};

/// Whether this platform can drag files out of the window. Elsewhere a press on a row stays a
/// click, with no save or other work done for a drag that cannot start. Tests drive the
/// decisions on every platform (they never reach the platform code).
pub const SUPPORTED: bool = cfg!(any(windows, test));

/// Drag `paths` out of the window: runs the operating system's drag loop and returns when the
/// files are dropped or the drag is cancelled. Must run on the window's thread (iced's
/// `window::run`). Only copy and link are offered, never move. The platform code logs how it
/// ended (dropped, cancelled, not started, failed); nothing else depends on it.
pub fn start(window: &dyn HasWindowHandle, paths: &[PathBuf]) {
    if !paths.is_empty() {
        platform_start(window, paths);
    }
}

#[cfg(windows)]
fn platform_start(window: &dyn HasWindowHandle, paths: &[PathBuf]) {
    win32::start(window, paths);
}

/// Never called: [`SUPPORTED`] keeps a press on a row a plain click here.
#[cfg(not(windows))]
fn platform_start(_window: &dyn HasWindowHandle, _paths: &[PathBuf]) {}

/// Whether the primary mouse button is held now. A press and its release can reach frename in
/// one batch (a touchpad tap), before the release listener exists; this catches that.
#[cfg(all(windows, not(test)))]
pub fn primary_button_down() -> bool {
    win32::primary_button_down()
}

/// Whether the primary mouse button is held now: assumed so where it cannot be read, and in
/// tests, which drive the press and the moves themselves.
#[cfg(any(not(windows), test))]
pub fn primary_button_down() -> bool {
    true
}
