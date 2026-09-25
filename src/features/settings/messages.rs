//! Messages from the settings window.

use frename_core::{CommentStorage, InOutStorage};

/// User changes in the settings window. Each one is saved immediately.
#[derive(Debug, Clone)]
pub enum Message {
    /// Start playing videos as soon as they are opened.
    SetAutoplayVideo(bool),
    /// Draw every tag in one neutral color.
    SetMonochromeTags(bool),
    /// Save comments inside the video file or in a text file next to it.
    SetCommentStorage(CommentStorage),
    /// Save in/out points inside the video file (as a Premiere Pro marker) or in the file name.
    SetInOutStorage(InOutStorage),
    /// The tag added to videos with a comment while comments are inside the video (empty: off).
    SetCommentedTag(String),
    /// Move the open folder's comments and in/out points into the chosen storage.
    ConvertFolder,
    /// Stop the running folder conversion.
    CancelConversion,
}
