//! "Reset cache and reload": reads each file's comment and in/out points from the file again,
//! replacing what the folder's `.frename` file list remembered, and refreshes its row. No options.

use std::path::Path;

use frename_core::FileTagger;
use iced::widget::column;
use iced::Element;

use super::super::{ItemResult, ItemStatus};

pub fn label() -> String {
    fl!("batch-action-reload-files")
}

pub fn view<'a, M: 'a>() -> Element<'a, M> {
    super::panel(
        label(),
        fl!("batch-action-reload-files-hint"),
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
