//! "In/out points: comment ⇄ video (XMP)": moves each file's in/out points between a line of
//! its comment and an Adobe XMP marker in the video. The rest of the comment stays where it is.

use std::path::Path;

use frename_core::{FileTagger, InOutStorage, MetadataMove};
use iced::Element;

use super::super::page::{self, Change};
use super::super::ItemResult;
use crate::ui::{form, layout};

pub fn label() -> String {
    fl!("batch-action-move-in-out")
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Where the in/out points go.
    SetTo(InOutStorage),
}

#[derive(Debug, Clone)]
pub struct Options {
    to: InOutStorage,
}

impl Default for Options {
    fn default() -> Self {
        // A move usually brings files in line with the storage chosen in the settings.
        Self {
            to: frename_core::metadata_storage().in_out,
        }
    }
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetTo(to) => self.to = to,
        }
    }

    pub fn prepare(&mut self, to: InOutStorage) {
        self.to = to;
    }

    pub fn operation(&self) -> super::Operation {
        super::Operation::MoveInOut(self.to)
    }

    pub fn view(&self) -> Element<'_, Message> {
        let choice = |label: String, description: Element<'static, Message>, to| {
            form::radio_option(label, Some(description), to, Some(self.to), Message::SetTo)
        };
        let change = match self.to {
            InOutStorage::InVideo => Change::IntoVideos,
            InOutStorage::Comment => Change::IntoComments,
        };
        page::page(
            label(),
            fl!("batch-action-move-in-out-hint"),
            &[change],
            [page::option_row(
                fl!("batch-option-direction"),
                layout::choices([
                    choice(
                        fl!("batch-action-move-in-out-into-videos"),
                        form::description(fl!("settings-in-out-in-video-hint")),
                        InOutStorage::InVideo,
                    ),
                    choice(
                        fl!("batch-action-move-in-out-into-comments"),
                        form::example(fl!("settings-in-out-comment-example")),
                        InOutStorage::Comment,
                    ),
                ]),
            )],
        )
    }
}

/// Move the in/out points of the file at `path` into `to`.
pub fn run(to: InOutStorage, path: &Path) -> ItemResult {
    super::item_result(FileTagger::move_metadata(path, MetadataMove::InOut(to)))
}
