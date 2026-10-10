//! The recent folders list (#63): doing what [`recent_folders::Effect`] asks — reading the list
//! from the database, checking which folders still exist, opening one, forgetting one — and
//! routing the keys to the list while it is open.

use std::path::PathBuf;

use frename_core::recent_folders::now_ms;
use frename_core::AppStateStore;
use iced::widget::scrollable::RelativeOffset;
use iced::Task;

use super::FolderWorkspace;
use crate::features::folder_workspace::Message;
use crate::features::recent_folders::{self, Effect, RECENT_LIST_SCROLLABLE_ID};
use crate::features::{folder, tag_panel};

impl FolderWorkspace {
    /// While the list is open it has the arrows, Enter and Esc; the keys it does not use are
    /// dropped, so nothing happens behind it. `None`: the key is not the list's (or it is
    /// closed); `Some(None)`: swallowed.
    pub(super) fn recent_folders_key(
        &self,
        message: &Message,
    ) -> Option<Option<recent_folders::Message>> {
        use recent_folders::Message as Recent;
        if !self.recent_folders.is_open() {
            return None;
        }
        match message {
            Message::TagPanel(tag_panel::Message::SelectUp) => Some(Some(Recent::Move(-1))),
            Message::TagPanel(tag_panel::Message::SelectDown) => Some(Some(Recent::Move(1))),
            Message::SaveSelectedTag => Some(Some(Recent::ChooseHighlighted)),
            Message::EscapePressed => Some(Some(Recent::Escape)),
            Message::TagPanel(
                tag_panel::Message::SelectLeft
                | tag_panel::Message::SelectRight
                | tag_panel::Message::ToggleSelectedTag,
            )
            | Message::Folder(folder::Message::PreviousFile | folder::Message::NextFile)
            | Message::FocusSearchBarAndKey(_)
            | Message::RemoveTag => Some(None),
            _ => None,
        }
    }

    pub(super) fn handle_recent_folders(
        &mut self,
        message: recent_folders::Message,
    ) -> Task<Message> {
        // Fullscreen shows no file list to open the dropdown over.
        if self.media_fullscreen && matches!(message, recent_folders::Message::Toggle) {
            return Task::none();
        }
        let walked = matches!(message, recent_folders::Message::Move(_));
        match self.recent_folders.update(message) {
            Effect::None if walked => self.scroll_recent_folders_to_highlight(),
            Effect::None => Task::none(),
            Effect::Reload => Task::batch([
                self.reload_recent_folders(),
                // Ctrl+R can come while a search field has the keys: it would keep Enter.
                iced::advanced::widget::operate(
                    iced::advanced::widget::operation::focusable::unfocus::<()>(),
                )
                .map(|()| Message::Noop),
            ]),
            Effect::Open(pair) => Task::done(Message::ScanFolder(pair)),
            Effect::Forget(folder) => {
                self.app_db.forget_recent_folder(&folder);
                Task::none()
            }
            Effect::Clear => {
                self.app_db.clear_recent_folders();
                Task::none()
            }
        }
    }

    /// Read the list from the database and check, off the interface's thread, which of its
    /// folders exist: a disconnected network drive can take seconds to answer.
    pub(super) fn reload_recent_folders(&mut self) -> Task<Message> {
        self.recent_folders
            .set_entries(self.app_db.get_recent_folders(), now_ms());
        let folders = self.recent_folders.folders();
        if folders.is_empty() {
            return Task::none();
        }
        Task::future(async move {
            let results = tokio::task::spawn_blocking(move || {
                folders
                    .into_iter()
                    .map(|folder| {
                        let found = folder.is_dir();
                        (folder, found)
                    })
                    .collect::<Vec<(PathBuf, bool)>>()
            })
            .await
            .unwrap_or_default();
            Message::RecentFolders(recent_folders::Message::Checked(results))
        })
    }

    /// Keep the highlighted row of the list on the empty screen in view.
    fn scroll_recent_folders_to_highlight(&self) -> Task<Message> {
        let Some(y) = self.recent_folders.highlight_fraction() else {
            return Task::none();
        };
        iced::widget::operation::snap_to(
            iced::widget::Id::new(RECENT_LIST_SCROLLABLE_ID),
            RelativeOffset {
                x: None,
                y: Some(y),
            },
        )
    }
}
