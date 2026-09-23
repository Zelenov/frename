//! Messages from the settings window.

/// User changes in the settings window. Each one is saved immediately.
#[derive(Debug, Clone)]
pub enum Message {
    /// Start playing videos as soon as they are opened.
    SetAutoplayVideo(bool),
    /// Draw every tag in one neutral color.
    SetMonochromeTags(bool),
}
