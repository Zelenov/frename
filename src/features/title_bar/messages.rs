//! Messages of the title bar.

/// What the title bar asks the app to do with the main window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// The left button went down on the bar's empty part.
    Pressed,
    /// The pointer moved over the bar: with the button still down, the window starts moving.
    Moved,
    /// The left button came up over the bar.
    Released,
    /// A double click on the bar's empty part: maximize, or restore.
    DoubleClicked,
    /// A right press on the bar's empty part: the system's window menu.
    SystemMenu,
    Minimize,
    ToggleMaximize,
    Close,
}
