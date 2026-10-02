//! "Move comments": moves each file's comment between the video (XMP) and a text file next to
//! it. Tags and in/out points stay where they are.

use std::path::Path;

use frename_core::{CommentStorage, FileTagger, MetadataMove};
use iced::Element;

use super::super::page::{self, Change};
use super::super::ItemResult;
use crate::ui::{form, layout};

pub fn label() -> String {
    fl!("batch-action-move-comments")
}

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
        Self {
            to: frename_core::metadata_storage().comment,
        }
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
        let choice = |label: String, description: Element<'static, Message>, to| {
            form::radio_option(label, Some(description), to, Some(self.to), Message::SetTo)
        };
        let change = match self.to {
            CommentStorage::InVideo => Change::IntoVideos,
            CommentStorage::TextFile => Change::TextFiles,
        };
        page::page(
            label(),
            fl!("batch-action-move-comments-hint"),
            &[change],
            [page::option_row(
                fl!("batch-option-direction"),
                layout::choices([
                    choice(
                        fl!("batch-action-move-comments-into-videos"),
                        form::description(fl!("settings-comments-in-video-hint")),
                        CommentStorage::InVideo,
                    ),
                    choice(
                        fl!("batch-action-move-comments-into-text-files"),
                        form::example(fl!("settings-comments-text-file-example")),
                        CommentStorage::TextFile,
                    ),
                ]),
            )],
        )
    }
}

/// Move the comment of the file at `path` into `to`.
pub fn run(to: CommentStorage, path: &Path) -> ItemResult {
    super::item_result(FileTagger::move_metadata(path, MetadataMove::Comments(to)))
}
