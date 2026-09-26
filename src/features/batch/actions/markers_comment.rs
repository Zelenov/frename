//! "Markers ⇄ comment": turns the timecoded lines of each file's comment (`03:24 — shaky`) into
//! clip markers, or copies each file's markers into its comment as such lines. One line format
//! serves both: `<time>[–<time>] — <name>[ — <comment>]`.

use std::path::Path;

use frename_core::FileTagger;
use iced::widget::{column, radio};
use iced::Element;

use super::super::{ItemResult, ItemStatus};

pub const LABEL: &str = "Markers ⇄ comment";

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
                "Comment lines with a time into markers",
                Direction::CommentToMarkers,
                Some(self.direction),
                Message::SetDirection
            )
            .text_size(13),
            radio(
                "Markers into the comment (a copy: the markers stay)",
                Direction::MarkersToComment,
                Some(self.direction),
                Message::SetDirection
            )
            .text_size(13),
        ]
        .spacing(8);
        super::panel(
            LABEL,
            "A line like “03:24 — Take 3 — nice light” is a marker at 3:24 named “Take 3” with the \
             comment “nice light”; “0:41-0:47 — Lion” is a marker from 0:41 to 0:47. The name \
             and the comment are split at the first “ — ” or “ -- ”, not at a plain “ - ”. \
             Running either way again adds nothing twice."
                .to_string(),
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
        // The format cannot hold markers, or the write failed (the file is read-only or open
        // in Premiere): the comment is left as it was.
        Err(error) => {
            log::warn!("{LABEL}: {path:?} not changed: {error}");
            ItemResult {
                status: ItemStatus::Failed,
                update: None,
            }
        }
    }
}
