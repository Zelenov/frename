//! "Move comments": moves each file's comment between the video (XMP) and a text file next to
//! it. Tags and in/out points stay where they are.

use std::path::Path;

use frename_core::{CommentStorage, FileTagger, MetadataMove};
use iced::widget::{column, radio};
use iced::Element;

use super::super::ItemResult;

pub const LABEL: &str = "Move comments";

#[derive(Debug, Clone)]
pub enum Message {
    /// Where the comments go.
    SetTo(CommentStorage),
}

#[derive(Debug, Clone)]
pub struct Options {
    to: CommentStorage,
}

impl Default for Options {
    fn default() -> Self {
        // A move usually brings files in line with the storage chosen in the settings.
        Self { to: frename_core::metadata_storage().comment }
    }
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetTo(to) => self.to = to,
        }
    }

    pub fn prepare(&mut self, to: CommentStorage) {
        self.to = to;
    }

    pub fn operation(&self) -> super::Operation {
        super::Operation::MoveComments(self.to)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let choices = column![
            radio("From text files into the videos (XMP)", CommentStorage::InVideo, Some(self.to), Message::SetTo)
                .text_size(13),
            radio("From the videos (XMP) into text files", CommentStorage::TextFile, Some(self.to), Message::SetTo)
                .text_size(13),
        ]
        .spacing(8);
        super::panel(
            LABEL,
            "Moves the comment of each checked file to the chosen place. Tags and in/out points stay where they are."
                .to_string(),
            choices.into(),
        )
    }
}

/// Move the comment of the file at `path` into `to`.
pub fn run(to: CommentStorage, path: &Path) -> ItemResult {
    super::item_result(FileTagger::move_metadata(path, MetadataMove::Comments(to)))
}
