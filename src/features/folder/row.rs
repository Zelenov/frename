//! One row of the file list (`docs/design/design-system.md` §13.4.2, §13.6.6): the check column in
//! batch mode (or the file's state in the last job), the status column, the name as mini chips
//! and mono text, the marker count, and the first line of the comment under it.

use frename_core::FileId;
use iced::widget::{column, container, mouse_area, row, space, text::Wrapping, Id};
use iced::{mouse, Alignment, Element, Length, Padding};

use crate::features::batch::{BatchState, ItemStatus};

use crate::ui::icons::{icon, spinner, Icon};
use crate::ui::tokens::*;
use crate::ui::tooltip::{self, Position};
use crate::ui::{form, list, style, text};
use crate::widgets;

use super::view::ListProps;
use super::{InlineRename, Message, FOLDER_RENAME_INPUT_ID, FOLDER_ROW_HEIGHT};

/// The row's inset from its top and bottom (its sides are the list row's own).
const ROW_PADDING: Padding = Padding {
    top: SPACE_XS,
    bottom: SPACE_XS,
    left: 0.0,
    right: 0.0,
};

/// How many characters of a matching comment line show before the first hit.
const FRAGMENT_LEAD_CHARS: usize = 12;

/// What a row needs to know about its file, besides the file itself.
pub struct RowState<'a> {
    pub index: usize,
    pub selected: bool,
    /// A job runs: the row neither opens nor checks its file.
    pub locked: bool,
    /// Why the file failed in the last job, when it did and the action said.
    pub failure: Option<&'a str>,
}

/// The row of `file`.
pub fn view<'a>(
    props: &ListProps<'a, '_>,
    file: &'a frename_core::File,
    state: RowState<'a>,
) -> Element<'a, Message> {
    let snapshot = file.snapshot();
    let batch = props.batch;
    let rename = props
        .rename
        .filter(|r| r.id == file.id() && batch.is_none());

    // The columns before the name: the check column in batch mode, then the status column.
    let mut lead = row![];
    if let Some(batch) = batch {
        lead = lead.push(check_cell(
            batch,
            file.id(),
            state.locked,
            state.failure,
            props.spinner_frame,
        ));
    }
    let lead = lead.push(status_cell(
        file.has_subtitles(),
        state.selected && state.locked,
    ));
    let lead_width = STATUS_COLUMN + if batch.is_some() { CHECK_COLUMN } else { 0.0 };

    let body: Element<'a, Message> = match rename {
        Some(rename) => row![lead, container(rename_editor(rename)).width(Length::Fill)]
            .align_y(Alignment::Center)
            .into(),
        None => {
            let marker_count = snapshot.marker_count();
            let not_saved = props.markers_not_saved.contains_key(&file.id());
            let name = widgets::file_name_display::view(
                snapshot,
                props.tag_color_mapping,
                props.tag_palette,
                name_width(props.width, lead_width, marker_count, not_saved),
            );
            let name_line = row![
                lead,
                // A long name is cut at the row's end: the marker count and the warning stay.
                container(name).width(Length::Fill).clip(true),
            ]
            .push((marker_count > 0).then(|| marker_badge(marker_count)))
            .push(not_saved.then(not_saved_mark))
            .spacing(SPACE_XS)
            .align_y(Alignment::Center);
            // Indented like the name, so the comment starts under it.
            let comment_width = props.width - lead_width - 2.0 * SPACE_S - SCROLL_GUTTER;
            // While searching, a file found by its comment shows the hit instead of the first line.
            let fragment = props
                .directory
                .and_then(|dir| dir.comment_fragment(file, FRAGMENT_LEAD_CHARS));
            let comment_line = row![
                space().width(lead_width),
                comment_line(snapshot, fragment, props.spinner_frame, comment_width)
            ];
            column![name_line, comment_line]
                .spacing(SPACE_XXS)
                .width(Length::Fill)
                .into()
        }
    };
    let body = container(body)
        .padding(ROW_PADDING)
        .width(Length::Fill)
        .height(Length::Fill)
        .center_y(Length::Fill);

    // A job's rows keep their look but take no clicks, and nothing hovers; the open file closed
    // by the job keeps its fill without the bar (its status column shows the lock).
    let item: Element<'a, Message> = if state.locked {
        container(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(style::selectable(state.selected))
            .into()
    } else {
        let item = list::row_item(body, state.selected, HOVER, Length::Fill);
        match (rename.is_some(), batch.is_some()) {
            // The editor takes the clicks while renaming.
            (true, _) => item,
            // Batch mode previews files but does not edit them, renaming included.
            (false, true) => mouse_area(item)
                .on_press(Message::SelectFile(state.index))
                .on_right_press(Message::OpenFileMenu(state.index))
                .interaction(mouse::Interaction::Pointer)
                .into(),
            (false, false) => mouse_area(item)
                .on_press(Message::SelectFile(state.index))
                .on_double_click(Message::StartRename(state.index))
                .on_right_press(Message::OpenFileMenu(state.index))
                .interaction(mouse::Interaction::Pointer)
                .into(),
        }
    };
    let item = container(item)
        .width(Length::Fill)
        .height(FOLDER_ROW_HEIGHT);
    // The full name, since a long one is cut. Beside the list, not over the next row on the way to
    // it (§8.13); so are the row's other tooltips.
    tooltip::tip_text(item, snapshot.file_name(), Position::Right)
}

/// The status column: `captions` when a subtitle file is next to the video; the lock when the job
/// closed this, the open file.
fn status_cell<'a>(has_subtitles: bool, closed_by_job: bool) -> Element<'a, Message> {
    let content: Option<Element<'a, Message>> = if closed_by_job {
        Some(tooltip::tip_text(
            icon(Icon::Lock, ICON_S, TEXT_SECONDARY),
            fl!("folder-closed-by-job"),
            Position::Right,
        ))
    } else if has_subtitles {
        Some(tooltip::tip_text(
            icon(Icon::Captions, ICON_MARK, TEXT_SECONDARY),
            fl!("folder-has-subtitles"),
            Position::Right,
        ))
    } else {
        None
    };
    container(content.unwrap_or_else(|| space().into()))
        .center_x(STATUS_COLUMN)
        .into()
}

/// Where the file stands in the last job, as the check column shows it (§13.6.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Outcome {
    /// Not part of a job, or waiting its turn: the checkbox.
    Check,
    Working,
    Changed,
    Unchanged,
    Failed,
    NotReached,
}

impl Outcome {
    fn of(status: Option<ItemStatus>, running: bool) -> Self {
        match status {
            None => Self::Check,
            Some(ItemStatus::Pending) if running => Self::Check,
            Some(ItemStatus::Pending) => Self::NotReached,
            Some(ItemStatus::Running) => Self::Working,
            Some(ItemStatus::Done) => Self::Changed,
            Some(ItemStatus::Skipped) => Self::Unchanged,
            Some(ItemStatus::Failed) => Self::Failed,
        }
    }

    /// Its icon and color, and what its tooltip says.
    fn look(self, failure: Option<&str>) -> Option<(Icon, iced::Color, String)> {
        match self {
            Self::Check | Self::Working => None,
            Self::Changed => Some((Icon::CircleCheck, SUCCESS, fl!("folder-outcome-changed"))),
            Self::Unchanged => Some((
                Icon::CircleMinus,
                TEXT_SECONDARY,
                fl!("folder-outcome-unchanged"),
            )),
            Self::Failed => Some((
                Icon::CircleAlert,
                ERROR,
                failure.map_or_else(|| fl!("folder-outcome-failed"), str::to_string),
            )),
            Self::NotReached => Some((
                Icon::CircleDashed,
                TEXT_SECONDARY,
                fl!("folder-outcome-not-reached"),
            )),
        }
    }
}

/// The check column: the checkbox, or the file's state in the last job. A click on a state
/// turns it back into the checkbox, as a click on the checkbox would.
fn check_cell<'a>(
    batch: &BatchState,
    id: FileId,
    locked: bool,
    failure: Option<&str>,
    spinner_frame: usize,
) -> Element<'a, Message> {
    let outcome = Outcome::of(batch.status(id), batch.is_running());
    let content: Element<'a, Message> = match outcome.look(failure) {
        _ if outcome == Outcome::Working => tooltip::tip_text(
            spinner(spinner_frame, ICON_M, ACCENT_TEXT),
            fl!("folder-outcome-working"),
            Position::Right,
        ),
        Some((glyph, color, tip)) => {
            let mark = mouse_area(icon(glyph, ICON_M, color));
            let mark = if locked {
                mark
            } else {
                mark.on_press(Message::ToggleChecked(id))
                    .interaction(mouse::Interaction::Pointer)
            };
            tooltip::tip_text(mark, tip, Position::Right)
        }
        None => {
            let check = form::checkbox("", batch.is_checked(id));
            let check = if locked {
                check
            } else {
                check.on_toggle(move |_| Message::ToggleChecked(id))
            };
            check.into()
        }
    };
    container(content).center_x(CHECK_COLUMN).into()
}

/// The marker count at the end of the name line.
/// The room the name line leaves for the name in a list `list_width` px wide: the columns before
/// it, the row's inset, the scrollbar's gutter and the marks after it go first.
fn name_width(list_width: f32, lead_width: f32, marker_count: usize, not_saved: bool) -> f32 {
    let marker = if marker_count > 0 {
        SPACE_XS + ICON_S + SPACE_XXS + CAPTION_CHAR_WIDTH * marker_count.to_string().len() as f32
    } else {
        0.0
    };
    let warning = if not_saved { SPACE_XS + ICON_MARK } else { 0.0 };
    list_width - lead_width - SPACE_XS - 2.0 * SPACE_S - SCROLL_GUTTER - marker - warning
}

fn marker_badge<'a>(count: usize) -> Element<'a, Message> {
    row![
        icon(Icon::MapPin, ICON_S, TEXT_SECONDARY),
        text::caption(count.to_string()).wrapping(Wrapping::None),
    ]
    .spacing(SPACE_XXS)
    .align_y(Alignment::Center)
    .into()
}

/// The file's markers could not be written.
fn not_saved_mark<'a>() -> Element<'a, Message> {
    tooltip::tip_text(
        icon(Icon::CircleAlert, ICON_MARK, ERROR),
        fl!("folder-markers-not-saved"),
        Position::Right,
    )
}

/// The line under a file's name: the first line of its comment, a spinner while the comment is
/// still loading, or nothing. Always one line, so every row keeps its height.
fn comment_line<'a>(
    snapshot: &'a frename_core::FileSnapshot,
    fragment: Option<frename_core::CommentFragment>,
    spinner_frame: usize,
    width: f32,
) -> Element<'a, Message> {
    if snapshot.comment_loading() {
        return spinner(spinner_frame, ICON_S, TEXT_SECONDARY).into();
    }
    if let Some(fragment) = fragment {
        return text::caption_marked(
            &fragment.text,
            &fragment.highlights,
            width,
            CAPTION_CHAR_WIDTH,
        );
    }
    let first = snapshot.comment().lines().next().unwrap_or_default();
    text::caption(text::fit(first, width, CAPTION_CHAR_WIDTH).into_owned())
        .wrapping(Wrapping::None)
        .into()
}

/// The in-place rename editor that replaces a row's name: the whole file name in a field, and
/// when Enter was refused, a red edge and the reason on the line under it.
fn rename_editor(rename: &InlineRename) -> Element<'_, Message> {
    let field = match rename.error {
        Some(_) => form::invalid_text_field("", &rename.text),
        None => form::text_field("", &rename.text),
    }
    .id(Id::from(FOLDER_RENAME_INPUT_ID))
    .on_input(Message::RenameInput)
    .on_submit(Message::SubmitRename);
    let error = rename.error.map(|error| {
        row![
            icon(Icon::CircleAlert, ICON_S, ERROR),
            text::error(error.text()).wrapping(Wrapping::None),
        ]
        .spacing(SPACE_XS)
        .align_y(Alignment::Center)
    });
    column![field].push(error).spacing(SPACE_XXS).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_waiting_file_keeps_its_checkbox_until_the_job_stops() {
        assert_eq!(Outcome::of(None, false), Outcome::Check);
        assert_eq!(Outcome::of(Some(ItemStatus::Pending), true), Outcome::Check);
        assert_eq!(
            Outcome::of(Some(ItemStatus::Pending), false),
            Outcome::NotReached
        );
    }

    #[test]
    fn each_outcome_has_its_own_icon() {
        let icons: Vec<Icon> = [
            Outcome::Changed,
            Outcome::Unchanged,
            Outcome::Failed,
            Outcome::NotReached,
        ]
        .into_iter()
        .filter_map(|o| o.look(None).map(|(glyph, _, _)| glyph))
        .collect();
        assert_eq!(icons.len(), 4);
        assert!(icons
            .iter()
            .enumerate()
            .all(|(i, a)| !icons[i + 1..].contains(a)));
    }

    #[test]
    fn a_failed_file_says_why_when_the_action_told() {
        let (_, _, tip) = Outcome::Failed.look(Some("No credit left")).unwrap();
        assert_eq!(tip, "No credit left");
        assert_eq!(
            Outcome::of(Some(ItemStatus::Running), true),
            Outcome::Working
        );
        assert!(
            Outcome::Working.look(None).is_none(),
            "drawn as the spinner"
        );
    }
}
