//! Layout pieces (`docs/design/design-system.md` §8–9): pages, rows, the navigation list, the
//! button bar, notices and help.

use iced::widget::text::IntoFragment;
use iced::widget::{button, column, container, row, space, Column};
use iced::{Alignment, Color, Element, Length, Padding};

use super::icons::{icon, Icon};
use super::style::{self, ButtonKind};
use super::text;
use super::tokens::*;
use super::tooltip::{self, Position};

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
        .max_width(PAGE_MAX_WIDTH + 2.0 * SPACE_XL)
        .into()
}

/// A window on the system (§9.2): navigation on the left, the scrolling page on the right, the
/// button bar at the bottom.
pub fn window_with_navigation<'a, M: 'a>(
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

/// One option per row: the label in its column on the left, the controls on the right (§10.1).
pub fn setting_row<'a, M: 'a>(
    label: impl IntoFragment<'a>,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    labelled_row(text::body(label).into(), content.into())
}

/// One option per row in a narrow place (§5.3, §13.9 "Page under 440"): the label above its
/// controls, so a short label does not leave a column of empty space beside them.
pub fn stacked_row<'a, M: 'a>(
    label: impl IntoFragment<'a>,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    stacked(text::strong(label).into(), content.into())
}

/// A [`stacked_row`] whose label carries ⓘ with `help` (§8.11).
pub fn stacked_row_with_info<'a, M: 'a>(
    label: impl IntoFragment<'a>,
    help: impl IntoFragment<'a>,
    content: impl Into<Element<'a, M>>,
) -> Element<'a, M> {
    let label = row![text::strong(label), info(help)]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center);
    stacked(label.into(), content.into())
}

fn stacked<'a, M: 'a>(label: Element<'a, M>, content: Element<'a, M>) -> Element<'a, M> {
    column![label, content].spacing(SPACE_S).into()
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

/// A column of choices (checkboxes, radio options, a line of text over buttons), 12 px apart.
pub fn choices<'a, M: 'a>(items: impl IntoIterator<Item = Element<'a, M>>) -> Column<'a, M> {
    Column::with_children(items).spacing(SPACE_M)
}

/// [`choices`] whose first line of text meets the label beside them.
pub fn aligned<'a, M: 'a>(items: impl IntoIterator<Item = Element<'a, M>>) -> Column<'a, M> {
    choices(items).padding(Padding {
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
/// `dot` puts the update dot at its end.
pub fn nav_item<'a, M: Clone + 'a>(
    glyph: Icon,
    label: impl IntoFragment<'a>,
    selected: bool,
    dot: bool,
    on_press: M,
) -> Element<'a, M> {
    nav_item_with(glyph, label, selected, dot.then(update_dot), Some(on_press))
}

/// [`nav_item`] with `trailing` at its right end (a badge), and without `on_press` disabled.
pub fn nav_item_with<'a, M: Clone + 'a>(
    glyph: Icon,
    label: impl IntoFragment<'a>,
    selected: bool,
    trailing: Option<Element<'a, M>>,
    on_press: Option<M>,
) -> Element<'a, M> {
    let enabled = on_press.is_some();
    let color = match (enabled, selected) {
        (false, _) => TEXT_DISABLED,
        (true, true) => ACCENT_TEXT,
        (true, false) => TEXT_SECONDARY,
    };
    let label = match (enabled, selected) {
        (false, _) => text::body(label).color(TEXT_DISABLED),
        (true, true) => text::strong(label),
        (true, false) => text::secondary(label),
    };
    let mark = container(space())
        .width(SELECTION_BAR)
        .height(ICON_M)
        .style(style::fill(if selected {
            ACCENT_TEXT
        } else {
            Color::TRANSPARENT
        }));
    // The label fills what is left: iced lays `Fill` children out last, so the trailing badge
    // keeps its whole width and the label is the one that wraps.
    let content = row![
        mark,
        icon(glyph, ICON_M, color),
        container(label).width(Length::Fill)
    ]
    .push(trailing)
    .spacing(SPACE_S)
    .align_y(Alignment::Center);
    // 32 px for one line; a label that wraps (a long translation) makes the item taller.
    // Clipped: a long label never draws past the item's background.
    button(content)
        .clip(true)
        .width(Length::Fill)
        .padding(Padding {
            top: ROW_PADDING_Y,
            bottom: ROW_PADDING_Y,
            left: SPACE_XS,
            right: SPACE_M,
        })
        .style(style::button(ButtonKind::Nav(selected)))
        .on_press_maybe(on_press)
        .into()
}

/// The dot that says "an update is ready".
pub fn update_dot<'a, M: 'a>() -> Element<'a, M> {
    container(space())
        .width(DOT_SIZE)
        .height(DOT_SIZE)
        .style(style::dot(ACCENT_TEXT))
        .into()
}

/// The bar at the bottom of a window: a hint on the left, the buttons on the right, Close or
/// Cancel always the last (§9.2).
pub fn button_bar<'a, M: 'a>(
    hint: impl IntoFragment<'a>,
    buttons: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    // The hint takes what the buttons leave (a `Fill` child is laid out last), so a long hint
    // wraps instead of squeezing the buttons.
    let bar = row![container(text::secondary(hint)).width(Length::Fill)]
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

/// A 1-px line between two regions one above the other.
pub fn horizontal_line<'a, M: 'a>() -> Element<'a, M> {
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
    tooltip::tip_text(
        icon(Icon::Info, ICON_S, TEXT_SECONDARY),
        help,
        Position::Bottom,
    )
}
