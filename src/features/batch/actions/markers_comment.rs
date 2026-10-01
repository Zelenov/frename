//! "Markers ⇄ comment": turns the timecoded lines of each file's comment (`03:24 — shaky`) into
//! clip markers, or copies each file's markers into its comment as such lines. One line format
//! serves both: `<time>[–<time>] — <name>[ — <comment>]`.

use std::path::Path;

use frename_core::FileTagger;
use iced::Element;

use super::super::page::{self, Change};
use super::super::{ItemResult, ItemStatus};
use crate::ui::{form, layout};

/// The log is always English, unlike the UI text `label()` returns.
const LOG_LABEL: &str = "Markers <-> comment";

pub fn label() -> String {
    fl!("batch-action-markers-comment")
}

/// Which way the lines go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Timecoded comment lines become markers and leave the comment.
    CommentToMarkers,
    /// Markers are copied into the comment as lines; they stay in the video.
    MarkersToComment,
}

#[derive(Debug, Clone)]
pub enum Message {
    SetDirection(Direction),
}

#[derive(Debug, Clone)]
pub struct Options {
    direction: Direction,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            direction: Direction::CommentToMarkers,
        }
    }
}

impl Direction {
    /// Stable name for persisting the last run's option (#65).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CommentToMarkers => "comment_to_markers",
            Self::MarkersToComment => "markers_to_comment",
        }
    }

    /// Parse a persisted name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        match name {
            "markers_to_comment" => Self::MarkersToComment,
            _ => Self::CommentToMarkers,
        }
    }
}

impl Options {
    /// Preset the direction, when Settings opens the action.
    pub fn prepare(&mut self, direction: Direction) {
        self.direction = direction;
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetDirection(direction) => self.direction = direction,
        }
    }

    pub fn operation(&self) -> super::Operation {
        super::Operation::MarkersComment(self.direction)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let choice = |label: String, description: Element<'static, Message>, direction| {
            form::radio_option(
                label,
                Some(description),
                direction,
                Some(self.direction),
                Message::SetDirection,
            )
        };
        let changes: &[Change] = match self.direction {
            Direction::CommentToMarkers => &[Change::IntoVideos, Change::IntoComments],
            Direction::MarkersToComment => &[Change::IntoComments],
        };
        page::page(
            label(),
            fl!("batch-action-markers-comment-hint-short"),
            changes,
            [page::option_row_with_info(
                fl!("batch-option-direction"),
                fl!("batch-action-markers-comment-hint"),
                layout::choices([
                    choice(
                        fl!("batch-action-markers-comment-to-markers"),
                        form::example(fl!("settings-markers-comment-example")),
                        Direction::CommentToMarkers,
                    ),
                    choice(
                        fl!("batch-action-markers-to-comment"),
                        form::description(fl!("batch-action-markers-to-comment-hint")),
                        Direction::MarkersToComment,
                    ),
                ]),
            )],
        )
    }
}

/// Convert the file at `path` in `direction`. Either way, markers the clip already has on the
/// same moment are merged into one.
pub fn run(direction: Direction, path: &Path) -> ItemResult {
    // Both ways also merge markers the clip already has on one moment; the report says how many.
    let outcome = match direction {
        Direction::CommentToMarkers => FileTagger::comment_to_markers_reporting(path),
        Direction::MarkersToComment => FileTagger::markers_to_comment_reporting(path),
    };
    match outcome {
        Ok((outcome, merged)) => {
            let mut result = super::item_result(outcome);
            if merged > 0 && result.status == ItemStatus::Done {
                result.reason = Some(fl!("batch-markers-merged", count = merged));
            }
            result
        }
        // The format cannot hold markers, the file is damaged, or the write failed (it is
        // read-only or open in Premiere): the comment is left as it was.
        Err(error) => {
            log::warn!("{LOG_LABEL}: {path:?} not changed: {error}");
            ItemResult::failed(error.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    fn clip_with_duplicates(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("frename-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let file = dir.join("clip.mov");
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("crates/frename-core/tests/fixtures/tiny.mov");
        std::fs::copy(fixture, &file).expect("copy fixture");
        let marker = |name: &str| {
            let mut m = frename_core::Marker::new(5_000);
            m.name = name.to_string();
            m
        };
        FileTagger::save_markers(
            &file,
            &[marker("Lion"), marker("Lion roars")],
            &HashSet::new(),
        )
        .expect("markers");
        file
    }

    /// Issue #145: a file whose duplicates were merged says how many in the result.
    #[test]
    fn a_file_whose_duplicate_markers_were_merged_says_so() {
        for direction in [Direction::MarkersToComment, Direction::CommentToMarkers] {
            let file = clip_with_duplicates(direction.as_str());
            let result = run(direction, &file);
            assert_eq!(result.status, ItemStatus::Done, "{direction:?}");
            assert_eq!(
                result.reason.as_deref(),
                Some("1 duplicate marker merged"),
                "{direction:?}"
            );
            assert_eq!(FileTagger::load_markers(&file).map(|m| m.len()), Some(1));
            let _ = std::fs::remove_dir_all(file.parent().expect("folder"));
        }
    }
}
