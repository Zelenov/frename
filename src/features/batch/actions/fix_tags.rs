//! "Fix tags by priority": puts the tags in each file's name in the folder's tag order, the
//! order of the tag panel, as its sync-down button does for the open file. No options.

use std::path::Path;

use frename_core::FileTagger;
use iced::Element;

use super::super::page::{self, Change};
use super::super::ItemResult;
use crate::ui::text;

pub fn label() -> String {
    fl!("batch-action-fix-tags")
}

pub fn view<'a, M: 'a>() -> Element<'a, M> {
    page::page(
        label(),
        fl!("batch-action-fix-tags-hint"),
        &[Change::Renames],
        [text::secondary(fl!("batch-action-fix-tags-order")).into()],
    )
}

/// Sort the tags of the file at `path` in the folder's order.
pub fn run(path: &Path) -> ItemResult {
    super::item_result(FileTagger::sort_tags_by_folder_order(path))
}
