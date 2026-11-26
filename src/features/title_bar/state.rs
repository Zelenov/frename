//! The title bar's state: whether a press on its empty part may still become a move of the window.

use super::{Edge, Message};

/// What the bar asks of the main window; the app turns it into a window task.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowOp {
    Drag,
    Resize(Edge),
    ToggleMaximize,
    SystemMenu,
    Minimize,
    Close,
}

#[derive(Debug, Default)]
pub struct State {
    /// The left button went down on the bar's empty part and has not come up or left it since.
    pressed: bool,
}

impl State {
    /// Whether the next move over the bar starts moving the window: the view listens for moves
    /// only then.
    pub fn pressed(&self) -> bool {
        self.pressed
    }

    pub fn update(&mut self, msg: Message) -> Option<WindowOp> {
        match msg {
            Message::Pressed => {
                self.pressed = true;
                None
            }
            Message::Moved if self.pressed => {
                self.pressed = false;
                Some(WindowOp::Drag)
            }
            Message::Moved => None,
            Message::Released => {
                self.pressed = false;
                None
            }
            Message::DoubleClicked | Message::ToggleMaximize => {
                self.pressed = false;
                Some(WindowOp::ToggleMaximize)
            }
            Message::SystemMenu => Some(WindowOp::SystemMenu),
            Message::Minimize => Some(WindowOp::Minimize),
            Message::Close => Some(WindowOp::Close),
            Message::Resize(edge) => {
                self.pressed = false;
                Some(WindowOp::Resize(edge))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_then_a_move_drags_the_window_once() {
        let mut state = State::default();
        assert_eq!(state.update(Message::Pressed), None);
        assert_eq!(state.update(Message::Moved), Some(WindowOp::Drag));
        assert_eq!(state.update(Message::Moved), None);
    }

    #[test]
    fn a_move_without_a_press_does_nothing() {
        let mut state = State::default();
        assert_eq!(state.update(Message::Moved), None);
    }

    #[test]
    fn leaving_the_bar_after_a_press_ends_it() {
        // Pressed, then the pointer left the bar (and the button came up somewhere else): a later
        // hover over the bar must not move the window.
        let mut state = State::default();
        state.update(Message::Pressed);
        assert_eq!(state.update(Message::Released), None);
        assert!(!state.pressed());
        assert_eq!(state.update(Message::Moved), None);
    }

    #[test]
    fn a_double_click_maximizes_and_ends_the_press() {
        let mut state = State::default();
        state.update(Message::Pressed);
        assert_eq!(
            state.update(Message::DoubleClicked),
            Some(WindowOp::ToggleMaximize)
        );
        assert_eq!(state.update(Message::Moved), None);
    }

    #[test]
    fn an_edge_press_resizes_from_that_edge() {
        let mut state = State::default();
        assert_eq!(
            state.update(Message::Resize(Edge::SouthEast)),
            Some(WindowOp::Resize(Edge::SouthEast))
        );
    }
}
