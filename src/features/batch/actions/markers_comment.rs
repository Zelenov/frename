//! "Markers ⇄ comment": turns the timecoded lines of each file's comment (`03:24 — shaky`) into
//! clip markers, or copies each file's markers into its comment as such lines. One line format
//! serves both: `<time>[–<time>] — <name>[ — <comment>]`.

use std::path::Path;

use frename_core::FileTagger;
use iced::widget::{column, radio};
use iced::Element;

use super::super::ItemResult;

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
        let choices = column![
            radio(
                fl!("batch-action-markers-comment-to-markers"),
                Direction::CommentToMarkers,
                Some(self.direction),
                Message::SetDirection
            )
            .text_size(13),
            radio(
                fl!("batch-action-markers-to-comment"),
                Direction::MarkersToComment,
                Some(self.direction),
                Message::SetDirection
            )
            .text_size(13),
        ]
        .spacing(8);
        super::panel(
            label(),
            fl!("batch-action-markers-comment-hint"),
            choices.into(),
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
