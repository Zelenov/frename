//! Tooltips (`docs/design/design-system.md` §8.13): the command's name, then the keys that do it
//! as key caps. Every icon-only control has one; none holds an error.

use iced::widget::text::IntoFragment;
use iced::widget::{column, container, row, space, tooltip};
use std::time::Duration;

use iced::{Alignment, Element, Padding};

pub use iced::widget::tooltip::Position;

use super::badge::key_cap;
use super::style;
use super::text;
use super::tokens::*;
use super::z::{layered, Z};

const PADDING: Padding = Padding {
    top: SPACE_TIGHT,
    bottom: SPACE_TIGHT,
    left: SPACE_S,
    right: SPACE_S,
};

/// What a tooltip says: a name, the keys that do it, and an optional second line.
#[derive(Debug, Clone, Default)]
pub struct Tip {
    pub label: String,
    pub keys: Vec<&'static str>,
    pub detail: Option<String>,
}

impl Tip {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Self::default()
        }
    }

    /// The keys that do it, one cap each (`["Shift", "F2"]`).
    pub fn keys(mut self, keys: &[&'static str]) -> Self {
        self.keys = keys.to_vec();
        self
    }

    /// A second line under the name.
    pub fn detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    fn view<'a, M: 'a>(self) -> Element<'a, M> {
        let first = row![text::tooltip(self.label)]
            .extend(self.keys.into_iter().map(|key| key_cap(key, false)))
            .spacing(SPACE_XS)
            .align_y(Alignment::Center);
        column![first]
            .extend(
                self.detail
                    .map(|d| text::tooltip(d).color(TEXT_SECONDARY).into()),
            )
            .spacing(SPACE_XXS)
            .into()
    }
}

impl From<String> for Tip {
    fn from(label: String) -> Self {
        Tip::new(label)
    }
}

impl From<&str> for Tip {
    fn from(label: &str) -> Self {
        Tip::new(label)
    }
}

/// `content` with `tip` shown after a hover of `TOOLTIP_DELAY`, on the `position` side away from
/// the edge the control sits on.
pub fn tip<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    tip: impl Into<Tip>,
    position: Position,
) -> Element<'a, M> {
    layered(
        tooltip(
            content,
            container(tip.into().view())
                .max_width(TOOLTIP_MAX_WIDTH)
                .padding(PADDING)
                .style(style::popup),
            position,
        )
        .gap(SPACE_TIGHT)
        .delay(TOOLTIP_DELAY),
        Z::Tooltip,
    )
}

/// `content` with a one-line text tooltip.
pub fn tip_text<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    label: impl IntoFragment<'a>,
    position: Position,
) -> Element<'a, M> {
    layered(
        tooltip(
            content,
            container(text::tooltip(label))
                .max_width(TOOLTIP_MAX_WIDTH)
                .padding(PADDING)
                .style(style::popup),
            position,
        )
        .gap(SPACE_TIGHT)
        .delay(TOOLTIP_DELAY),
        Z::Tooltip,
    )
}

/// `content` with `preview` beside it at once, over every tooltip: what follows a drag (a chip
/// dropped on the trash, §13.5.6). In the tooltip's box; nothing shows while `preview` is `None`,
/// and the widget tree stays the same either way so `content` keeps its state.
pub fn drag_preview<'a, M: 'a>(
    content: impl Into<Element<'a, M>>,
    preview: Option<Element<'a, M>>,
    position: Position,
) -> Element<'a, M> {
    let body: Element<'a, M> = match preview {
        Some(preview) => container(preview)
            .padding(PADDING)
            .style(style::popup)
            .into(),
        None => space().into(),
    };
    layered(
        tooltip(content, body, position)
            .gap(SPACE_TIGHT)
            .delay(Duration::ZERO)
            .snap_within_viewport(true),
        Z::DragPreview,
    )
}
