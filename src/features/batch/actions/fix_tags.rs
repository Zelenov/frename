//! "Fix tags by priority": puts the tags in each file's name in the folder's tag order, the
//! order of the tag panel, as its sync-down button does for the open file. No options.

use std::path::Path;

use frename_core::FileTagger;
use iced::widget::column;
use iced::Element;

use super::super::ItemResult;

pub const LABEL: &str = "Fix tags by priority";

pub fn view<'a, M: 'a>() -> Element<'a, M> {
    super::panel(
        LABEL,
        "Puts the tags in the name of each checked file in the order of the tag panel, so the \
         higher a tag is there, the earlier it comes in the name. Tags the folder does not know \
         yet come first, as in the tag panel. Files whose order changes are renamed."
            .to_string(),
        column![].into(),
    )
}

/// Sort the tags of the file at `path` in the folder's order.
pub fn run(path: &Path) -> ItemResult {
    super::item_result(FileTagger::sort_tags_by_folder_order(path))
}
