//! "Move in/out points": moves each file's in/out points between its name and an Adobe XMP
//! marker in the video. Comments stay where they are.

use std::path::Path;

use frename_core::{FileTagger, InOutStorage, MetadataMove};
use iced::widget::{column, radio};
use iced::Element;

use super::super::ItemResult;

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
        let choices = column![
            radio(
                fl!("batch-action-move-in-out-into-videos"),
                InOutStorage::InVideo,
                Some(self.to),
                Message::SetTo,
            )
            .text_size(13),
            radio(
                fl!("batch-action-move-in-out-into-file-names"),
                InOutStorage::FileName,
                Some(self.to),
                Message::SetTo
            )
            .text_size(13),
        ]
        .spacing(8);
        super::panel(
            label(),
            fl!("batch-action-move-in-out-hint"),
            choices.into(),
        )
    }
}

/// Move the in/out points of the file at `path` into `to`.
pub fn run(to: InOutStorage, path: &Path) -> ItemResult {
    super::item_result(FileTagger::move_metadata(path, MetadataMove::InOut(to)))
}
