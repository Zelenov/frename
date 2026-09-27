//! "Move in/out points out of file names": older versions could keep in/out points in the file
//! name (`clip.in_00_01_05.out_00_02_10.mp4`). This takes them out of each file's name and
//! saves them where Settings keep in/out points now: in the comment or inside the video. A file
//! that already has in/out points stored keeps those, and its line in the report says so. No
//! options of its own: where the points go is a setting, which its panel opens.

use std::path::Path;

use frename_core::{FileTagger, InOutStorage, MoveOutcome, NameInOut};
use iced::widget::{button, row, text};
use iced::{Element, Length};

use super::super::ItemResult;
use super::ActionMessage;
use crate::theme;

/// The log is always English, unlike the UI text `label()` returns.
const LOG_LABEL: &str = "Move in/out points out of file names";

pub fn label() -> String {
    fl!("batch-action-in-out-from-names")
}

pub fn view<'a>() -> Element<'a, ActionMessage> {
    let status = match frename_core::metadata_storage().in_out {
        InOutStorage::Comment => fl!("batch-action-in-out-from-names-status-comment"),
        InOutStorage::InVideo => fl!("batch-action-in-out-from-names-status-video"),
    };
    // The status wraps when the panel is narrow; the button keeps its label on one line.
    let settings = row![
        text(status).size(13).width(Length::Fill),
        button(
            text(fl!("batch-action-in-out-from-names-settings"))
                .size(12)
                .wrapping(iced::widget::text::Wrapping::None),
        )
        .on_press(ActionMessage::OpenSettings)
        .padding([3, 10])
        .style(theme::icon_button_style(true)),
    ]
    .spacing(12)
    .align_y(iced::Alignment::Center);
    super::panel(
        label(),
        fl!("batch-action-in-out-from-names-hint"),
        settings.into(),
    )
}

/// The points as the comment line shows them.
fn points(points: NameInOut) -> String {
    frename_core::format_in_out_line(points.start, points.end).unwrap_or_default()
}

/// Move the in/out points in the name of the file at `path` to where they are kept now.
pub fn run(path: &Path) -> ItemResult {
    let result = FileTagger::move_in_out_out_of_name(path);
    let failed = matches!(result.outcome, MoveOutcome::Failed(_));
    let mut item = super::item_result(result.outcome);
    if failed {
        log::warn!("{LOG_LABEL}: {path:?} could not be renamed");
        item.reason = Some(fl!("batch-action-in-out-from-names-not-renamed"));
    }
    if let Some((from_name, stored)) = result.kept_stored {
        log::info!(
            "{LOG_LABEL}: {path:?} keeps its stored {} and drops the name's {}",
            points(stored),
            points(from_name)
        );
        let kept = fl!(
            "batch-action-in-out-from-names-kept",
            stored = points(stored),
            name = points(from_name)
        );
        item.reason = Some(match item.reason {
            Some(reason) => format!("{reason}; {kept}"),
            None => kept,
        });
    }
    item
}
