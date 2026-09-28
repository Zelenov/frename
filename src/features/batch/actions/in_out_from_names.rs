//! "Move in/out points out of file names": older versions could keep in/out points in the file
//! name (`clip.in_00_01_05.out_00_02_10.mp4`). This takes them out of each file's name and
//! saves them where Settings keep in/out points now: in the comment or inside the video. A file
//! that already has in/out points stored keeps those, and its line in the report says so. No
//! options of its own: where the points go is a setting, which its panel opens.

use std::path::Path;

use frename_core::{
    format_in_out_range, FileTagger, InOutStorage, MoveOutcome, NameInOutMove, NameInOutProblem,
};
use iced::Element;

use super::super::page::{self, Change};
use super::super::ItemResult;
use super::ActionMessage;

pub fn label() -> String {
    fl!("batch-action-in-out-from-names")
}

pub fn view<'a>() -> Element<'a, ActionMessage> {
    let (goes_to, change) = match frename_core::metadata_storage().in_out {
        InOutStorage::Comment => (
            fl!("batch-action-in-out-from-names-status-comment"),
            Change::IntoComments,
        ),
        InOutStorage::InVideo => (
            fl!("batch-action-in-out-from-names-status-video"),
            Change::IntoVideos,
        ),
    };
    page::page(
        label(),
        fl!("batch-action-in-out-from-names-hint"),
        &[Change::Renames, change],
        [page::linked_row(
            fl!("batch-option-goes-to"),
            goes_to,
            fl!("batch-ai-change"),
            ActionMessage::OpenSettings,
        )],
    )
}

/// Move the in/out points in the name of the file at `path` to where they are kept now.
pub fn run(path: &Path) -> ItemResult {
    // The log is always English, unlike the UI text `label()` returns.
    let log_label = super::Action::InOutFromNames.log_id();
    match FileTagger::move_in_out_out_of_name(path) {
        NameInOutMove::NothingToMove => super::item_result(MoveOutcome::NothingToMove),
        NameInOutMove::Moved { path, kept_stored } => {
            let mut item = super::item_result(MoveOutcome::Moved(path.clone()));
            if let Some(kept) = kept_stored {
                let (stored, from_name) = (
                    format_in_out_range(kept.stored),
                    format_in_out_range(kept.from_name),
                );
                log::info!(
                    "{log_label}: {path:?} keeps its stored {stored} and drops the name's {from_name}"
                );
                item.reason = Some(fl!(
                    "batch-action-in-out-from-names-kept",
                    stored = stored,
                    name = from_name
                ));
            }
            item
        }
        NameInOutMove::Failed { path, problem } => {
            let reason = match problem {
                NameInOutProblem::NameWouldBeEmpty => {
                    log::warn!("{log_label}: {path:?} left alone: the name would be empty");
                    fl!("batch-action-in-out-from-names-empty")
                }
                NameInOutProblem::NameTaken(name) => {
                    log::warn!("{log_label}: {path:?} left alone: {name:?} already exists");
                    fl!("batch-action-in-out-from-names-taken", name = name)
                }
                NameInOutProblem::NotRenamed => {
                    log::warn!("{log_label}: {path:?} could not be renamed");
                    fl!("batch-action-in-out-from-names-not-renamed")
                }
            };
            let mut item = super::item_result(MoveOutcome::Failed(path));
            item.reason = Some(reason);
            item
        }
    }
}
