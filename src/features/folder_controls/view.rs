//! The toolbar under the file list (`docs/design/design-system.md` §13.4.3): moving between files,
//! opening a folder. Settings and the mode switch are in the app bar (`app_bar`, #64). Receives
//! only what it shows.

use iced::widget::{column, container, mouse_area, row, space};
use iced::{Alignment, Element, Length, Padding};

use crate::features::folder;
use crate::ui::icon_button::IconButton;
use crate::ui::icons::Icon;
use crate::ui::segmented::{segmented, Segment};
use crate::ui::style;
use crate::ui::tokens::*;
use crate::ui::tooltip::{Position, Tip};

/// What the toolbar shows.
pub struct ToolbarProps {
    pub has_previous: bool,
    pub has_next: bool,
    /// A file is open, so it can be shown in the list.
    pub has_selected: bool,
    /// A job runs: the folder cannot change meanwhile, so the recent folders wait for it.
    pub batch_running: bool,
    /// The recent folders list is open: its button is latched.
    pub recent_open: bool,
}

/// The toolbar's side padding.
const PADDING: Padding = Padding {
    top: 0.0,
    bottom: 0.0,
    left: SPACE_S,
    right: SPACE_S,
};

/// Render the toolbar. Buttons that cannot act now are disabled where they are.
pub fn view(props: ToolbarProps) -> Element<'static, folder::Message> {
    use folder::Message as M;
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

    // Groups 12 px apart; the buttons of a group touch. All five fit the narrowest list.
    let files = row![
        Element::from(previous),
        Element::from(next),
        Element::from(locate)
    ];
    let opening = row![open, Element::from(recent)];
    let bar = row![files, opening]
        .spacing(SPACE_M)
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

/// What the app bar shows (§13.12, version B).
pub struct AppBarProps {
    /// Batch mode is on: the Batch segment is the selected one.
    pub batch_mode: bool,
    /// How many files are checked, shown after "Batch" while batch mode is on.
    pub checked: usize,
    /// A newer frename version, when the update check found one: the dot on Settings.
    pub update_available: Option<String>,
    /// The version just installed, until its release notes were seen: the dot on Settings too.
    pub whats_new: Option<String>,
}

/// The bar under the OS title bar: the mode switch (single file or batch) on the left, Settings
/// on the right. A running job goes on behind the closed batch panel (#249), so the switch is
/// never disabled. `Esc` still leaves batch mode (`EscapePressed`).
pub fn app_bar(props: AppBarProps) -> Element<'static, folder::Message> {
    use folder::Message as M;
    let mode = segmented([
        Segment {
            icon: Some(Icon::FileVideoCamera),
            label: fl!("app-bar-single"),
            count: None,
            selected: !props.batch_mode,
            on_press: M::SetBatchMode(false),
            tip: props
                .batch_mode
                .then(|| format!("{} (Esc)", fl!("folder-controls-batch-back"))),
        },
        Segment {
            icon: Some(Icon::ListChecks),
            label: fl!("app-bar-batch"),
            count: props.batch_mode.then_some(props.checked),
            selected: props.batch_mode,
            on_press: M::SetBatchMode(true),
            tip: (!props.batch_mode).then(|| fl!("folder-controls-batch")),
        },
    ]);
    let settings_tip = match (&props.update_available, &props.whats_new) {
        (Some(version), _) => fl!(
            "folder-controls-update-available",
            version = version.clone()
        ),
        (None, Some(version)) => fl!("folder-controls-whats-new", version = version.clone()),
        (None, None) => fl!("settings-window-title"),
    };
    let settings = IconButton::new(Icon::Settings)
        .dot(props.update_available.is_some() || props.whats_new.is_some())
        .tip(settings_tip, Position::Bottom)
        .on_press(M::OpenSettings);
    column![
        container(
            row![mode, space::horizontal(), Element::from(settings)].align_y(Alignment::Center)
        )
        .padding(Padding::from([0.0, SPACE_S]))
        .width(Length::Fill)
        .center_y(APP_BAR_HEIGHT - LINE)
        .style(style::window),
        container(space())
            .width(Length::Fill)
            .height(LINE)
            .style(style::divider),
    ]
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The toolbar's five buttons and its padding fit the narrowest file list.
    #[test]
    fn the_toolbar_fits_the_narrowest_list() {
        let width = PADDING.left + PADDING.right + 5.0 * BAR_HEIGHT + SPACE_M;
        assert!(width <= FILE_LIST_MIN_WIDTH, "{width}");
    }
}
