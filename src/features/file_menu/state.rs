//! State of the file context menu: where the right button last went down, and the menu shown.

use frename_core::FileId;
use iced::{Point, Task};

use super::Message;

/// The menu, while it is shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpenMenu {
    /// The file the actions apply to.
    pub file: FileId,
    /// Where the menu opens: its top-left corner is at the pointer, in window coordinates.
    pub position: Point,
}

#[derive(Debug, Default)]
pub struct FileMenuState {
    /// Where the right button last went down: the file's own right-click message comes right
    /// after it and opens the menu there.
    right_pressed_at: Option<Point>,
    open: Option<OpenMenu>,
}

impl FileMenuState {
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::RightPressed(position) => {
                self.right_pressed_at = Some(position);
                self.open = None;
            }
            Message::Open(file) => {
                // Only right after a right press: the position is for this click alone.
                if let Some(position) = self.right_pressed_at.take() {
                    self.open = Some(OpenMenu { file, position });
                }
            }
            Message::Close | Message::Choose(..) => self.open = None,
        }
        Task::none()
    }

    /// The menu shown, if any.
    pub fn open(&self) -> Option<OpenMenu> {
        self.open
    }

    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::file_menu::FileAction;

    fn file() -> FileId {
        FileId::new()
    }

    #[test]
    fn a_right_click_on_a_file_opens_the_menu_where_the_button_went_down() {
        let mut state = FileMenuState::default();
        let id = file();
        let _ = state.update(Message::RightPressed(Point::new(40.0, 120.0)));
        let _ = state.update(Message::Open(id));
        assert_eq!(
            state.open(),
            Some(OpenMenu {
                file: id,
                position: Point::new(40.0, 120.0)
            })
        );
    }

    #[test]
    fn choosing_an_item_esc_or_a_right_click_elsewhere_closes_it() {
        let mut state = FileMenuState::default();
        let id = file();
        let open = |state: &mut FileMenuState| {
            let _ = state.update(Message::RightPressed(Point::ORIGIN));
            let _ = state.update(Message::Open(id));
            assert!(state.is_open());
        };
        open(&mut state);
        let _ = state.update(Message::Choose(id, FileAction::CopyName));
        assert!(!state.is_open());
        open(&mut state);
        let _ = state.update(Message::Close);
        assert!(!state.is_open());
        open(&mut state);
        let _ = state.update(Message::RightPressed(Point::new(5.0, 5.0)));
        assert!(!state.is_open());
    }

    #[test]
    fn a_menu_asked_for_without_a_right_press_does_not_open() {
        let mut state = FileMenuState::default();
        let _ = state.update(Message::Open(file()));
        assert!(!state.is_open());
        // A press is used once: a second request does not reopen at the old point.
        let id = file();
        let _ = state.update(Message::RightPressed(Point::ORIGIN));
        let _ = state.update(Message::Open(id));
        let _ = state.update(Message::Close);
        let _ = state.update(Message::Open(id));
        assert!(!state.is_open());
    }
}
