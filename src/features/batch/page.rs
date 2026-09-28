//! The parts every action page has, in this order (`docs/design/design-system.md` §13.6.4): the
//! title and what the action does, what it changes, its options as setting rows, its plan, and
//! a notice when something stops it. Actions build their pages only from these, so every page
//! looks the same.

use iced::widget::text::IntoFragment;
use iced::widget::{column, row, Column};
use iced::{Alignment, Element};

use crate::ui::icons::{icon, Icon};
use crate::ui::tokens::*;
use crate::ui::{button, layout, text};

/// What an action changes, said before it runs so the risk is seen (§13.6.4 item 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Renames files.
    Renames,
    /// Writes into the videos (XMP, the rotation flag).
    IntoVideos,
    /// Writes text files next to the videos.
    TextFiles,
    /// Writes into the comments, wherever they are kept.
    IntoComments,
    /// Writes subtitle files next to the videos.
    Subtitles,
    /// Changes only frename's own records.
    OwnRecords,
}

impl Change {
    fn look(self) -> (Icon, String) {
        match self {
            Change::Renames => (Icon::PencilLine, fl!("batch-change-renames")),
            Change::IntoVideos => (Icon::FileVideoCamera, fl!("batch-change-videos")),
            Change::TextFiles => (Icon::FileText, fl!("batch-change-text-files")),
            Change::IntoComments => (Icon::MessageSquareText, fl!("batch-change-comments")),
            Change::Subtitles => (Icon::Captions, fl!("batch-change-subtitles")),
            Change::OwnRecords => (Icon::Database, fl!("batch-change-records")),
        }
    }
}

/// A page: its title and description, what it changes, then `parts` (option rows, the plan,
/// notices), 16 px apart.
pub fn page<'a, M: 'a>(
    title: impl IntoFragment<'a>,
    description: impl IntoFragment<'a>,
    changes: &[Change],
    parts: impl IntoIterator<Item = Element<'a, M>>,
) -> Element<'a, M> {
    let heading = column![text::title(title), text::secondary(description)].spacing(SPACE_XXS);
    let changes = row(changes.iter().map(|change| {
        let (glyph, words) = change.look();
        row![icon(glyph, ICON_M, TEXT_SECONDARY), text::secondary(words)]
            .spacing(SPACE_TIGHT)
            .align_y(Alignment::Center)
            .into()
    }))
    .spacing(SPACE_M)
    .wrap();
    column![heading, changes]
        .extend(parts)
        .spacing(SPACE_L)
        .into()
}

/// A value set somewhere else (in Settings), with the link that changes it right after it:
/// "Claude Haiku 4.5 *Change*" (§8.21).
pub fn value_with_link<'a, M: Clone + 'a>(
    value: impl IntoFragment<'a>,
    link: impl IntoFragment<'a>,
    on_press: M,
) -> Element<'a, M> {
    row![text::body(value), button::link(link).on_press(on_press)]
        .spacing(SPACE_S)
        .align_y(Alignment::Center)
        .wrap()
        .into()
}

/// A setting row whose value is set somewhere else: `label`, then [`value_with_link`].
pub fn linked_row<'a, M: Clone + 'a>(
    label: impl IntoFragment<'a>,
    value: impl IntoFragment<'a>,
    link: impl IntoFragment<'a>,
    on_press: M,
) -> Element<'a, M> {
    layout::setting_row(
        label,
        layout::aligned([value_with_link(value, link, on_press)]),
    )
}

/// The plan of a paid action (§13.6.4 item 4): a key-value row per fact, the numbers strong.
pub fn plan<'a, M: 'a>(rows: impl IntoIterator<Item = (String, String)>) -> Element<'a, M> {
    Column::with_children(rows.into_iter().map(|(key, value)| {
        row![
            iced::widget::container(text::secondary(key)).width(LABEL_WIDTH),
            text::strong(value),
        ]
        .spacing(SPACE_L)
        .into()
    }))
    .spacing(SPACE_XXS)
    .into()
}

/// Lines of secondary text: what is skipped and why, what is sent where.
pub fn notes<'a, M: 'a>(lines: impl IntoIterator<Item = String>) -> Element<'a, M> {
    Column::with_children(lines.into_iter().map(|line| text::secondary(line).into()))
        .spacing(SPACE_XXS)
        .into()
}
