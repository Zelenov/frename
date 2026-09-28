//! The job's page (`docs/design/design-system.md` §13.6.5, §13.6.7): while it runs, the progress
//! and the live figures with Cancel; once it has ended, one notice with the outcome, the
//! figures, what it cost, the files not done, and Run again / Close.

use frename_core::FileId;
use iced::widget::{column, container, progress_bar, row, Column};
use iced::{Alignment, Color, Element, Length, Padding};

use crate::features::folder_workspace::Directory;
use crate::ui::layout::{self, NoticeKind};
use crate::ui::tokens::*;
use crate::ui::{button, scroll, style, text};

use super::actions::{describe_ai::duration_text, spend_line};
use super::page;
use super::state::Progress;
use super::view::{page_body, with_button_bar};
use super::{Action, BatchState, Message};

/// How a job ended, said once, in the result's notice.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Outcome {
    /// Every file finished and none failed.
    Done,
    /// The user cancelled it.
    Cancelled,
    /// An error stopped it (no credit, no connection): the reason.
    Stopped(String),
    /// It finished, but some files were not done.
    Problems,
}

impl Outcome {
    fn of(progress: &Progress, stopped: Option<&str>) -> Self {
        match stopped {
            Some(reason) => Outcome::Stopped(reason.to_string()),
            None if progress.cancelled && progress.finished < progress.total => Outcome::Cancelled,
            None if progress.failed > 0 => Outcome::Problems,
            None => Outcome::Done,
        }
    }

    fn kind(&self) -> NoticeKind {
        match self {
            Outcome::Done => NoticeKind::Success,
            Outcome::Cancelled => NoticeKind::Info,
            Outcome::Stopped(_) => NoticeKind::Error,
            Outcome::Problems => NoticeKind::Warning,
        }
    }

    /// The word after the action's name in the page's heading.
    fn state_word(&self) -> String {
        match self {
            Outcome::Done | Outcome::Problems => fl!("batch-job-finished"),
            Outcome::Cancelled | Outcome::Stopped(_) => fl!("batch-job-stopped"),
        }
    }
}

/// The job's page for `progress`.
pub fn view<'a>(
    state: &'a BatchState,
    progress: Progress,
    directory: Option<&'a Directory>,
) -> Element<'a, Message> {
    let action = state.job_action().unwrap_or(state.action());
    if progress.running {
        running(state, action, progress, directory)
    } else {
        result(state, action, progress, directory)
    }
}

/// The action's name as the page's heading, with the job's state after it.
fn heading<'a>(action: Action, state_word: String) -> Element<'a, Message> {
    row![text::heading(action.label()), text::secondary(state_word)]
        .spacing(SPACE_S)
        .align_y(Alignment::Center)
        .into()
}

/// The name of the file `id` in `directory`.
fn file_name(directory: Option<&Directory>, id: FileId) -> String {
    directory
        .and_then(|d| d.file_by_id(id))
        .and_then(|f| f.file_path().file_name())
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn running<'a>(
    state: &'a BatchState,
    action: Action,
    progress: Progress,
    directory: Option<&'a Directory>,
) -> Element<'a, Message> {
    let (item_fraction, step) = state.item_now();
    let time_left = match progress.time_left() {
        Some(left) => fl!("batch-time-left", time = duration_text(left.as_secs_f64())),
        None => fl!("batch-time-estimating"),
    };
    let counts = row![
        text::strong(fl!(
            "batch-progress-files",
            finished = (progress.finished as i64),
            total = (progress.total as i64)
        )),
        iced::widget::space::horizontal(),
        text::secondary(time_left),
        text::secondary(fl!(
            "batch-time-spent",
            time = duration_text(progress.elapsed.as_secs_f64())
        )),
    ]
    .spacing(SPACE_M)
    .align_y(Alignment::Center);
    let bar = progress_bar(
        0.0..=progress.total.max(1) as f32,
        progress.finished as f32 + item_fraction,
    )
    .girth(PROGRESS_HEIGHT)
    .style(style::progress);
    let current = state.current().map(|id| file_name(directory, id));
    // A long name is cut at the page's edge instead of running past it.
    let current = row![]
        .push(current.map(|name| {
            container(text::mono(name).wrapping(iced::widget::text::Wrapping::None)).clip(true)
        }))
        .push((!step.is_empty()).then(|| text::secondary(step)))
        .spacing(SPACE_S);
    let body = column![
        heading(action, fl!("batch-job-running")),
        column![counts, bar, current].spacing(SPACE_S),
        figures(action, &progress),
    ]
    .spacing(SPACE_XL);
    let cancel = if progress.cancelled {
        button::secondary(fl!("batch-stopping"))
    } else {
        button::secondary(fl!("batch-cancel")).on_press(Message::Cancel)
    };
    with_button_bar(
        page_body(body),
        fl!("batch-locked-until-end"),
        [cancel.into()],
    )
}

fn result<'a>(
    state: &'a BatchState,
    action: Action,
    progress: Progress,
    directory: Option<&'a Directory>,
) -> Element<'a, Message> {
    let outcome = Outcome::of(&progress, state.stopped());
    let mut body = Column::new()
        .push(heading(action, outcome.state_word()))
        .push(notice(state, &outcome, &progress))
        .push(figures(action, &progress))
        .spacing(SPACE_XL);
    if let Some(usage) = progress.usage {
        let spend = spend_line(state.job_ai_model(), usage);
        let spend = if progress.usage_unknown {
            format!("{} {spend}", fl!("batch-ai-at-least"))
        } else {
            spend
        };
        body = body.push(page::plan([(fl!("batch-plan-cost"), spend)]));
    }
    if let Some(report) = state.report() {
        body = body.push(text::secondary(report));
    }
    body = body.push(files_not_done(state, action, &outcome, directory));
    body = body.push(files_written(state, action, directory));

    let retry = state.retryable();
    let again = (!retry.is_empty()).then(|| {
        button::secondary(fl!("batch-run-again", count = (retry.len() as i64)))
            .on_press(Message::Retry)
            .into()
    });
    let close = button::primary(fl!("batch-close"))
        .on_press(Message::CloseReport)
        .into();
    with_button_bar(page_body(body), "", again.into_iter().chain([close]))
}

/// The one notice that says how the job ended, with the fix when there is one.
fn notice<'a>(state: &BatchState, outcome: &Outcome, progress: &Progress) -> Element<'a, Message> {
    let total = progress.total as i64;
    let (headline, detail) = match outcome {
        Outcome::Done => (
            fl!("batch-result-done", total = total),
            Some(fl!(
                "batch-result-done-detail",
                changed = (progress.done as i64),
                unchanged = (progress.skipped as i64)
            )),
        ),
        Outcome::Cancelled => (
            fl!(
                "batch-stopped",
                finished = (progress.finished as i64),
                total = total
            ),
            None,
        ),
        Outcome::Stopped(reason) => (reason.clone(), None),
        Outcome::Problems => (
            fl!(
                "batch-result-problems",
                failed = (progress.failed as i64),
                total = total
            ),
            None,
        ),
    };
    let fix = (state.out_of_credit()).then(|| {
        button::secondary(fl!("batch-add-credit"))
            .on_press(Message::OpenBilling)
            .into()
    });
    layout::notice(outcome.kind(), headline, detail, fix)
}

/// Changed, unchanged, not done, and not reached when the job stopped early: a number over its
/// word each, "not done" in red only when there are some.
fn figures<'a>(action: Action, progress: &Progress) -> Element<'a, Message> {
    let not_done_color = if progress.failed > 0 { ERROR } else { TEXT };
    let mut figures = row![
        figure(progress.done, action.done_label(), TEXT),
        figure(progress.skipped, fl!("batch-figure-unchanged"), TEXT),
        figure(
            progress.failed,
            fl!("batch-figure-not-done"),
            not_done_color
        ),
    ]
    .spacing(SPACE_XXL);
    if !progress.running && progress.not_reached() > 0 {
        figures = figures.push(figure(
            progress.not_reached(),
            fl!("batch-figure-not-reached"),
            TEXT_SECONDARY,
        ));
    }
    figures.into()
}

fn figure<'a>(count: usize, word: String, color: Color) -> Element<'a, Message> {
    column![
        text::heading(count.to_string()).color(color),
        text::secondary(word)
    ]
    .into()
}

/// The files the job did not do, with why, as a table; for some actions also the files it left
/// alone with a reason. When every failure has the reason the notice already gives, only the
/// names are listed.
fn files_not_done<'a>(
    state: &'a BatchState,
    action: Action,
    outcome: &Outcome,
    directory: Option<&'a Directory>,
) -> Option<Element<'a, Message>> {
    let stop_reason = match outcome {
        Outcome::Stopped(reason) => Some(reason.as_str()),
        _ => None,
    };
    let failed = state.failed().into_iter().map(|(id, reason)| {
        let reason = reason.filter(|r| Some(*r) != stop_reason);
        (id, reason.map(str::to_string))
    });
    // Generating subtitles also says why each video it left alone got none, and moving in/out
    // points out of names which files kept the points they had stored.
    let listed = match action {
        Action::GenerateSubtitles => state.skipped_with_reason(),
        _ => Vec::new(),
    };
    let rows: Vec<(FileId, Option<String>)> = failed
        .chain(
            listed
                .into_iter()
                .map(|(id, reason)| (id, Some(reason.to_string()))),
        )
        .collect();
    if rows.is_empty() {
        return None;
    }
    let with_reasons = rows.iter().any(|(_, reason)| reason.is_some());
    let caption = match action {
        Action::GenerateSubtitles => fl!("batch-failed-subtitles"),
        _ => fl!("batch-files-not-done"),
    };
    let title = row![
        text::title(caption),
        iced::widget::space::horizontal(),
        button::link(fl!("batch-open-log")).on_press(Message::OpenLog),
    ]
    .align_y(Alignment::Center);
    let header = with_reasons.then(|| {
        table_row(
            text::caption(fl!("batch-table-file")).into(),
            Some(text::caption(fl!("batch-table-why")).into()),
        )
    });
    let lines = Column::with_children(rows.into_iter().map(|(id, reason)| {
        table_row(
            // A file name has no spaces: it breaks inside the word rather than run past its cell.
            text::mono(file_name(directory, id))
                .color(TEXT)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                .into(),
            with_reasons.then(|| text::body(reason.unwrap_or_default()).into()),
        )
    }));
    let shown = FAILED_ROWS_SHOWN * TABLE_ROW_HEIGHT;
    let table = column![]
        .push(header)
        .push(container(scroll::vertical(lines)).max_height(shown));
    Some(column![title, table].spacing(SPACE_S).into())
}

/// The videos that got subtitle files and which ones, when the action says (a Premiere
/// transcript was written), as a table.
fn files_written<'a>(
    state: &'a BatchState,
    action: Action,
    directory: Option<&'a Directory>,
) -> Option<Element<'a, Message>> {
    if action != Action::GenerateSubtitles {
        return None;
    }
    let written = state.done_with_reason();
    if written.is_empty() {
        return None;
    }
    let header = table_row(
        text::caption(fl!("batch-table-file")).into(),
        Some(text::caption(fl!("batch-table-written")).into()),
    );
    let lines = Column::with_children(written.into_iter().map(|(id, files)| {
        table_row(
            text::mono(file_name(directory, id))
                .color(TEXT)
                .wrapping(iced::widget::text::Wrapping::WordOrGlyph)
                .into(),
            Some(text::mono(files.to_string()).into()),
        )
    }));
    let shown = FAILED_ROWS_SHOWN * TABLE_ROW_HEIGHT;
    let table = column![]
        .push(header)
        .push(container(scroll::vertical(lines)).max_height(shown));
    Some(
        column![text::title(fl!("batch-written-subtitles")), table]
            .spacing(SPACE_S)
            .into(),
    )
}

/// A row of the table: the file name, then why, with a line under it.
fn table_row<'a>(
    name: Element<'a, Message>,
    why: Option<Element<'a, Message>>,
) -> Element<'a, Message> {
    let cells = row![container(name).width(Length::FillPortion(2)).clip(true)]
        .push(why.map(|why| container(why).width(Length::FillPortion(3)).clip(true)))
        .spacing(SPACE_S)
        .padding(Padding {
            top: SPACE_TIGHT,
            bottom: SPACE_TIGHT,
            left: SPACE_S,
            right: SPACE_S,
        });
    column![cells, layout::horizontal_line()].into()
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn progress(finished: usize, failed: usize, cancelled: bool) -> Progress {
        Progress {
            total: 12,
            finished,
            done: finished - failed,
            skipped: 0,
            failed,
            running: false,
            cancelled,
            usage: None,
            usage_unknown: false,
            elapsed: Duration::from_secs(60),
        }
    }

    #[test]
    fn the_outcome_is_said_once_by_what_ended_the_job() {
        assert_eq!(Outcome::of(&progress(12, 0, false), None), Outcome::Done);
        assert_eq!(
            Outcome::of(&progress(12, 3, false), None),
            Outcome::Problems
        );
        assert_eq!(Outcome::of(&progress(7, 0, true), None), Outcome::Cancelled);
        assert_eq!(
            Outcome::of(&progress(1, 1, false), Some("Stopped: no credit.")),
            Outcome::Stopped("Stopped: no credit.".to_string())
        );
    }

    #[test]
    fn a_cancel_after_the_last_file_is_still_done() {
        assert_eq!(Outcome::of(&progress(12, 0, true), None), Outcome::Done);
    }

    #[test]
    fn each_outcome_has_its_own_notice() {
        let kinds = [
            Outcome::Done.kind(),
            Outcome::Cancelled.kind(),
            Outcome::Stopped(String::new()).kind(),
            Outcome::Problems.kind(),
        ];
        for (i, kind) in kinds.iter().enumerate() {
            assert!(kinds[i + 1..].iter().all(|other| other != kind));
        }
    }
}
