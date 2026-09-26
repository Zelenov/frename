//! Messages from the settings window.

use frename_core::ai::key::KeyState;
use frename_core::ai::SummaryLanguage;
use frename_core::{CommentStorage, InOutStorage};

use crate::features::batch::Operation;

/// User changes in the settings window. Each setting is saved immediately; the API key only on
/// Save.
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
    /// The language AI descriptions are written in.
    SetSummaryLanguage(SummaryLanguage),
    /// Typing in the API key field.
    KeyInput(String),
    /// Show or hide the typed key.
    ToggleShowKey,
    /// Save the typed key in the credential store. Handled by the app, on a worker thread.
    SaveKey,
    /// Show the key field again to type a new key over the saved one.
    ReplaceKey,
    /// Keep the saved key after all.
    CancelReplaceKey,
    /// Remove the saved key from the credential store. Handled by the app.
    RemoveKey,
    /// Whether a key is saved: read in the background, or after Save / Remove. `Err` says why a
    /// save or removal failed. `request` numbers the read, so an answer that took longer than a
    /// later one cannot override it.
    KeyState {
        request: u64,
        result: Result<KeyState, String>,
    },
}
