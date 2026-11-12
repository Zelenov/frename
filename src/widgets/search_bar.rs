//! The search bar over a list (`docs/design/design-system.md` §13.4.1, §13.5.1): a search field
//! across the column and, after it, what else narrows the same list (the file filter). Stateless:
//! the parent holds the text.

use iced::widget::{container, row, Id};
use iced::{Alignment, Element, Length, Padding};

use crate::ui::tokens::*;
use crate::ui::{form, text};

/// Widget id for the tag search bar text input (for focus and global key capture).
pub const SEARCH_BAR_INPUT_ID: &str = "search-bar-input";

/// Widget id for the file search bar text input. Every bar needs its own id, otherwise a
/// focus operation would target both of them.
pub const FILE_SEARCH_BAR_INPUT_ID: &str = "file-search-bar-input";

/// A 40-px bar: 6 px around a 28-px field.
const PADDING: Padding = Padding {
    top: SPACE_TIGHT,
    bottom: SPACE_TIGHT,
    left: SPACE_S,
    right: SPACE_S,
};

/// What a search bar shows and does.
pub struct SearchBar<'a, M> {
    /// The field's id, for focus.
    pub input_id: &'static str,
    /// An example of what to type ("Find a file").
    pub placeholder: String,
    pub value: &'a str,
    /// The tooltip of the clear button.
    pub clear_tip: String,
    pub on_clear: M,
    /// Enter: always handled, so Windows does not beep on an unhandled Enter.
    pub on_submit: M,
    /// After the field, 6 px apart: what else narrows the same list.
    pub trailing: Option<Element<'a, M>>,
}

/// Render `bar`; typing sends `on_input`.
pub fn view<'a, M: Clone + 'a>(
    bar: SearchBar<'a, M>,
    on_input: impl Fn(String) -> M + 'a,
) -> Element<'a, M> {
    let on_input = std::rc::Rc::new(on_input);
    let SearchBar {
        input_id,
        placeholder,
        value,
        clear_tip,
        on_clear,
        on_submit,
        trailing,
    } = bar;
    // The placeholder is cut with "…" at the room the field really has, never mid-word (#393).
    let search = iced::widget::responsive(move |size| {
        let room = size.width - 2.0 * (form::SEARCH_ICON_ROOM + LINE);
        let on_input = on_input.clone();
        let field = form::text_field(&text::fit_body(&placeholder, room), value)
            .id(Id::from(input_id))
            .on_input(move |typed| on_input(typed))
            .on_submit(on_submit.clone());
        form::search_field(
            field,
            !value.is_empty(),
            on_clear.clone(),
            clear_tip.clone(),
        )
    });
    container(
        row![container(search).width(Length::Fill).height(CONTROL_HEIGHT)]
            .push(trailing)
            .spacing(SPACE_TIGHT)
            .align_y(Alignment::Center),
    )
    .padding(PADDING)
    .width(Length::Fill)
    .height(SEARCH_BAR_HEIGHT)
    .into()
}
