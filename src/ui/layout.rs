//! Layout pieces (`docs/design/design-system.md` §8–9): pages, rows, the navigation list, the
//! button bar, notices and help.

use iced::widget::text::IntoFragment;
use iced::widget::{button, column, container, row, scrollable, space, tooltip, Column};
use iced::{Alignment, Color, Element, Length, Padding};

use super::icons::{icon, Icon};
use super::style::{self, ButtonKind};
use super::text;
use super::tokens::*;

/// Where the first line of a control sits below the top of a 28-px control: labels and choice
/// groups move down by it so their text lines up with a field's text.
const CONTROL_TEXT_OFFSET: f32 = CONTROL_PADDING_Y;

/// A page: its heading, then its rows 24 px apart, padded from the window's edges.
pub fn page<'a, M: 'a>(
    heading: impl IntoFragment<'a>,
    rows: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    column![text::heading(heading)]
        .extend(rows)
        .spacing(SPACE_XL)
        .padding(Padding {
            top: PAGE_PADDING_Y,
            bottom: PAGE_PADDING_Y,
            left: SPACE_XL,
            right: SPACE_XL,
        })
        .width(Length::Fill)
        .into()
}

/// A window on the system (§9.2): navigation on the left, the scrolling page on the right, the
/// button bar at the bottom.
pub fn window<'a, M: 'a>(
    navigation: impl Into<Element<'a, M>>,
    page: impl Into<Element<'a, M>>,
    buttons: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    container(column![
        row![navigation.into(), vertical_line(), page.into()].height(Length::Fill),
        buttons.into(),
    ])
    .width(Length::Fill)
    .height(Length::Fill)
    .style(style::panel)
    .into()
}

/// The navigation column of a window: its items, 2 px apart, on the window's darkest surface.
pub fn sidebar<'a, M: 'a>(items: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    container(Column::with_children(items).spacing(SPACE_XXS))
        .width(NAV_WIDTH)
        .height(Length::Fill)
        .padding(SPACE_S)
        .style(style::window)
        .into()
}

/// `content` scrolling vertically; `id` lets a task scroll it.
pub fn scroll<'a, M: 'a>(id: &'static str, content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    let bar = scrollable::Scrollbar::new()
        .width(SCROLLBAR_WIDTH)
        .scroller_width(SCROLLBAR_WIDTH)
        .spacing(SCROLLBAR_GAP);
    scrollable(content)
        .direction(scrollable::Direction::Vertical(bar))
        .id(iced::widget::Id::new(id))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::scrollable)
        .into()
}

/// One option per row: the label in its column on the left, the controls on the right (§10.1).
pub fn setting_row<'a, M: 'a>(
    label: impl IntoFragment<'a>,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    labelled_row(text::body(label).into(), content.into())
}

/// A setting row whose label carries ⓘ with `help` (§8.11).
pub fn setting_row_with_info<'a, M: 'a>(
    label: impl IntoFragment<'a>,
    help: impl IntoFragment<'a>,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    let label = row![text::body(label), info(help)]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center);
    labelled_row(label.into(), content.into())
}

fn labelled_row<'a, M: 'a>(label: Element<'a, M>, content: Element<'a, M>) -> Element<'a, M> {
    row![
        container(label).width(LABEL_WIDTH).padding(Padding {
            top: CONTROL_TEXT_OFFSET,
            ..Padding::ZERO
        }),
        container(content).width(Length::Fill),
    ]
    .spacing(SPACE_L)
    .into()
}

/// The controls of a row, 8 px apart.
pub fn controls<'a, M: 'a>(items: impl IntoIterator<Item = Element<'a, M>>) -> Column<'a, M> {
    Column::with_children(items).spacing(SPACE_S)
}

/// A column of controls whose first line of text meets the row label's (checkboxes, radio
/// options, a line of text over buttons), 12 px apart.
pub fn aligned<'a, M: 'a>(items: impl IntoIterator<Item = Element<'a, M>>) -> Column<'a, M> {
    Column::with_children(items)
        .spacing(SPACE_M)
        .padding(Padding {
            top: CONTROL_TEXT_OFFSET,
            ..Padding::ZERO
        })
}

/// Content indented under a checkbox or a radio option, level with its label.
pub fn indented<'a, M: 'a>(content: impl Into<Element<'a, M>>) -> Element<'a, M> {
    container(content)
        .padding(Padding {
            left: CHOICE_INDENT,
            ..Padding::ZERO
        })
        .into()
}

/// An item of a navigation list: the page shown is marked by a bar, a fill and bold text.
pub fn nav_item<'a, M: Clone + 'a>(
    glyph: Icon,
    label: impl IntoFragment<'a>,
    selected: bool,
    dot: bool,
    on_press: M,
) -> Element<'a, M> {
    let color = if selected {
        ACCENT_TEXT
    } else {
        TEXT_SECONDARY
    };
    let label = if selected {
        text::strong(label)
    } else {
        text::secondary(label)
    };
    let mark = container(space())
        .width(SELECTION_BAR)
        .height(ICON_M)
        .style(style::fill(if selected {
            ACCENT_TEXT
        } else {
            Color::TRANSPARENT
        }));
    let mut content = row![mark, icon(glyph, ICON_M, color), label]
        .spacing(SPACE_S)
        .align_y(Alignment::Center);
    if dot {
        content = content.push(space::horizontal()).push(update_dot());
    }
    // 32 px for one line; a label that wraps (a long translation) makes the item taller.
    button(content)
        .width(Length::Fill)
        .padding(Padding {
            top: ROW_PADDING_Y,
            bottom: ROW_PADDING_Y,
            left: SPACE_XS,
            right: SPACE_M,
        })
        .style(style::button(ButtonKind::Nav(selected)))
        .on_press(on_press)
        .into()
}

/// The dot that says "an update is ready".
pub fn update_dot<'a, M: 'a>() -> Element<'a, M> {
    container(space())
        .width(DOT_SIZE)
        .height(DOT_SIZE)
        .style(style::fill(ACCENT_TEXT))
        .into()
}

/// The bar at the bottom of a window: a hint on the left, the buttons on the right, Close or
/// Cancel always the last (§9.2).
pub fn button_bar<'a, M: 'a>(
    hint: impl IntoFragment<'a>,
    buttons: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let bar = row![text::secondary(hint), space::horizontal()]
        .extend(buttons)
        .spacing(SPACE_S)
        .align_y(Alignment::Center);
    column![
        horizontal_line(),
        container(bar)
            .padding(Padding {
                left: SPACE_L,
                right: SPACE_L,
                ..Padding::ZERO
            })
            .center_y(BUTTON_BAR_HEIGHT - LINE)
            .style(style::window),
    ]
    .into()
}

fn horizontal_line<'a, M: 'a>() -> Element<'a, M> {
    container(space())
        .width(Length::Fill)
        .height(LINE)
        .style(style::divider)
        .into()
}

/// A 1-px line between two regions side by side.
pub fn vertical_line<'a, M: 'a>() -> Element<'a, M> {
    container(space())
        .width(LINE)
        .height(Length::Fill)
        .style(style::divider)
        .into()
}

/// The kinds of notice (§8.12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    Info,
    Success,
    Warning,
    Error,
}

impl NoticeKind {
    fn look(self) -> (Icon, Color) {
        match self {
            NoticeKind::Info => (Icon::Info, ACCENT_TEXT),
            NoticeKind::Success => (Icon::CircleCheck, SUCCESS),
            NoticeKind::Warning => (Icon::TriangleAlert, WARNING),
            NoticeKind::Error => (Icon::CircleAlert, ERROR),
        }
    }
}

/// Buttons side by side, 8 px apart.
pub fn buttons<'a, M: 'a>(items: impl IntoIterator<Item = Element<'a, M>>) -> Element<'a, M> {
    iced::widget::Row::with_children(items)
        .spacing(SPACE_S)
        .into()
}

/// A notice in the flow of a page (§8.12): a colored edge and icon, what happened (bold when a
/// second line follows), what to do, and at most two buttons.
pub fn notice<'a, M: 'a>(
    kind: NoticeKind,
    headline: impl IntoFragment<'a>,
    detail: Option<String>,
    actions: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let (glyph, color) = kind.look();
    let headline = if detail.is_some() {
        text::strong(headline)
    } else {
        text::body(headline)
    };
    let actions: Vec<Element<'a, M>> = actions.into_iter().collect();
    let mut lines = column![headline].extend(detail.map(|d| text::secondary(d).into()));
    if !actions.is_empty() {
        lines = lines.push(container(buttons(actions)).padding(Padding {
            top: SPACE_S,
            ..Padding::ZERO
        }));
    }
    let body = row![
        container(icon(glyph, ICON_M, color)).padding(Padding {
            top: SPACE_XXS,
            ..Padding::ZERO
        }),
        container(lines).width(Length::Fill),
    ]
    .spacing(SPACE_S)
    .padding(Padding {
        top: SPACE_S,
        bottom: SPACE_S,
        left: SPACE_M - NOTICE_BAR,
        right: SPACE_M,
    });
    container(
        row![
            container(space())
                .width(NOTICE_BAR)
                .height(Length::Fill)
                .style(style::fill(color)),
            body,
        ]
        .height(Length::Shrink),
    )
    .width(Length::Fill)
    .style(style::notice(kind == NoticeKind::Error))
    .clip(true)
    .into()
}

/// A status in one line: an icon and a few words ("Saved in …").
pub fn inline_status<'a, M: 'a>(
    kind: NoticeKind,
    content: impl IntoFragment<'a>,
) -> Element<'a, M> {
    let (glyph, color) = kind.look();
    row![icon(glyph, ICON_M, color), text::body(content)]
        .spacing(SPACE_S)
        .align_y(Alignment::Center)
        .into()
}

/// ⓘ: help that is never needed to choose, shown on hover (§8.11).
pub fn info<'a, M: 'a>(help: impl IntoFragment<'a>) -> Element<'a, M> {
    with_tooltip(
        icon(Icon::Info, ICON_S, TEXT_SECONDARY),
        help,
        tooltip::Position::Bottom,
    )
}

/// `content` with a tooltip (§8.13).
pub fn with_tooltip<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    tip: impl IntoFragment<'a>,
    position: tooltip::Position,
) -> Element<'a, M> {
    tooltip(
        content,
        container(text::tooltip(tip))
            .max_width(TOOLTIP_MAX_WIDTH)
            .padding(Padding {
                top: SPACE_XS + SPACE_XXS,
                bottom: SPACE_XS + SPACE_XXS,
                left: SPACE_S,
                right: SPACE_S,
            })
            .style(style::tooltip),
        position,
    )
    .gap(SPACE_XS + SPACE_XXS)
    .delay(TOOLTIP_DELAY)
    .into()
}
