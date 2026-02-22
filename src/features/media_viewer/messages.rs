//! Messages for the media_viewer feature.

use super::{image, video};

/// Messages handled by MediaViewerState.
/// FolderWorkspace intercepts `Unloaded`; all other variants are forwarded to update().
#[derive(Debug, Clone)]
pub enum Message {
    /// Internal video player messages.
    Video(video::Message),
    /// Internal image viewer messages.
    Image(image::Message),
    /// FolderWorkspace sends this to begin media teardown before a file rename.
    Unload,
    /// Emitted by MediaViewerState when teardown is complete and it is safe to rename.
    /// FolderWorkspace intercepts this before calling update().
    Unloaded,
}
