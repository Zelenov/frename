//! The title bar's view, one row of `TITLE_BAR_HEIGHT`: the folder and the drag space, the mode
//! switch and Settings, and the window buttons.

use iced::widget::{button, container, mouse_area, row, space, text::Wrapping};
use iced::{alignment, Alignment, Element, Length};

use super::Message;
use crate::ui::icons::{icon, Icon};
use crate::ui::style::{self, ButtonKind};
use crate::ui::text;
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position};

/// The folder the window shows: its name, and the folder it is in.
pub struct Folder {
    pub name: String,
    pub parent: String,
}

/// What the bar holds besides its own buttons, built by the workspace.
pub struct Parts<'a, M> {
    /// The mode switch (single file or batch), while a folder is open.
    pub mode: Option<Element<'a, M>>,
    pub folder: Option<Folder>,
    /// Settings, with its update dot.
    pub settings: Element<'a, M>,
}

/// The bar. `maximized` turns maximize into restore; `on` lifts the bar's own messages.
pub fn view<'a, M: Clone + 'a>(
    parts: Parts<'a, M>,
    maximized: bool,
    on: fn(Message) -> M,
) -> Element<'a, M> {
    // The folder is plain text in the drag space: the object's name first, then where it is.
    let label: Element<'a, M> = match parts.folder {
        Some(folder) => row![
            text::strong(folder.name).wrapping(Wrapping::None),
            text::caption(folder.parent)
                .wrapping(Wrapping::None)
                .color(TEXT_SECONDARY),
        ]
        .spacing(SPACE_S)
        .align_y(Alignment::Center)
        .into(),
        None => space().into(),
    };
    let caption = container(label)
        .padding([0.0, SPACE_M])
        .width(Length::Fill)
        .height(Length::Fill)
        .align_y(alignment::Vertical::Center)
        .clip(true);
    // Every pixel that is not a control moves the window: the buttons keep their own presses.
    let grip = mouse_area(caption)
        .on_press(on(Message::Pressed))
        .on_move(move |_| on(Message::Moved))
        .on_release(on(Message::Released))
        .on_double_click(on(Message::DoubleClicked))
        .on_right_press(on(Message::SystemMenu));

    let (size_icon, size_tip) = if maximized {
        (Icon::Copy, fl!("title-bar-restore"))
    } else {
        (Icon::Square, fl!("title-bar-maximize"))
    };
    // Always a child, empty without a folder, so the bar keeps its shape (and its widgets' state).
    // Beside Settings, the app's controls in one group on the right, with room around them.
    let mode = container(parts.mode.unwrap_or_else(|| space().into())).center_y(TITLE_BAR_HEIGHT);
    container(
        row![grip, mode]
            .push(space().width(SPACE_M))
            .push(parts.settings)
            .push(space().width(SPACE_M))
            .push(window_button(
                Icon::Minus,
                fl!("title-bar-minimize"),
                false,
                on(Message::Minimize),
            ))
            .push(window_button(
                size_icon,
                size_tip,
                false,
                on(Message::ToggleMaximize),
            ))
            .push(window_button(
                Icon::X,
                fl!("title-bar-close"),
                true,
                on(Message::Close),
            ))
            .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(TITLE_BAR_HEIGHT)
    .style(style::window)
    .into()
}

/// One window button: as wide as the system's own, the close one red under the pointer.
fn window_button<'a, M: Clone + 'a>(
    glyph: Icon,
    tip: String,
    close: bool,
    message: M,
) -> Element<'a, M> {
    let face = container(icon(glyph, ICON_S, TEXT))
        .center_x(Length::Fill)
        .center_y(Length::Fill);
    let button = button(face)
        .width(WINDOW_BUTTON_WIDTH)
        .height(TITLE_BAR_HEIGHT)
        .padding(0)
        .style(style::button(ButtonKind::WindowButton(close)))
        .on_press(message);
    tooltip::tip(button, tip, Position::Bottom)
}
