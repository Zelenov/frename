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

/// Segment range highlight on the progress bar (inverted accent – warm yellow-green).
pub const SEGMENT: Color = Color::from_rgba(1.0, 0.85, 0.2, 0.75);

/// Secondary text on dark translucent surfaces (subtitle list); brighter than TEXT_MUTED.
pub const TEXT_SOFT: Color = Color::from_rgb(0.75, 0.75, 0.8);

/// Splitter bar (matches track).
pub const VOLUME: Color = Color::from_rgb(0.35, 0.80, 0.50);

pub const SPLITTER: Color = Color::from_rgb(0.25, 0.25, 0.25);

/// Splitter when hovered or dragged.
pub const SPLITTER_ACTIVE: Color = Color::from_rgb(0.40, 0.40, 0.40);

/// Container style for panels (tag list, folder list, controls bars).
pub fn panel_container_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(BG_PANEL)),
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
pub fn elevated_container_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(BG_ELEVATED)),
        ..Default::default()
    }
}

/// Container style for elevated surfaces with a visible border (e.g. search bar).
pub fn elevated_container_bordered_style(_theme: &iced::Theme) -> iced::widget::container::Style {
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

/// Fullscreen subtitle caption: dark translucent pill so text reads over any picture.
pub fn subtitle_caption_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgba(0.0, 0.0, 0.0, 0.62))),
        border: iced::border::rounded(8),
        ..Default::default()
    }
}

/// Fullscreen subtitle list panel: translucent so the picture stays visible behind it.
pub fn subtitle_list_style(_theme: &iced::Theme) -> iced::widget::container::Style {
    iced::widget::container::Style {
        background: Some(Background::Color(Color::from_rgba(0.06, 0.06, 0.06, 0.72))),
        ..Default::default()
    }
}

/// Row in the subtitle list: accent tint for the cue on screen, hover highlight otherwise.
pub fn cue_row_style(
    active: bool,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + Clone {
    move |_theme: &iced::Theme, status: iced::widget::button::Status| {
        let bg = match (active, status) {
            (true, _) => ACCENT_TAG_ROW,
            (false, iced::widget::button::Status::Hovered) => Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            _ => Color::TRANSPARENT,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color: TEXT,
            border: iced::border::rounded(4),
            shadow: iced::Shadow::default(),
            snap: true,
        }
    }
}

/// Container style for main app background.
pub fn main_container_style(_theme: &iced::Theme) -> iced::widget::container::Style {
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

/// Entry of a vertical list of choices (batch actions): the selected one has the accent
/// background, a disabled one muted text.
pub fn list_item_button_style(
    selected: bool,
    enabled: bool,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + Clone {
    move |_theme: &iced::Theme, status: iced::widget::button::Status| {
        let bg = match status {
            _ if selected => ACCENT_SELECTED,
            iced::widget::button::Status::Hovered if enabled => TRACK,
            _ => Color::TRANSPARENT,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color: if enabled { TEXT } else { TEXT_MUTED },
            border: iced::Border {
                radius: 3.0.into(),
                ..iced::Border::default()
            },
            shadow: iced::Shadow::default(),
            snap: true,
        }
    }
}

/// Check box tinted with a job outcome (green for done, red for failed): the theme's check box
/// with `color` in place of the accent.
pub fn outcome_checkbox_style(
    color: Color,
) -> impl Fn(&iced::Theme, iced::widget::checkbox::Status) -> iced::widget::checkbox::Style {
    move |theme: &iced::Theme, status: iced::widget::checkbox::Status| {
        let style = iced::widget::checkbox::primary(theme, status);
        iced::widget::checkbox::Style {
            background: Background::Color(color),
            icon_color: Color::WHITE,
            border: iced::Border {
                color,
                ..style.border
            },
            ..style
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
            border: iced::border::rounded(u32::MAX).width(1).color(TEXT_MUTED),
            shadow: iced::Shadow::default(),
            icon: TEXT_MUTED,
        },
    }
}

/// How a clip marker's color is drawn: close to Premiere's marker colors, gray for a value
/// frename does not know.
pub fn marker_color(color: frename_core::MarkerColor) -> Color {
    use frename_core::MarkerColor as M;
    match color {
        M::Green => Color::from_rgb(0.36, 0.76, 0.36),
        M::Red => Color::from_rgb(0.86, 0.22, 0.22),
        M::Orange => Color::from_rgb(0.93, 0.55, 0.15),
        M::Yellow => Color::from_rgb(0.93, 0.85, 0.20),
        M::White => Color::from_rgb(0.95, 0.95, 0.95),
        M::Blue => Color::from_rgb(0.28, 0.48, 0.96),
        M::Cyan => Color::from_rgb(0.15, 0.78, 0.86),
        M::Lavender => Color::from_rgb(0.70, 0.58, 0.94),
        M::Magenta => Color::from_rgb(0.92, 0.28, 0.78),
        M::Other(_) => TEXT_MUTED,
    }
}

/// A round button filled with a marker color: the color dot of a marker row and the colors of
/// its picker. `selected` rings it in white.
pub fn marker_dot_style(
    color: Color,
    selected: bool,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + Clone {
    move |_theme: &iced::Theme, status: iced::widget::button::Status| {
        let ring = match status {
            _ if selected => TEXT,
            iced::widget::button::Status::Hovered => TEXT_SOFT,
            _ => Color::TRANSPARENT,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(color)),
            text_color: TEXT,
            border: iced::Border {
                radius: 999.0.into(),
                width: 2.0,
                color: ring,
            },
            shadow: iced::Shadow::default(),
            snap: true,
        }
    }
}

/// Tab of the side list over the video (Subtitles / Markers): the open one underlined in the
/// accent color.
pub fn overlay_tab_style(
    active: bool,
) -> impl Fn(&iced::Theme, iced::widget::button::Status) -> iced::widget::button::Style + Clone {
    move |_theme: &iced::Theme, status: iced::widget::button::Status| {
        let bg = match (active, status) {
            (true, _) => ACCENT_SELECTED,
            (false, iced::widget::button::Status::Hovered) => Color::from_rgba(1.0, 1.0, 1.0, 0.08),
            _ => Color::TRANSPARENT,
        };
        iced::widget::button::Style {
            background: Some(Background::Color(bg)),
            text_color: if active { TEXT } else { TEXT_SOFT },
            border: iced::border::rounded(4),
            shadow: iced::Shadow::default(),
            snap: true,
        }
    }
}

/// The marker label over the progress bar: a pill on the picture that renames the marker.
pub fn marker_label_style(
    _theme: &iced::Theme,
    status: iced::widget::button::Status,
) -> iced::widget::button::Style {
    let bg = match status {
        iced::widget::button::Status::Hovered | iced::widget::button::Status::Pressed => {
            ACCENT_SELECTED
        }
        _ => BG_ELEVATED,
    };
    iced::widget::button::Style {
        background: Some(Background::Color(bg)),
        text_color: TEXT,
        border: iced::border::rounded(4),
        shadow: iced::Shadow::default(),
        snap: true,
    }
}
