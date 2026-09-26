//! Messages from the settings window.

use frename_core::{CommentStorage, CueLength, InOutStorage};

use crate::features::batch::Operation;
use crate::soniox_key::{KeyInfo, SonioxKey};

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
    /// Read the Soniox API key from the credential store, unless it was read already (sent
    /// when the window opens or the subtitle action is chosen).
    LoadSonioxKey,
    /// The key was read (internal).
    SonioxKeyLoaded(KeyInfo),
    /// Typing in the key field.
    SonioxKeyInput(String),
    /// Show or mask the typed key.
    ToggleShowSonioxKey,
    /// Save the typed key in the credential store.
    SaveSonioxKey,
    /// The key was saved, or why not (internal).
    SonioxKeySaved(Result<SonioxKey, String>),
    /// Show the key field again to type a new key over the saved one.
    ReplaceSonioxKey,
    /// Delete the saved key.
    RemoveSonioxKey,
    /// The key was deleted, or why not (internal).
    SonioxKeyRemoved(Result<(), String>),
    /// Check or uncheck a language spoken in the footage (a code such as "en").
    SetSubtitleLanguage(String, bool),
    /// How long generated subtitle cues may get.
    SetSubtitleCueLength(CueLength),
}
