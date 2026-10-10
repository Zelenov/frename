//! The file menu drawn over the window: a layer that catches clicks outside the menu (they close
//! it), and the menu itself at the pointer, moved in when it would stick out of the window.

use iced::widget::{mouse_area, opaque, pin, responsive, space, stack};
use iced::{Element, Length, Point, Size};

use crate::ui;
use crate::ui::menu::MenuItem;
use crate::ui::tokens::MENU_WIDTH;

use super::{FileAction, FileMenuState, Message};

/// The menu layer over the whole window, or nothing while the menu is closed.
pub fn view(state: &FileMenuState) -> Element<'_, Message> {
    let Some(menu) = state.open() else {
        return space().into();
    };
    responsive(move |area| {
        let items = FileAction::ALL.map(|action| {
            ui::menu::item(MenuItem {
                icon: None,
                label: label(action),
                keys: action.keys().to_vec(),
                checked: false,
                on_press: Some(Message::Choose(menu.file, action)),
            })
        });
        let size = ui::menu::size(items.len());
        let at = inside(menu.position, size, area);
        // A click anywhere else closes the menu and does nothing more: the click is not passed
        // to what is under it, as in any other program's menu.
        let outside = mouse_area(space().width(Length::Fill).height(Length::Fill))
            .on_press(Message::Close)
            .on_right_press(Message::Close)
            .on_middle_press(Message::Close);
        stack![
            opaque(outside),
            pin(ui::menu::menu(items, Length::Fixed(MENU_WIDTH))).position(at)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    })
    .into()
}

/// The item's label.
fn label(action: FileAction) -> String {
    match action {
        #[cfg(windows)]
        FileAction::ShowInFileManager => fl!("file-menu-show-in-explorer"),
        #[cfg(not(windows))]
        FileAction::ShowInFileManager => fl!("file-menu-show-in-file-manager"),
        FileAction::CopyPath => fl!("file-menu-copy-path"),
        FileAction::CopyName => fl!("file-menu-copy-name"),
    }
}

/// Where a menu of `size` opening at `pointer` goes so it stays inside `area`: moved left or up
/// by as much as it would stick out, never past the top-left corner.
fn inside(pointer: Point, size: Size, area: Size) -> Point {
    Point::new(
        pointer.x.min(area.width - size.width).max(0.0),
        pointer.y.min(area.height - size.height).max(0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_menu_near_the_right_or_bottom_edge_moves_in() {
        let area = Size::new(800.0, 600.0);
        let size = Size::new(200.0, 100.0);
        assert_eq!(
            inside(Point::new(10.0, 20.0), size, area),
            Point::new(10.0, 20.0)
        );
        assert_eq!(
            inside(Point::new(700.0, 550.0), size, area),
            Point::new(600.0, 500.0)
        );
        // A window smaller than the menu: pinned to the top-left corner.
        let tiny = Size::new(150.0, 50.0);
        assert_eq!(
            inside(Point::new(100.0, 40.0), size, tiny),
            Point::new(0.0, 0.0)
        );
    }
}
