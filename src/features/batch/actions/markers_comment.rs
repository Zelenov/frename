//! "Markers ⇄ comment": turns the timecoded lines of each file's comment (`03:24 — shaky`) into
//! clip markers, or copies each file's markers into its comment as such lines. One line format
//! serves both: `<time>[–<time>] — <name>[ — <comment>]`.

use std::path::Path;

use frename_core::FileTagger;
use iced::Element;

use super::super::page::{self, Change};
use super::super::ItemResult;
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
            [layout::setting_row_with_info(
                fl!("batch-option-direction"),
                fl!("batch-action-markers-comment-hint"),
                layout::aligned([
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

/// Convert the file at `path` in `direction`.
pub fn run(direction: Direction, path: &Path) -> ItemResult {
    let outcome = match direction {
        Direction::CommentToMarkers => FileTagger::comment_to_markers(path),
        Direction::MarkersToComment => FileTagger::markers_to_comment(path),
    };
    match outcome {
        Ok(outcome) => super::item_result(outcome),
        // The format cannot hold markers, the file is damaged, or the write failed (it is
        // read-only or open in Premiere): the comment is left as it was.
        Err(error) => {
            log::warn!("{LOG_LABEL}: {path:?} not changed: {error}");
            ItemResult::failed(error.to_string())
        }
    }
}
