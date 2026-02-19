//! Shared UI theme: colors and styling aligned with the video progress bar.

use iced::widget::scrollable::{AutoScroll, Rail, Scroller, Status};
use iced::Color;

/// Main app background (dark).
pub const BG_MAIN: Color = Color::from_rgb(0.10, 0.10, 0.10);

/// Panel background (video area, controls bar).
pub const BG_PANEL: Color = Color::from_rgb(0.12, 0.12, 0.12);

/// Elevated surface (list track, inputs).
pub const BG_ELEVATED: Color = Color::from_rgb(0.18, 0.18, 0.18);

/// Track / secondary surface (progress bar track, splitter).
pub const TRACK: Color = Color::from_rgb(0.25, 0.25, 0.25);

/// Accent (progress fill, selection, highlights).
pub const ACCENT: Color = Color::from_rgb(0.35, 0.65, 1.0);

/// Selected row / hover (accent with transparency).
pub const ACCENT_SELECTED: Color = Color::from_rgba(0.35, 0.65, 1.0, 0.25);

/// Selected row / hover (accent with transparency).
pub const DEBUG: Color = Color::from_rgba(0.35, 0.0, 0.0, 1.0);

/// Primary text (off-white).
pub const TEXT: Color = Color::from_rgb(0.92, 0.92, 0.95);

/// Muted / placeholder text.
pub const TEXT_MUTED: Color = Color::from_rgb(0.55, 0.55, 0.6);

/// Error / failed state (e.g. video failed to load).
pub const ERROR: Color = Color::from_rgb(0.95, 0.35, 0.35);

/// Splitter bar (matches track).
pub const SPLITTER: Color = Color::from_rgb(0.25, 0.25, 0.25);

/// Splitter when hovered or dragged.
pub const SPLITTER_ACTIVE: Color = Color::from_rgb(0.40, 0.40, 0.40);

/// Dark scrollable style: dark track and scroller to match panels.
pub fn dark_scrollable_style(
    _theme: &iced::Theme,
    status: Status,
) -> iced::widget::scrollable::Style {
    let rail = Rail {
        background: Some(iced::Background::Color(TRACK)),
        border: iced::border::rounded(2),
        scroller: Scroller {
            background: iced::Background::Color(SPLITTER_ACTIVE),
            border: iced::border::rounded(2),
        },
    };
    let hovered_rail = Rail {
        scroller: Scroller {
            background: iced::Background::Color(ACCENT),
            ..rail.scroller
        },
        ..rail
    };
    let (vertical_rail, horizontal_rail) = match status {
        Status::Hovered {
            is_vertical_scrollbar_hovered,
            is_horizontal_scrollbar_hovered,
            ..
        } => (
            if is_vertical_scrollbar_hovered {
                hovered_rail
            } else {
                rail
            },
            if is_horizontal_scrollbar_hovered {
                hovered_rail
            } else {
                rail
            },
        ),
        Status::Dragged {
            is_vertical_scrollbar_dragged,
            is_horizontal_scrollbar_dragged,
            ..
        } => (
            if is_vertical_scrollbar_dragged {
                hovered_rail
            } else {
                rail
            },
            if is_horizontal_scrollbar_dragged {
                hovered_rail
            } else {
                rail
            },
        ),
        _ => (rail, rail),
    };
    iced::widget::scrollable::Style {
        container: iced::widget::container::Style::default(),
        vertical_rail,
        horizontal_rail,
        gap: None,
        auto_scroll: AutoScroll {
            background: iced::Background::Color(BG_ELEVATED),
            border: iced::border::rounded(u32::MAX)
                .width(1)
                .color(TEXT_MUTED),
            shadow: iced::Shadow::default(),
            icon: TEXT_MUTED,
        },
    }
}
