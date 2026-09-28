//! The filter of the file list (`docs/design/design-system.md` §13.4.1): after the search field,
//! since it narrows the same list. A dropdown with one row per filter and the number of files it
//! matches; picking a row turns it on or off. Its label says how many are on.

use iced::Element;

use crate::features::folder_workspace::Directory;
use crate::ui::form;

use super::Message;

/// Which list filter a dropdown row controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FilterKind {
    Untagged,
    Subtitles,
    Comments,
    Markers,
}

impl FilterKind {
    fn label(self) -> String {
        match self {
            Self::Untagged => fl!("folder-controls-filter-untagged"),
            Self::Subtitles => fl!("folder-controls-filter-subtitles"),
            Self::Comments => fl!("folder-controls-filter-comments"),
            Self::Markers => fl!("folder-controls-filter-markers"),
        }
    }

    fn set(self, on: bool) -> Message {
        match self {
            Self::Untagged => Message::SetUntaggedOnly(on),
            Self::Subtitles => Message::SetSubtitledOnly(on),
            Self::Comments => Message::SetCommentedOnly(on),
            Self::Markers => Message::SetMarkedOnly(on),
        }
    }
}

/// One row of the filter dropdown: whether the filter is on and how many files match it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FilterItem {
    kind: FilterKind,
    active: bool,
    count: usize,
}

impl std::fmt::Display for FilterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mark = if self.active { "✓" } else { "   " };
        write!(f, "{mark} {}  {}", self.kind.label(), self.count)
    }
}

fn items(dir: &Directory) -> [FilterItem; 4] {
    [
        FilterItem {
            kind: FilterKind::Untagged,
            active: dir.untagged_only(),
            count: dir.untagged_count(),
        },
        FilterItem {
            kind: FilterKind::Subtitles,
            active: dir.subtitled_only(),
            count: dir.subtitled_count(),
        },
        FilterItem {
            kind: FilterKind::Comments,
            active: dir.commented_only(),
            count: dir.commented_count(),
        },
        FilterItem {
            kind: FilterKind::Markers,
            active: dir.marked_only(),
            count: dir.marked_count(),
        },
    ]
}

/// The label of the closed dropdown: "Filter", or "Filter (2)" with two filters on.
fn label(active: usize) -> String {
    if active == 0 {
        fl!("folder-controls-filter")
    } else {
        fl!("folder-controls-filter-active", count = (active as i64))
    }
}

/// The filter dropdown of `dir`'s list. It has no tooltip: one would draw over the open list.
pub fn view<'a>(dir: &Directory) -> Element<'a, Message> {
    let items = items(dir);
    let active = items.iter().filter(|item| item.active).count();
    form::dropdown(items.to_vec(), None::<FilterItem>, |item| {
        item.kind.set(!item.active)
    })
    .placeholder(label(active))
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picking_a_filter_row_flips_that_filter() {
        let on = FilterItem {
            kind: FilterKind::Markers,
            active: true,
            count: 3,
        };
        assert!(matches!(
            on.kind.set(!on.active),
            Message::SetMarkedOnly(false)
        ));
        assert!(matches!(
            FilterKind::Untagged.set(true),
            Message::SetUntaggedOnly(true)
        ));
    }

    #[test]
    fn a_row_shows_whether_it_is_on_and_its_count() {
        let item = FilterItem {
            kind: FilterKind::Comments,
            active: true,
            count: 7,
        };
        let shown = item.to_string();
        assert!(shown.starts_with('✓') && shown.ends_with('7'), "{shown}");
    }
}
