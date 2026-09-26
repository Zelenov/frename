//! Messages from the settings window.

use std::path::PathBuf;

use frename_core::{CommentStorage, InOutStorage};

use crate::features::batch::Operation;
use crate::features::updates;

/// User changes in the settings window. Each one is saved immediately.
#[derive(Debug, Clone)]
pub enum Message {
    /// Start playing videos as soon as they are opened.
    SetAutoplayVideo(bool),
    /// Draw every tag in one neutral color.
    SetMonochromeTags(bool),
    /// Put a space after each tag in file names (`Food. clip.mp4`).
    SetSpaceAfterTags(bool),
    /// Save comments inside the video file or in a text file next to it.
    SetCommentStorage(CommentStorage),
    /// Save in/out points inside the video file (as a Premiere Pro marker) or in the file name.
    SetInOutStorage(InOutStorage),
    /// The tag checked on videos that get a comment while comments are inside the video.
    SetCommentedTag(String),
    /// Whether videos that get a comment get the tag at all.
    SetCommentedTagEnabled(bool),
    /// After a storage change: open batch mode in the main window, set up to move the files'
    /// comments or in/out points to the new storage. Handled by the app.
    OpenBatchAction(Operation),
    /// The Updates section.
    Updates(updates::Message),
    /// **Import from an old frename folder…**: pick the folder of a zip version.
    ImportOldSettings,
    /// The folder picked for the import, or `None` when the picker was closed.
    OldSettingsFolderPicked(Option<PathBuf>),
}
