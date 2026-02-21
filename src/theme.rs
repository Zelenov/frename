//! Shared UI theme: colors and styling aligned with the video progress bar.

use iced::widget::scrollable::{AutoScroll, Rail, Scroller, Status};
use iced::{Background, Color};

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

/// Tag row selected background: same hue as ACCENT / selected chip border, higher opacity so it matches the border.
pub const ACCENT_TAG_ROW: Color = Color::from_rgba(0.35, 0.65, 1.0, 0.45);

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

/// Container style for panels (tag list, folder list, controls bars).
pub fn panel_container_style(
    _theme: &iced::Theme,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL)),
        ..Default::default()
    }
}

/// Container style for selectable rows (selected vs default background).
pub fn row_background_style(
    _theme: &iced::Theme,
    selected: bool,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(if selected {
            ACCENT_SELECTED
        } else {
            BG_PANEL
        })),
        ..Default::default()
    }
}

/// Container style for tag list/grid rows: selected = accent tint matching selected chip border opacity, unselected = panel.
pub fn tag_row_background_style(
    _theme: &iced::Theme,
    selected: bool,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(if selected {
            ACCENT_TAG_ROW
        } else {
            BG_PANEL
        })),
        ..Default::default()
    }
}

/// Container style for folder list rows (selected vs transparent).
pub fn selectable_row_style(
    _theme: &iced::Theme,
    selected: bool,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(if selected {
            ACCENT_SELECTED
        } else {
            Color::TRANSPARENT
        })),
        ..Default::default()
    }
}

/// Container style for elevated surfaces (file name area).
pub fn elevated_container_style(
    _theme: &iced::Theme,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(BG_ELEVATED)),
        ..Default::default()
    }
}

/// Container style for elevated surfaces with a visible border (e.g. search bar).
pub fn elevated_container_bordered_style(
    _theme: &iced::Theme,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(BG_ELEVATED)),
        border: iced::Border {
            radius: 4.0.into(),
            width: 1.0,
            color: TEXT_MUTED,
        },
        ..Default::default()
    }
}

/// Container style for main app background.
pub fn main_container_style(
    _theme: &iced::Theme,
) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(BG_MAIN)),
        ..Default::default()
    }
}

/// Icon button style (prev/next, video controls). When disabled, uses transparent bg and muted text.
pub fn icon_button_style(
    enabled: bool,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + Clone {
    move |_theme: &iced::Theme, status: iced::widget::button::Status| {
        let (bg, text_color) = if enabled {
            let bg = match status {
                iced::widget::button::Status::Hovered => TRACK,
                iced::widget::button::Status::Pressed => SPLITTER_ACTIVE,
                _ => Color::TRANSPARENT,
            };
            (bg, TEXT)
        } else {
            (Color::TRANSPARENT, TEXT_MUTED)
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color,
            border: iced::Border::default(),
            shadow: iced::Shadow::default(),
            snap: true,
        }
    }
}

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
