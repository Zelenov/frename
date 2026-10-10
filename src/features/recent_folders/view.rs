//! The list of recent folders (`docs/design/design-system.md` §8.14, §13.4.3, §13.7): a menu over
//! the window above the open button, and a column on the empty screen. Both are the same rows,
//! the same component: the folder's name and when it was opened, the folder it is in under that,
//! greyed and cut from the front; a folder that is gone is disabled and says so; the ✕ shows on
//! the row the pointer or the keys are on. All the text and numbers come from `state` and the
//! tokens.

use frename_core::recent_folders::{self, RecentFolder};
use iced::widget::{column, container, mouse_area, opaque, pin, responsive, row, space, stack};
use iced::{mouse, Alignment, Element, Length, Padding, Size};

use crate::ui::icon_button::IconButton;
use crate::ui::icons::{icon, Icon};
use crate::ui::menu::{self, MenuItem};
use crate::ui::tokens::*;
use crate::ui::tooltip::Position;
use crate::ui::{button, list, scroll, style, text};

use super::display::{age_text, shorten_path};
use super::state::{Entry, Presence, RecentFoldersState};
use super::{Message, RECENT_LIST_SCROLLABLE_ID};

/// The inset of a row's content from its edges, and the gap between its parts.
const GAP: f32 = SPACE_S;

/// The menu's padding, which `ui::menu::menu` owns; the rows sit inside it.
const MENU_PADDING: f32 = SPACE_XS;

/// A row's width of text: what the row leaves after its inset, the selection bar, the ✕ and the
/// gap before it.
fn text_room(row_width: f32) -> f32 {
    row_width - 2.0 * GAP - SELECTION_BAR - ICON_BUTTON_SMALL - GAP
}

/// How many characters of caption text fit in `width`, generously (the estimate is for wide
/// letters, so a cut line is never longer than its box).
fn captions_in(width: f32) -> usize {
    (width / CAPTION_CHAR_WIDTH).floor().max(0.0) as usize
}

/// The ✕ of a highlighted row, or the room it takes.
fn remove_slot<'a>(entry: &Entry, shown: bool) -> Element<'a, Message> {
    if !shown {
        return space().width(ICON_BUTTON_SMALL).into();
    }
    IconButton::new(Icon::X)
        .small()
        .tip(fl!("recent-folders-remove-tip"), Position::Top)
        .on_press(Message::Remove(entry.folder.folder.clone()))
        .into()
}

/// A row of the list. `width` is the row's width, which the texts are cut to.
fn entry_row<'a>(
    state: &RecentFoldersState,
    at: usize,
    entry: &'a Entry,
    width: f32,
) -> Element<'a, Message> {
    let folder: &RecentFolder = &entry.folder;
    let highlighted = state.highlight() == Some(at);
    let room = text_room(width);
    let missing = entry.presence == Presence::Missing;
    if state.is_asking(&folder.folder) {
        return question_row(entry, room);
    }
    // A folder that is gone is dimmed: its words and its path.
    let color = if missing { TEXT_DISABLED } else { TEXT };
    let grey = if missing {
        TEXT_DISABLED
    } else {
        TEXT_SECONDARY
    };
    let name = text::fit(&folder.name(), room * 0.6, BODY_CHAR_WIDTH).into_owned();
    let when = if missing {
        fl!("recent-folders-not-found")
    } else {
        age_text(recent_folders::age(folder.opened_at_ms, state.now_ms()))
    };
    let parent = folder
        .parent()
        .map(|parent| shorten_path(&parent, captions_in(room)))
        .unwrap_or_default();
    let top = row![
        container(text::body(name).color(color)).width(Length::Fill),
        text::caption(when).color(grey),
    ]
    .spacing(GAP)
    .align_y(Alignment::Center);
    let lines = column![top, text::caption(parent).color(grey)].width(Length::Fill);
    let content = row![lines, remove_slot(entry, highlighted)]
        .spacing(GAP)
        .align_y(Alignment::Center);
    let body = list::row_item(
        container(content).center_y(Length::Fill),
        highlighted,
        HOVER,
        Length::Fixed(RECENT_ROW_HEIGHT),
    );
    mouse_area(body)
        .on_press(Message::Choose(folder.folder.clone()))
        .on_enter(Message::Highlight(at))
        .interaction(mouse::Interaction::Pointer)
        .into()
}

/// The row of a missing folder the user clicked: what is wrong and what to do about it. Nothing
/// is removed until they say so, because the drive may come back.
fn question_row<'a>(entry: &'a Entry, room: f32) -> Element<'a, Message> {
    let folder = &entry.folder;
    let words = column![
        text::body(text::fit(&folder.name(), room * 0.5, BODY_CHAR_WIDTH).into_owned()),
        text::caption(fl!("recent-folders-missing-line")),
    ]
    .width(Length::Fill);
    let content = row![
        words,
        button::secondary(fl!("recent-folders-remove"))
            .on_press(Message::Remove(folder.folder.clone())),
        button::ghost(fl!("recent-folders-keep")).on_press(Message::Keep),
    ]
    .spacing(GAP)
    .align_y(Alignment::Center);
    // Opaque: a click on the row's blank part must not reach the empty screen's own click, which
    // opens the folder picker.
    opaque(list::row_item(
        container(content).center_y(Length::Fill),
        true,
        HOVER,
        Length::Fixed(RECENT_ROW_HEIGHT),
    ))
}

/// The rows of the list, one under the other.
fn rows<'a>(state: &'a RecentFoldersState, width: f32) -> Vec<Element<'a, Message>> {
    state
        .entries()
        .iter()
        .enumerate()
        .map(|(at, entry)| entry_row(state, at, entry, width))
        .collect()
}

/// A hairline between the rows and "Clear list", 4 px of room around it.
fn separator<'a>() -> Element<'a, Message> {
    container(
        container(space())
            .width(Length::Fill)
            .height(LINE)
            .style(style::divider),
    )
    .padding(Padding {
        top: SPACE_XS,
        bottom: SPACE_XS,
        ..Padding::ZERO
    })
    .into()
}

fn clear_item<'a>() -> Element<'a, Message> {
    menu::item(MenuItem {
        icon: Some(Icon::Trash),
        label: fl!("recent-folders-clear"),
        keys: Vec::new(),
        checked: false,
        on_press: Some(Message::Clear),
    })
}

/// How high the dropdown is with `count` rows: its edge, the rows, the hairline and Clear list.
fn menu_height(count: usize) -> f32 {
    2.0 * MENU_PADDING
        + count as f32 * RECENT_ROW_HEIGHT
        + (LINE + 2.0 * SPACE_XS)
        + MENU_ITEM_HEIGHT
}

/// Where the dropdown's top-left corner is in a window of `area`: its bottom edge on the toolbar
/// under the file list, whose column starts `left_width` + a splitter in; moved in when it would
/// stick out of the window.
// Coupled to the toolbar layout: BAR_HEIGHT + LINE is the toolbar under the file list, and the
// column starts after the video pane and a splitter (see `folder_workspace::view`).
fn menu_origin(area: Size, left_width: f32, count: usize) -> iced::Point {
    let height = menu_height(count);
    let x = (left_width + SPLITTER_HIT + SPACE_S).min(area.width - RECENT_LIST_WIDTH);
    let y = area.height - (LINE + BAR_HEIGHT) - height;
    iced::Point::new(x.max(0.0), y.max(0.0))
}

/// The dropdown layer over the whole window, or nothing while it is closed. `left_width` is the
/// video pane's width, which says where the file list's toolbar is.
pub fn dropdown(state: &RecentFoldersState, left_width: f32) -> Element<'_, Message> {
    if !state.is_open() {
        return space().into();
    }
    responsive(move |area| {
        let body: Vec<Element<'_, Message>> = if state.is_empty() {
            vec![menu::item(MenuItem {
                icon: None,
                label: fl!("recent-folders-none"),
                keys: Vec::new(),
                checked: false,
                on_press: None,
            })]
        } else {
            let mut all = rows(state, RECENT_LIST_WIDTH - 2.0 * MENU_PADDING);
            all.push(separator());
            all.push(clear_item());
            all
        };
        let at = menu_origin(area, left_width, state.entries().len());
        // A click beside the menu closes it and does nothing more.
        let beside = mouse_area(space().width(Length::Fill).height(Length::Fill))
            .on_press(Message::Close)
            .on_right_press(Message::Close)
            .on_middle_press(Message::Close);
        stack![
            opaque(beside),
            pin(menu::menu(body, Length::Fixed(RECENT_LIST_WIDTH))).position(at)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    })
    .into()
}

/// The list on the empty screen: its title, the rows (scrolling when the window is short) and
/// Clear list.
pub fn on_empty_screen(state: &RecentFoldersState) -> Element<'_, Message> {
    let header = row![
        icon(Icon::Folder, ICON_M, TEXT_SECONDARY),
        text::strong(fl!("recent-folders-title")),
    ]
    .spacing(SPACE_S)
    .align_y(Alignment::Center);
    let list = scroll::vertical_with_id(
        RECENT_LIST_SCROLLABLE_ID,
        column(rows(state, RECENT_LIST_WIDTH - SCROLL_GUTTER)),
    )
    .height(Length::Shrink);
    // The header is level with the rows' text.
    let header = container(header).padding(Padding {
        left: SPACE_S,
        ..Padding::ZERO
    });
    column![
        header,
        list,
        container(button::ghost(fl!("recent-folders-clear")).on_press(Message::Clear))
    ]
    .spacing(SPACE_S)
    .width(Length::Fixed(RECENT_LIST_WIDTH))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_list_fits_above_the_toolbar_in_the_smallest_window() {
        let most = frename_core::recent_folders::MAX_RECENT_FOLDERS;
        let at = menu_origin(
            Size::new(WINDOW_MIN_WIDTH, WINDOW_MIN_HEIGHT),
            VIDEO_WIDTH,
            most,
        );
        assert!(
            at.y > 0.0,
            "ten rows and Clear list are not pushed off the top"
        );
    }

    #[test]
    fn the_dropdown_sits_on_the_toolbar_over_the_file_list() {
        let area = Size::new(1440.0, 800.0);
        let at = menu_origin(area, 440.0, 10);
        assert_eq!(at.x, 440.0 + SPLITTER_HIT + SPACE_S);
        assert_eq!(
            at.y + menu_height(10),
            800.0 - BAR_HEIGHT - LINE,
            "its bottom edge is on the toolbar's line"
        );
    }

    #[test]
    fn the_dropdown_moves_in_when_it_would_stick_out() {
        let area = Size::new(600.0, 300.0);
        let at = menu_origin(area, 440.0, 10);
        assert_eq!(at.x, 600.0 - RECENT_LIST_WIDTH);
        assert_eq!(at.y, 0.0, "never above the window");
    }

    #[test]
    fn a_row_leaves_room_for_the_cross() {
        assert!(text_room(RECENT_LIST_WIDTH) < RECENT_LIST_WIDTH - ICON_BUTTON_SMALL);
        assert!(captions_in(text_room(RECENT_LIST_WIDTH)) > 20);
    }
}
