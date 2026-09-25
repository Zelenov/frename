//! "Reset cache and reload": reads each file's comment and in/out points from the file again,
//! replacing what the folder's `.frename` file list remembered, and refreshes its row. No options.

use std::path::Path;

use frename_core::FileTagger;
use iced::widget::column;
use iced::Element;

use super::super::{ItemResult, ItemStatus};

pub const LABEL: &str = "Reset cache and reload";

pub fn view<'a, M: 'a>() -> Element<'a, M> {
    super::panel(
        LABEL,
        "Reads the comment and in/out points of each checked file from the file itself again and \
         replaces what the folder remembered for it. Use it after the files were changed in another \
         program. Files whose remembered values were missing or out of date count as changed."
            .to_string(),
        column![].into(),
    )
}

/// Reload the file at `path`. Its row is refreshed either way; it counts as changed when the
/// cached values were missing or stale.
pub fn run(path: &Path) -> ItemResult {
    let status = if FileTagger::reload_metadata(path) {
        ItemStatus::Done
    } else {
        ItemStatus::Skipped
    };
    ItemResult {
        status,
        update: Some(super::reparsed(path.to_path_buf())),
    }
}
