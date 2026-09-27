//! The text styles (`docs/design/design-system.md` §4.2). No other text style exists.

use iced::widget::text::{IntoFragment, LineHeight};
use iced::widget::{text, Text};
use iced::{Color, Font, Pixels};

use super::tokens::*;

fn styled<'a>(
    content: impl IntoFragment<'a>,
    size: f32,
    line: f32,
    font: Font,
    color: Color,
) -> Text<'a> {
    text(content)
        .size(size)
        .line_height(LineHeight::Absolute(Pixels(line)))
        .font(font)
        .color(color)
}

/// The label of a control that sets its own text color by state (a button): no color of its own.
pub fn label<'a>(content: impl IntoFragment<'a>, font: Font) -> Text<'a> {
    text(content)
        .size(TEXT_BODY)
        .line_height(LineHeight::Absolute(Pixels(LINE_BODY)))
        .font(font)
}

/// A page title.
pub fn heading<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_HEADING, LINE_HEADING, FONT_STRONG, TEXT)
}

/// Everything else: labels, values, lines of text.
pub fn body<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, FONT, TEXT)
}

/// The value that matters in a line.
pub fn strong<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, FONT_STRONG, TEXT)
}

/// Help, descriptions, meta.
pub fn secondary<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, FONT, TEXT_SECONDARY)
}

/// What went wrong, next to where it did.
pub fn error<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_BODY, LINE_BODY, FONT, ERROR)
}

/// File names, paths, timecodes, examples of names.
pub fn mono<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_MONO, LINE_BODY, FONT_MONO, TEXT_SECONDARY)
}

/// The text of a tooltip.
pub fn tooltip<'a>(content: impl IntoFragment<'a>) -> Text<'a> {
    styled(content, TEXT_TOOLTIP, LINE_TOOLTIP, FONT, TEXT)
}
