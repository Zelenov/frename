//! "Move in/out points out of file names": older versions could keep in/out points in the file
//! name (`clip.in_00_01_05.out_00_02_10.mp4`). This takes them out of each file's name and
//! saves them where Settings keep in/out points now: in the comment or inside the video. A file
//! that already has in/out points stored keeps those, and its line in the report says so. No
//! options of its own: where the points go is a setting, which its panel opens.

use std::path::Path;

use frename_core::{format_in_out_range, FileTagger, InOutStorage, MoveOutcome, NameInOutProblem};
use iced::widget::{button, column, text};
use iced::Element;

use super::super::ItemResult;
use super::ActionMessage;
use crate::theme;

pub fn label() -> String {
    fl!("batch-action-in-out-from-names")
}

pub fn view<'a>() -> Element<'a, ActionMessage> {
    let status = match frename_core::metadata_storage().in_out {
        InOutStorage::Comment => fl!("batch-action-in-out-from-names-status-comment"),
        InOutStorage::InVideo => fl!("batch-action-in-out-from-names-status-video"),
    };
    // One above the other: side by side, the panel is too narrow for the status.
    let settings = column![
        text(status).size(13),
        button(text(fl!("batch-action-in-out-from-names-settings")).size(12))
            .on_press(ActionMessage::OpenSettings)
            .padding([3, 10])
            .style(theme::icon_button_style(true)),
    ]
    .spacing(8);
    super::panel(
        label(),
        fl!("batch-action-in-out-from-names-hint"),
        settings.into(),
    )
}

/// Move the in/out points in the name of the file at `path` to where they are kept now.
pub fn run(path: &Path) -> ItemResult {
    // The log is always English, unlike the UI text `label()` returns.
    let log_label = super::Action::InOutFromNames.log_id();
    let result = FileTagger::move_in_out_out_of_name(path);
    let failed = matches!(result.outcome, MoveOutcome::Failed(_));
    let mut item = super::item_result(result.outcome);
    match result.problem {
        Some(NameInOutProblem::NameWouldBeEmpty) => {
            log::warn!("{log_label}: {path:?} left alone: the name would be empty");
            item.reason = Some(fl!("batch-action-in-out-from-names-empty"));
        }
        Some(NameInOutProblem::NameTaken(name)) => {
            log::warn!("{log_label}: {path:?} left alone: {name:?} already exists");
            item.reason = Some(fl!("batch-action-in-out-from-names-taken", name = name));
        }
        None if failed => {
            log::warn!("{log_label}: {path:?} could not be renamed");
            item.reason = Some(fl!("batch-action-in-out-from-names-not-renamed"));
        }
        None => {}
    }
    if let Some(kept) = result.kept_stored {
        let (stored, from_name) = (
            format_in_out_range(kept.stored),
            format_in_out_range(kept.from_name),
        );
        log::info!(
            "{log_label}: {path:?} keeps its stored {stored} and drops the name's {from_name}"
        );
        let kept = fl!(
            "batch-action-in-out-from-names-kept",
            stored = stored,
            name = from_name
        );
        item.reason = Some(match item.reason {
            Some(reason) => format!("{reason}; {kept}"),
            None => kept,
        });
    }
    item
}
