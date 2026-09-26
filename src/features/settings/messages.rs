//! Messages from the settings window.

use std::path::PathBuf;

use frename_core::ai::key::KeyState;
use frename_core::ai::SummaryLanguage;
use frename_core::{CommentStorage, InOutStorage};

use crate::features::batch::Operation;
use crate::features::updates;

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
    /// The Updates section.
    Updates(updates::Message),
    /// **Import from an old frename folder…**: pick the folder of a zip version.
    ImportOldSettings,
    /// The folder picked for the import, or `None` when the picker was closed.
    OldSettingsFolderPicked(Option<PathBuf>),
    /// The language AI descriptions are written in.
    SetSummaryLanguage(SummaryLanguage),
    /// The API key section; never saved with the settings (the key lives in the credential
    /// store).
    Key(KeyMessage),
}

/// The API key section's messages.
#[derive(Debug, Clone)]
pub enum KeyMessage {
    /// Typing in the key field.
    Input(String),
    /// Show or hide the typed key.
    ToggleShow,
    /// Save the typed key in the credential store. Handled by the app, on a worker thread.
    Save,
    /// Show the key field again to type a new key over the saved one.
    Replace,
    /// Keep the saved key after all.
    CancelReplace,
    /// Ask before removing the saved key (a key is shown only once, when it is made).
    AskRemove,
    CancelRemove,
    /// Remove the saved key from the credential store. Handled by the app.
    Remove,
    /// Whether a key is saved: read in the background, or after Save / Remove. `Err` says why a
    /// save or removal failed. `request` numbers the read, so an answer that took longer than a
    /// later one cannot override it.
    State {
        request: u64,
        result: Result<KeyState, String>,
    },
}
