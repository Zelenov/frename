//! What the video pane shows without a picture (design system §13.3.1): no clip, loading, and a
//! clip that cannot be played. Every one says what is going on in words.

use iced::widget::{column, container};
use iced::{Alignment, Element, Length};

use crate::ui::empty;
use crate::ui::icons::{spinner, Icon};
use crate::ui::style;
use crate::ui::text;
use crate::ui::tokens::*;

/// Spinner steps after which a slow load says so: 3 s.
const SLOW_LOAD_TICKS: usize = 20;

/// No file is open.
pub fn no_clip<'a, M: 'a>() -> Element<'a, M> {
    panel(empty::pane(
        Icon::Clapperboard,
        fl!("media-viewer-no-clip"),
        None,
        None,
    ))
}

/// A file is being opened: the spinner and its name on black, where the picture will be.
pub fn loading<'a, M: 'a>(name: Option<String>, ticks: usize) -> Element<'a, M> {
    let slow =
        (ticks >= SLOW_LOAD_TICKS).then(|| text::secondary(fl!("media-viewer-loading-slow")));
    let block = column![spinner(ticks, ICON_L, TEXT_SECONDARY)]
        .push(name.map(text::mono))
        .push(slow)
        .spacing(SPACE_S)
        .align_x(Alignment::Center);
    container(block)
        .center(Length::Fill)
        .style(style::video)
        .into()
}

/// The file cannot be played; `reason` says why when it is known.
pub fn cannot_play<'a, M: 'a>(reason: Option<String>) -> Element<'a, M> {
    panel(empty::pane_in(
        Icon::CircleX,
        ERROR,
        fl!("media-viewer-cannot-play"),
        reason,
        None,
    ))
}

fn panel<'a, M: 'a>(content: Element<'a, M>) -> Element<'a, M> {
    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::panel)
        .into()
}

/// A short note ("Frame saved") floating over the bottom left of the picture, when the controls
/// bar has no room for it (§13.3.5).
pub fn floating_notice<'a, M: 'a>(notice: &'a str) -> Element<'a, M> {
    container(
        container(text::body(notice))
            .max_width(TOOLTIP_MAX_WIDTH)
            .padding(iced::Padding {
                top: SPACE_XS,
                bottom: SPACE_XS,
                left: SPACE_S,
                right: SPACE_S,
            })
            .style(style::popup),
    )
    .padding(SPACE_S)
    .width(Length::Fill)
    .height(Length::Fill)
    .align_left(Length::Fill)
    .align_bottom(Length::Fill)
    .into()
}
