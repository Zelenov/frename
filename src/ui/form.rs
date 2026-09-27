//! Form controls (`docs/design/design-system.md` §8.2–8.7): one checkbox, one radio, one field,
//! one dropdown look in the whole app.

use std::borrow::Borrow;

use iced::widget::text::{IntoFragment, LineHeight};
use iced::widget::{
    checkbox as iced_checkbox, column, mouse_area, pick_list, radio, text_input, Checkbox,
    PickList, TextInput,
};
use iced::{Element, Padding, Pixels};

use super::layout;
use super::style;
use super::text;
use super::tokens::*;

/// Text of controls: 20-px lines.
const LINE: LineHeight = LineHeight::Absolute(Pixels(LINE_BODY));

/// 28 px high with a 20-px line of text.
const FIELD_PADDING: Padding = Padding {
    top: CONTROL_PADDING_Y,
    bottom: CONTROL_PADDING_Y,
    left: SPACE_S,
    right: SPACE_S,
};

/// A checkbox; the caller adds `on_toggle` (without it, it is disabled).
pub fn checkbox<'a, M>(label: impl IntoFragment<'a>, checked: bool) -> Checkbox<'a, M> {
    iced_checkbox(checked)
        .label(label)
        .size(CHECK_SIZE)
        .spacing(SPACE_S)
        .text_size(TEXT_BODY)
        .text_line_height(LINE)
        .font(FONT)
        .style(style::checkbox)
}

/// A checkbox with one line under it, level with its label.
pub fn checkbox_with_hint<'a, M: 'a>(
    checkbox: impl Into<Element<'a, M>>,
    hint: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    column![checkbox.into(), layout::indented(hint)].into()
}

/// One option of a radio group, with an optional description under its name (§8.3). A click on
/// the description picks the option too.
pub fn radio_option<'a, V, M>(
    label: impl Into<String>,
    description: Option<Element<'a, M>>,
    value: V,
    selected: Option<V>,
    on_select: impl FnOnce(V) -> M,
) -> Element<'a, M>
where
    V: Copy + Eq,
    M: Clone + 'a,
{
    let message = on_select(value);
    let choice = radio(label.into(), value, selected, |_| message.clone())
        .size(CHECK_SIZE)
        .spacing(SPACE_S)
        .text_size(TEXT_BODY)
        .text_line_height(LINE)
        .font(FONT)
        .style(style::radio);
    let Some(description) = description else {
        return choice.into();
    };
    column![
        choice,
        mouse_area(layout::indented(description)).on_press(message.clone()),
    ]
    .into()
}

/// The description of a radio option: one line of secondary text.
pub fn description<'a, M: 'a>(content: impl IntoFragment<'a>) -> Element<'a, M> {
    text::secondary(content).into()
}

/// An example of a name or a timecode, as the description of a radio option.
pub fn example<'a, M: 'a>(content: impl IntoFragment<'a>) -> Element<'a, M> {
    text::mono(content).into()
}

/// A text field; the caller adds `on_input` (without it, it is disabled) and its width.
pub fn text_field<'a, M: Clone + 'a>(placeholder: &str, value: &str) -> TextInput<'a, M> {
    text_input(placeholder, value)
        .size(TEXT_BODY)
        .line_height(LINE)
        .padding(FIELD_PADDING)
        .font(FONT)
        .style(style::text_input)
}

/// A dropdown of `options` (§8.7).
pub fn dropdown<'a, T, L, V, M>(
    options: L,
    selected: Option<V>,
    on_select: impl Fn(T) -> M + 'a,
) -> PickList<'a, T, L, V, M>
where
    T: ToString + PartialEq + Clone + 'a,
    L: Borrow<[T]> + 'a,
    V: Borrow<T> + 'a,
    M: Clone + 'a,
{
    pick_list(options, selected, on_select)
        .text_size(TEXT_BODY)
        .text_line_height(LINE)
        .padding(FIELD_PADDING)
        .font(FONT)
        .style(style::pick_list)
        .menu_style(style::menu)
}
