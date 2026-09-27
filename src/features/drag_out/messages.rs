//! Messages of the drag out of the window.

use iced::Point;

/// Mouse events while a file row is pressed (from [`super::DragOutState::subscription`]).
#[derive(Debug, Clone)]
pub enum Message {
    /// The cursor moved to this position (window coordinates, logical pixels).
    Moved(Point),
    /// The left mouse button was released before a drag started.
    Released,
}
