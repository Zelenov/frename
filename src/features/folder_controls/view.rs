//! The toolbar under the file list (`docs/design/design-system.md` §13.4.3): moving between files,
//! opening a folder, and, until the app bar of #64, Settings and the batch mode toggle. Receives
//! only what it shows.

use iced::widget::{column, container, mouse_area, row, space};
use iced::{Alignment, Element, Length, Padding};

use crate::features::folder;
use crate::ui::icon_button::IconButton;
use crate::ui::icons::Icon;
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
    /// A newer frename version, when the update check found one: the dot on Settings.
    pub update_available: Option<String>,
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
    // Esc leaves batch mode (`EscapePressed`).
    let batch_tip = if props.batch_mode {
        Tip::new(fl!("folder-controls-batch-back")).keys(&["Esc"])
    } else {
        Tip::new(fl!("folder-controls-batch"))
    };
    let batch = IconButton::new(Icon::ListChecks)
        .latched(props.batch_mode)
        .tip(batch_tip, Position::Top)
        .on_press_maybe((!props.batch_running).then_some(M::SetBatchMode(!props.batch_mode)));

    // Groups 12 px apart; the buttons of a group touch.
    let files = row![
        Element::from(previous),
        Element::from(next),
        Element::from(locate)
    ];
    let app = row![Element::from(settings), Element::from(batch)];
    let bar = row![files, open, space::horizontal(), app]
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
