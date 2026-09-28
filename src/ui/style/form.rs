//! Looks of the form controls (`docs/design/design-system.md` §8.2–8.7, §8.15, §13.5.2): one
//! checkbox, one radio, one field, one dropdown, one progress bar in the whole app.

use iced::widget::{self, overlay::menu};
use iced::{Background, Border, Color, Theme};

use super::popup_shadow;
use crate::ui::tokens::*;

pub fn checkbox(_theme: &Theme, status: widget::checkbox::Status) -> widget::checkbox::Style {
    use widget::checkbox::Status::{Active, Disabled, Hovered};
    let (checked, hovered, disabled) = match status {
        Active { is_checked } => (is_checked, false, false),
        Hovered { is_checked } => (is_checked, true, false),
        Disabled { is_checked } => (is_checked, false, true),
    };
    let (background, edge) = match (checked, hovered, disabled) {
        (true, _, true) => (faded(ACCENT, DISABLED_ALPHA), Color::TRANSPARENT),
        (false, _, true) => (BG_PANEL, BORDER_SUBTLE),
        (true, true, _) => (ACCENT_HOVER, ACCENT_HOVER),
        (true, false, _) => (ACCENT, ACCENT),
        (false, true, _) => (BG_RAISED, TEXT_SECONDARY),
        (false, false, _) => (BG_RAISED, BORDER_CONTROL),
    };
    widget::checkbox::Style {
        background: Background::Color(background),
        icon_color: if disabled {
            faded(TEXT_ON_FILL, DISABLED_ALPHA)
        } else {
            TEXT_ON_FILL
        },
        border: Border {
            color: edge,
            width: LINE,
            radius: RADIUS_CHECK.into(),
        },
        text_color: Some(if disabled { TEXT_DISABLED } else { TEXT }),
    }
}

/// The checkbox on a tag chip (§13.5.2): white and black read on all 16 light colors, where the
/// app's blue checkbox does not.
pub fn chip_checkbox(_theme: &Theme, status: widget::checkbox::Status) -> widget::checkbox::Style {
    use widget::checkbox::Status::{Active, Disabled, Hovered};
    let checked = match status {
        Active { is_checked } | Hovered { is_checked } | Disabled { is_checked } => is_checked,
    };
    widget::checkbox::Style {
        background: Background::Color(if checked {
            CHIP_CHECK_ON
        } else {
            CHIP_CHECK_OFF
        }),
        icon_color: CHIP_CHECK_TICK,
        border: Border {
            color: if checked {
                CHIP_CHECK_ON
            } else {
                CHIP_CHECK_EDGE
            },
            width: LINE,
            radius: RADIUS_CHECK.into(),
        },
        text_color: Some(TAG_TEXT),
    }
}

pub fn radio(_theme: &Theme, status: widget::radio::Status) -> widget::radio::Style {
    let (selected, hovered) = match status {
        widget::radio::Status::Active { is_selected } => (is_selected, false),
        widget::radio::Status::Hovered { is_selected } => (is_selected, true),
    };
    widget::radio::Style {
        background: Background::Color(if selected { ACCENT } else { BG_RAISED }),
        dot_color: TEXT_ON_FILL,
        border_width: LINE,
        border_color: match (selected, hovered) {
            (true, _) => ACCENT,
            (false, true) => TEXT_SECONDARY,
            (false, false) => BORDER_CONTROL,
        },
        text_color: Some(TEXT),
    }
}

pub fn text_input(theme: &Theme, status: widget::text_input::Status) -> widget::text_input::Style {
    field(false)(theme, status)
}

/// A text field; `invalid` draws its edge in `ERROR` (§8.6, a message goes under it).
pub fn field(
    invalid: bool,
) -> impl Fn(&Theme, widget::text_input::Status) -> widget::text_input::Style {
    move |_theme, status| {
        use widget::text_input::Status::{Active, Disabled, Focused, Hovered};
        let (background, edge, width, value) = match status {
            Disabled => (BG_PANEL, BORDER_SUBTLE, LINE, TEXT_DISABLED),
            Focused { .. } => (
                BG_RAISED,
                if invalid { ERROR } else { ACCENT_TEXT },
                RING,
                TEXT,
            ),
            _ if invalid => (BG_RAISED, ERROR, LINE, TEXT),
            Hovered => (BG_RAISED, TEXT_SECONDARY, LINE, TEXT),
            Active => (BG_RAISED, BORDER_CONTROL, LINE, TEXT),
        };
        widget::text_input::Style {
            background: Background::Color(background),
            border: Border {
                color: edge,
                width,
                radius: RADIUS_S.into(),
            },
            icon: TEXT_SECONDARY,
            placeholder: TEXT_PLACEHOLDER,
            value,
            selection: SELECTED,
        }
    }
}

/// A multi-line field (the comment, a marker's name): the text field's look.
pub fn text_editor(
    _theme: &Theme,
    status: widget::text_editor::Status,
) -> widget::text_editor::Style {
    use widget::text_editor::Status::{Active, Disabled, Focused, Hovered};
    let (background, edge, width, value) = match status {
        Active => (BG_RAISED, BORDER_CONTROL, LINE, TEXT),
        Hovered => (BG_RAISED, TEXT_SECONDARY, LINE, TEXT),
        Focused { .. } => (BG_RAISED, ACCENT_TEXT, RING, TEXT),
        Disabled => (BG_PANEL, BORDER_SUBTLE, LINE, TEXT_DISABLED),
    };
    widget::text_editor::Style {
        background: Background::Color(background),
        border: Border {
            color: edge,
            width,
            radius: RADIUS_S.into(),
        },
        placeholder: TEXT_PLACEHOLDER,
        value,
        selection: SELECTED,
    }
}

pub fn pick_list(_theme: &Theme, status: widget::pick_list::Status) -> widget::pick_list::Style {
    let edge = match status {
        widget::pick_list::Status::Active => BORDER_CONTROL,
        widget::pick_list::Status::Hovered => TEXT_SECONDARY,
        widget::pick_list::Status::Opened { .. } => ACCENT_TEXT,
    };
    widget::pick_list::Style {
        text_color: TEXT,
        placeholder_color: TEXT,
        handle_color: TEXT_SECONDARY,
        background: Background::Color(BG_RAISED),
        border: Border {
            color: edge,
            width: LINE,
            radius: RADIUS_S.into(),
        },
    }
}

/// The list of a dropdown and every menu (§8.14).
pub fn menu(_theme: &Theme) -> menu::Style {
    menu::Style {
        background: Background::Color(BG_OVERLAY),
        border: Border {
            color: BORDER_SUBTLE,
            width: LINE,
            radius: RADIUS_M.into(),
        },
        text_color: TEXT,
        selected_text_color: TEXT,
        selected_background: Background::Color(SELECTED),
        shadow: popup_shadow(),
    }
}

/// The progress bar of a job (§8.15).
pub fn progress(_theme: &Theme) -> widget::progress_bar::Style {
    widget::progress_bar::Style {
        background: Background::Color(BG_RAISED),
        bar: Background::Color(ACCENT),
        border: Border {
            radius: RADIUS_CHECK.into(),
            ..Border::default()
        },
    }
}
