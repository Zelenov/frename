//! The toolbar under the file list (`docs/design/design-system.md` §13.4.3): moving between files,
//! opening a folder, and, until the app bar of #64, Settings and the batch mode toggle. Receives
//! only what it shows.

use iced::widget::{column, container, mouse_area, opaque, responsive, row, space, stack};
use iced::{Alignment, Element, Length, Padding};

use super::fold::{Fold, Folded};
use crate::features::folder;
use crate::ui::icon_button::IconButton;
use crate::ui::icons::Icon;
use crate::ui::menu::{self, MenuItem};
use crate::ui::style;
use crate::ui::tokens::*;
use crate::ui::tooltip::{Position, Tip};

/// What the toolbar shows.
pub struct ToolbarProps {
    pub has_previous: bool,
    pub has_next: bool,
    /// A file is open, so it can be shown in the list.
    pub has_selected: bool,
    /// Batch mode is on: its toggle is latched.
    pub batch_mode: bool,
    /// A job runs: the batch toggle waits for it.
    pub batch_running: bool,
    /// The recent folders list is open: its button is latched.
    pub recent_open: bool,
    /// A newer frename version, when the update check found one: the dot on Settings.
    pub update_available: Option<String>,
    /// The file list's width, which says what folds into More (§13.9).
    pub width: f32,
    /// The More menu is open: its button is latched.
    pub more_open: bool,
}

const PADDING: Padding = Padding {
    top: 0.0,
    bottom: 0.0,
    left: SPACE_S,
    right: SPACE_S,
};

/// Render the toolbar. Buttons that cannot act now are disabled where they are.
pub fn view(props: ToolbarProps) -> Element<'static, folder::Message> {
    use folder::Message as M;
    let fold = Fold::for_width(props.width);
    let previous = IconButton::new(Icon::ChevronLeft)
        .tip(
            Tip::new(fl!("folder-controls-previous")).keys(&["PgUp"]),
            Position::Top,
        )
        .on_press_maybe(props.has_previous.then_some(M::PreviousFile));
    let next = IconButton::new(Icon::ChevronRight)
        .tip(
            Tip::new(fl!("folder-controls-next")).keys(&["PgDn"]),
            Position::Top,
        )
        .on_press_maybe(props.has_next.then_some(M::NextFile));
    let locate = IconButton::new(Icon::LocateFixed)
        .tip(fl!("folder-controls-scroll"), Position::Top)
        .on_press_maybe(props.has_selected.then_some(M::ScrollToSelected));
    // A click picks a folder; a right-click picks one file, which takes no room in the bar.
    let open = mouse_area(
        IconButton::new(Icon::FolderOpen)
            .tip(
                Tip::new(fl!("folder-controls-open")).detail(fl!("folder-controls-open-file")),
                Position::Top,
            )
            .on_press(M::OpenFolder),
    )
    .on_right_press(M::OpenFile);
    // The recent folders, right after the open button (they are one group).
    let recent = IconButton::new(Icon::ChevronDown)
        .latched(props.recent_open)
        .tip(
            Tip::new(fl!("recent-folders-tip")).keys(&["Ctrl", "R"]),
            Position::Top,
        )
        .on_press_maybe((!props.batch_running).then_some(M::ToggleRecentFolders));

    let settings_tip = match &props.update_available {
        Some(version) => fl!(
            "folder-controls-update-available",
            version = version.clone()
        ),
        None => fl!("settings-window-title"),
    };
    let settings = IconButton::new(Icon::Settings)
        .dot(props.update_available.is_some())
        .tip(settings_tip, Position::Top)
        .on_press(M::OpenSettings);
    // Esc leaves batch mode (`EscapePressed`), but not while its job runs.
    let batch_tip = batch_toggle_tip(props.batch_mode, props.batch_running);
    let batch = IconButton::new(Icon::ListChecks)
        .latched(props.batch_mode)
        .tip(batch_tip, Position::Top)
        .on_press_maybe((!props.batch_running).then_some(M::SetBatchMode(!props.batch_mode)));

    let more = IconButton::new(Icon::Ellipsis)
        .latched(props.more_open)
        .tip(Tip::new(fl!("folder-controls-more")), Position::Top)
        .on_press(M::ToggleToolbarMore);

    // Groups 12 px apart (4 when narrow); the buttons of a group touch. What the width has no
    // room for is in More.
    let mut files = row![Element::from(previous), Element::from(next)];
    if fold.locate {
        files = files.push(Element::from(locate));
    }
    let mut opening = row![open];
    if fold.recent {
        opening = opening.push(Element::from(recent));
    }
    let mut app = row![Element::from(settings)];
    if fold.batch {
        app = app.push(Element::from(batch));
    }
    if fold.has_more() {
        app = app.push(Element::from(more));
    }
    let bar = row![files, opening, space::horizontal(), app]
        .spacing(fold.gap())
        .align_y(Alignment::Center);
    // A line on top: the rows above it scroll.
    column![
        container(space())
            .width(Length::Fill)
            .height(LINE)
            .style(style::divider),
        container(bar)
            .padding(PADDING)
            .width(Length::Fill)
            .center_y(BAR_HEIGHT)
            .style(style::panel),
    ]
    .into()
}

/// Where the More menu's left edge is in a window `window_width` wide: its right edge on the
/// toolbar's right padding, moved in to the list's left padding when the list is narrower than
/// the menu, and in from the window's edge.
// Coupled to the workspace layout: the list's column starts after the video pane (`left_width`)
// and a splitter (see `folder_workspace::view`).
fn more_left(window_width: f32, left_width: f32, folder_width: f32) -> f32 {
    let list_left = left_width + SPLITTER_HIT;
    let x = (list_left + folder_width - SPACE_S - MENU_WIDTH_WIDE).max(list_left + SPACE_S);
    x.min(window_width - MENU_WIDTH_WIDE).max(0.0)
}

/// The More menu over the window, above the toolbar: the buttons the list's width has no room
/// for, with their names and keys. Nothing while it is closed or nothing is folded.
pub fn more_menu(
    props: &ToolbarProps,
    left_width: f32,
    folder_width: f32,
) -> Element<'static, folder::Message> {
    use folder::Message as M;
    let fold = Fold::for_width(folder_width);
    let folded = fold.in_more();
    if !props.more_open || folded.is_empty() {
        return space().into();
    }
    let items: Vec<MenuItem<M>> = folded
        .iter()
        .map(|item| match item {
            Folded::Locate => MenuItem {
                icon: Some(Icon::LocateFixed),
                label: fl!("folder-controls-scroll"),
                keys: Vec::new(),
                checked: false,
                on_press: props.has_selected.then_some(M::ScrollToSelected),
            },
            Folded::Recent => MenuItem {
                icon: Some(Icon::ChevronDown),
                label: fl!("recent-folders-tip"),
                keys: vec!["Ctrl", "R"],
                checked: props.recent_open,
                on_press: (!props.batch_running).then_some(M::ToggleRecentFolders),
            },
            Folded::Batch => {
                let tip = batch_toggle_tip(props.batch_mode, props.batch_running);
                MenuItem {
                    icon: Some(Icon::ListChecks),
                    label: tip.label,
                    keys: tip.keys,
                    checked: props.batch_mode,
                    on_press: (!props.batch_running).then_some(M::SetBatchMode(!props.batch_mode)),
                }
            }
        })
        .collect();
    responsive(move |area| {
        let rows: Vec<Element<'static, M>> = items_to_rows(&items);
        // A click beside the menu closes it and does nothing more.
        let beside = mouse_area(space().width(Length::Fill).height(Length::Fill))
            .on_press(M::CloseToolbarMore)
            .on_right_press(M::CloseToolbarMore)
            .on_middle_press(M::CloseToolbarMore);
        // Anchored by its bottom edge on the toolbar's line, so a name that wraps grows it
        // upward and never over the toolbar.
        let popup = container(menu::menu(rows, Length::Fixed(MENU_WIDTH_WIDE)))
            .width(Length::Fill)
            .height(Length::Fill)
            .align_bottom(Length::Fill)
            .padding(Padding {
                left: more_left(area.width, left_width, folder_width),
                bottom: LINE + BAR_HEIGHT,
                ..Padding::ZERO
            });
        stack![opaque(beside), popup]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    })
    .into()
}

fn items_to_rows(items: &[MenuItem<folder::Message>]) -> Vec<Element<'static, folder::Message>> {
    items
        .iter()
        .map(|item| {
            menu::item(MenuItem {
                icon: item.icon,
                label: item.label.clone(),
                keys: item.keys.clone(),
                checked: item.checked,
                on_press: item.on_press.clone(),
            })
        })
        .collect()
}

/// The batch toggle's tip: its name, and `Esc` when `Esc` would leave batch mode.
fn batch_toggle_tip(batch_mode: bool, batch_running: bool) -> Tip {
    let tip = Tip::new(if batch_mode {
        fl!("folder-controls-batch-back")
    } else {
        fl!("folder-controls-batch")
    });
    if batch_mode && !batch_running {
        tip.keys(&["Esc"])
    } else {
        tip
    }
}

#[cfg(test)]
mod more_tests {
    use super::*;

    #[test]
    fn the_menu_ends_at_the_toolbars_right_padding_and_keeps_inside_the_list_when_it_can() {
        let list_left = 440.0 + SPLITTER_HIT;
        assert_eq!(
            more_left(1440.0, 440.0, 200.0),
            list_left + SPACE_S,
            "a narrow list: its left edge"
        );
        assert_eq!(
            more_left(1440.0, 440.0, 500.0) + MENU_WIDTH_WIDE,
            list_left + 500.0 - SPACE_S
        );
        assert_eq!(more_left(600.0, 440.0, 200.0), 600.0 - MENU_WIDTH_WIDE);
    }
}

#[cfg(test)]
mod batch_tip_tests {
    use super::*;

    #[test]
    fn the_batch_toggle_tip_names_the_way_back_and_shows_esc_only_when_idle() {
        let enter = batch_toggle_tip(false, false);
        assert_eq!(enter.label, fl!("folder-controls-batch"));
        assert!(enter.keys.is_empty());
        let back = batch_toggle_tip(true, false);
        assert_eq!(back.label, fl!("folder-controls-batch-back"));
        assert_eq!(back.keys, vec!["Esc"]);
        let running = batch_toggle_tip(true, true);
        assert_eq!(running.label, fl!("folder-controls-batch-back"));
        assert!(running.keys.is_empty());
    }
}
