//! Messages from the settings window.

use frename_core::{CommentStorage, InOutStorage};

/// User changes in the settings window. Each one is saved immediately.
#[derive(Debug, Clone)]
pub enum Message {
    /// Start playing videos as soon as they are opened.
    SetAutoplayVideo(bool),
    /// Draw every tag in one neutral color.
    SetMonochromeTags(bool),
    /// Save comments inside the media file (XMP) or in a text file next to it.
    SetCommentStorage(CommentStorage),
    /// Save in/out points as an Adobe XMP marker inside the media file, or in the file name.
    SetInOutStorage(InOutStorage),
    /// Move the open folder's comments and in/out points into the chosen storage.
    ConvertFolder,
}
