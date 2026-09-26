//! UI for the batch panel: the action list, the chosen action's options, and the job panel
//! every action shares.

use frename_core::FileId;
use iced::widget::{
    button, column, container, progress_bar, row, scrollable, text, tooltip, Space,
};
use iced::{Element, Length};

use crate::features::folder_workspace::Directory;
use crate::theme;

use super::state::Progress;
use super::{Action, BatchState, Message};

const ACTION_LIST_WIDTH: f32 = 190.0;
const FAILED_LIST_HEIGHT: f32 = 120.0;

/// Render the batch panel. `directory` names the files of the job.
pub fn view<'a>(state: &'a BatchState, directory: Option<&'a Directory>) -> Element<'a, Message> {
    let actions = column(
        Action::ALL
            .iter()
            .map(|action| action_entry(*action, state.action())),
    )
    .spacing(2)
    .width(Length::Fixed(ACTION_LIST_WIDTH));

    let options = container(action_options(state))
        .padding([4, 16])
        .width(Length::Fill)
        .height(Length::Fill);

    // The panel's title: without it the list reads as loose buttons, not as actions on the
    // checked files.
    let heading = row![
        text(fl!("batch-title")).size(18),
        text(fl!("batch-on-checked", count = state.checked_count()))
            .size(13)
            .color(theme::TEXT_MUTED),
    ]
    .spacing(10)
    .align_y(iced::Alignment::End);
    // Back to the open file; locked while a job runs, like the batch button in the controls bar.
    let close = tooltip(
        button(text("✕").size(14))
            .on_press_maybe((!state.is_running()).then_some(Message::SetActive(false)))
            .padding([2, 8])
            .style(theme::icon_button_style(!state.is_running())),
        container(text(fl!("batch-back")))
            .padding([2, 6])
            .style(theme::elevated_container_style),
        tooltip::Position::Left,
    );
    let title =
        row![heading, Space::new().width(Length::Fill), close].align_y(iced::Alignment::Center);

    let mut content = column![
        container(title).padding([4, 10]),
        row![actions, options].height(Length::Fill),
    ]
    .spacing(8);
    if let Some(progress) = state.progress() {
        content = content.push(job_panel(state, progress, directory));
    }

    container(content)
        .padding(8)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(theme::panel_container_style)
        .into()
}

/// One entry of the action list.
fn action_entry(action: Action, selected: Action) -> Element<'static, Message> {
    button(text(action.label()).size(13))
        .on_press(Message::SelectAction(action))
        .width(Length::Fill)
        .padding([6, 10])
        .style(theme::list_item_button_style(action == selected, true))
        .into()
}

/// The selected action's panel, and the button that runs it on the checked files.
fn action_options(state: &BatchState) -> Element<'_, Message> {
    let count = state.checked_count();
    let can_run = count > 0 && state.operation().is_some() && !state.is_running();
    let run = button(text(fl!("batch-run", count = count)).size(13))
        .on_press_maybe(can_run.then_some(Message::Run))
        .padding([6, 14]);

    column![
        state.actions().view(state.action()).map(Message::Action),
        run
    ]
    .spacing(12)
    .into()
}

/// The job panel shared by every action: progress, the file in work, the outcome counts, and
/// Cancel while it runs; a summary, the failed files and Close once it has ended.
fn job_panel<'a>(
    state: &'a BatchState,
    progress: Progress,
    directory: Option<&'a Directory>,
) -> Element<'a, Message> {
    let name = |id: FileId| -> String {
        directory
            .and_then(|d| d.file_by_id(id))
            .and_then(|f| f.file_path().file_name())
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let counts = text(fl!(
        "batch-counts",
        done = progress.done,
        skipped = progress.skipped,
        failed = progress.failed
    ))
    .size(12)
    .color(theme::TEXT_SOFT);

    let mut panel = column![].spacing(6);
    if progress.running {
        let current = state.current().map(name).unwrap_or_default();
        let (label, cancel) = if progress.cancelled {
            (fl!("batch-stopping"), None)
        } else {
            (fl!("batch-cancel"), Some(Message::Cancel))
        };
        panel = panel
            .push(
                row![
                    text(format!("{} / {}", progress.finished, progress.total)).size(13),
                    text(current)
                        .size(12)
                        .color(theme::TEXT_MUTED)
                        .wrapping(iced::widget::text::Wrapping::None),
                ]
                .spacing(10),
            )
            .push(
                progress_bar(0.0..=progress.total.max(1) as f32, progress.finished as f32).girth(6),
            )
            .push(
                row![
                    counts,
                    Space::new().width(Length::Fill),
                    button(text(label).size(13)).on_press_maybe(cancel)
                ]
                .align_y(iced::Alignment::Center),
            );
    } else {
        let summary = if progress.finished < progress.total {
            fl!(
                "batch-stopped",
                finished = progress.finished,
                total = progress.total
            )
        } else {
            fl!("batch-finished", total = progress.total)
        };
        panel = panel.push(text(summary).size(13)).push(
            row![
                counts,
                Space::new().width(Length::Fill),
                button(text(fl!("batch-close")).size(13)).on_press(Message::CloseReport),
            ]
            .align_y(iced::Alignment::Center),
        );
        let failed = state.failed();
        if !failed.is_empty() {
            let names = column(
                failed
                    .into_iter()
                    .map(|id| text(name(id)).size(12).color(theme::ERROR).into()),
            );
            panel = panel
                .push(text(fl!("batch-failed")).size(12).color(theme::TEXT_MUTED))
                .push(
                    container(
                        scrollable(names)
                            .width(Length::Fill)
                            .style(theme::dark_scrollable_style),
                    )
                    .max_height(FAILED_LIST_HEIGHT),
                );
        }
    }
    container(panel)
        .padding(10)
        .width(Length::Fill)
        .style(theme::elevated_container_style)
        .into()
}
