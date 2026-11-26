//! Messages of the title bar.

use iced::window::Direction;

/// An edge or corner of the window, the side a resize moves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Edge {
    North,
    South,
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl Edge {
    pub fn direction(self) -> Direction {
        match self {
            Edge::North => Direction::North,
            Edge::South => Direction::South,
            Edge::East => Direction::East,
            Edge::West => Direction::West,
            Edge::NorthEast => Direction::NorthEast,
            Edge::NorthWest => Direction::NorthWest,
            Edge::SouthEast => Direction::SouthEast,
            Edge::SouthWest => Direction::SouthWest,
        }
    }
}

/// What the title bar and the window's edges ask the app to do with the main window.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Message {
    /// The left button went down on the bar's empty part.
    Pressed,
    /// The pointer moved over the bar's empty part while the button was down there: the window
    /// starts moving.
    Moved,
    /// The left button came up over the bar's empty part, or the pointer left it: whatever comes
    /// next is no longer a drag of the bar.
    Released,
    /// A double click on the bar's empty part: maximize, or restore.
    DoubleClicked,
    /// A right press on the bar's empty part: the system's window menu.
    SystemMenu,
    Minimize,
    ToggleMaximize,
    Close,
    /// The left button went down on one of the window's edges or corners: resize from there.
    Resize(Edge),
}
