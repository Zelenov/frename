//! The filter of the file list (`docs/design/design-system.md` §13.4.1): after the search field,
//! since it narrows the same list. A 28-px button with the `list-filter` icon, and the number of
//! filters on inside it; it opens a menu with one checkbox per filter and the number of files
//! each matches, and "Show all". The menu stays open while filters are ticked.

use iced::widget::{self, container, mouse_area, row, space, stack};
use iced::{Alignment, Element, Length, Padding};

use crate::features::folder_workspace::Directory;
use crate::ui::badge::{badge, BadgeKind};
use crate::ui::icons::{icon, Icon};
use crate::ui::menu::{self, MenuItem};
use crate::ui::style::{self, ButtonKind};
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position};
use crate::ui::{form, text};

use super::Message;

/// Which list filter a menu row controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterKind {
    Untagged,
    Subtitles,
    Comments,
    Markers,
}

impl FilterKind {
    fn label(self) -> String {
        match self {
            Self::Untagged => fl!("folder-controls-filter-untagged"),
            Self::Subtitles => fl!("folder-controls-filter-subtitles"),
            Self::Comments => fl!("folder-controls-filter-comments"),
            Self::Markers => fl!("folder-controls-filter-markers"),
        }
    }

    fn set(self, on: bool) -> Message {
        match self {
            Self::Untagged => Message::SetUntaggedOnly(on),
            Self::Subtitles => Message::SetSubtitledOnly(on),
            Self::Comments => Message::SetCommentedOnly(on),
            Self::Markers => Message::SetMarkedOnly(on),
        }
    }
}

/// One row of the filter menu: whether the filter is on and how many files match it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FilterItem {
    kind: FilterKind,
    active: bool,
    count: usize,
}

fn items(dir: &Directory) -> [FilterItem; 4] {
    [
        FilterItem {
            kind: FilterKind::Untagged,
            active: dir.untagged_only(),
            count: dir.untagged_count(),
        },
        FilterItem {
            kind: FilterKind::Subtitles,
            active: dir.subtitled_only(),
            count: dir.subtitled_count(),
        },
        FilterItem {
            kind: FilterKind::Comments,
            active: dir.commented_only(),
            count: dir.commented_count(),
        },
        FilterItem {
            kind: FilterKind::Markers,
            active: dir.marked_only(),
            count: dir.marked_count(),
        },
    ]
}

/// How many filters are on.
fn active_count(items: &[FilterItem]) -> usize {
    items.iter().filter(|item| item.active).count()
}

/// The button after the search field: `list-filter`, and the count when filters are on. It
/// looks pressed while its menu is open.
pub fn button<'a>(dir: &Directory, open: bool) -> Element<'a, Message> {
    let active = active_count(&items(dir));
    let content = row![icon(Icon::ListFilter, ICON_M, TEXT)]
        .push((active > 0).then(|| badge(BadgeKind::Accent, active.to_string())))
        .spacing(SPACE_XS)
        .align_y(Alignment::Center);
    let kind = if open {
        ButtonKind::Segment(true)
    } else {
        ButtonKind::Secondary
    };
    let pressable = widget::button(content)
        .padding(CONTROL_ICON_INSET)
        .height(CONTROL_HEIGHT)
        .on_press(Message::ToggleFilterMenu)
        .style(style::button(kind));
    tooltip::tip_text(pressable, fl!("folder-filter-tip"), Position::Bottom)
}

/// A row of the menu: the filter's checkbox and the number of files it matches.
fn filter_row<'a>(item: FilterItem) -> Element<'a, Message> {
    menu::row_of(
        row![
            form::checkbox(item.kind.label(), item.active).on_toggle(move |on| item.kind.set(on)),
            space::horizontal(),
            text::caption(item.count.to_string()),
        ]
        .spacing(SPACE_M)
        .align_y(Alignment::Center),
    )
}

/// The open menu over the list, at its top right; a click beside it closes it.
pub fn menu<'a>(dir: &Directory) -> Element<'a, Message> {
    let rows = items(dir)
        .into_iter()
        .map(filter_row)
        .chain(std::iter::once(menu::item(MenuItem {
            icon: None,
            label: fl!("folder-show-all"),
            keys: Vec::new(),
            checked: false,
            on_press: Some(Message::ShowAll),
        })));
    let popup = container(menu::menu(rows, Length::Fixed(MENU_WIDTH)))
        .padding(Padding {
            right: SPACE_S,
            ..Padding::ZERO
        })
        .align_right(Length::Fill);
    let beside = mouse_area(container(space()).width(Length::Fill).height(Length::Fill))
        .on_press(Message::CloseFilterMenu);
    stack![beside, popup]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ticking_a_filter_turns_it_on_or_off() {
        assert!(matches!(
            FilterKind::Markers.set(false),
            Message::SetMarkedOnly(false)
        ));
        assert!(matches!(
            FilterKind::Untagged.set(true),
            Message::SetUntaggedOnly(true)
        ));
    }

    #[test]
    fn the_button_counts_the_filters_that_are_on() {
        let item = |active| FilterItem {
            kind: FilterKind::Comments,
            active,
            count: 7,
        };
        assert_eq!(active_count(&[item(true), item(false), item(true)]), 2);
        assert_eq!(active_count(&[item(false)]), 0);
    }
}
