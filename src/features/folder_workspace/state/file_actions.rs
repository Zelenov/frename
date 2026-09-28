//! The file context menu's actions (#48): show the file in Explorer or the file manager, copy
//! its full path, copy its name. The file's pending edits are saved first, so the action sees
//! the file's final name.

use frename_core::{FileId, FileTagger};
use iced::Task;

use super::FolderWorkspace;
use crate::features::file_menu::{self, system, FileAction};
use crate::features::folder_workspace::Message;

impl FolderWorkspace {
    /// Messages of the menu; a chosen item runs its action on the menu's file.
    pub(super) fn handle_file_menu(&mut self, msg: file_menu::Message) -> Task<Message> {
        let chosen = match msg {
            file_menu::Message::Choose(id, action) => Some((id, action)),
            _ => None,
        };
        let task = self.file_menu.update(msg).map(Message::FileMenu);
        match chosen {
            Some((id, action)) => Task::batch([task, self.request_file_action(id, action)]),
            None => task,
        }
    }

    /// A menu key: the action on the open file. An open menu closes, as a key press elsewhere
    /// would close it.
    pub(super) fn file_action_on_open_file(&mut self, action: FileAction) -> Task<Message> {
        let Some(id) = self
            .directory
            .as_ref()
            .and_then(|dir| dir.selected_file())
            .map(|file| file.id())
        else {
            return Task::none();
        };
        let close = self.handle_file_menu(file_menu::Message::Close);
        Task::batch([close, self.request_file_action(id, action)])
    }

    /// Run `action` on the file `id` once its edits are on disk: the open file with edits not
    /// saved yet is saved first, as a click on its row saves it (a same-file refresh, which
    /// unloads a playing video first), and a save already under way is waited for. The save
    /// runs the action ([`Self::file_action_after_save`]). Any other file runs it at once.
    fn request_file_action(&mut self, id: FileId, action: FileAction) -> Task<Message> {
        // A new request replaces one still waiting: only the last key or item counts.
        self.pending_file_action = None;
        let is_open = self.file_workspace.file().is_some_and(|f| f.id() == id);
        if is_open && self.open_file_unsaved(id) {
            let Some(file) = self
                .directory
                .as_ref()
                .and_then(|dir| dir.file_by_id(id))
                .cloned()
            else {
                return Task::none();
            };
            self.pending_file_action = Some((id, action));
            return Task::done(Message::FileOpened(file));
        }
        if self
            .pending_file_updates
            .iter()
            .any(|(queued, _)| *queued == id)
        {
            self.pending_file_action = Some((id, action));
            return Task::none();
        }
        Task::done(Message::RunFileAction(id, action))
    }

    /// The file `id` was just saved: the action waiting for that save runs now. Not when the
    /// save was refused: the name the action was asked for is not the file's, and the
    /// workspace's notice says why it was not saved.
    pub(super) fn file_action_after_save(&mut self, id: FileId, refused: bool) -> Task<Message> {
        let Some((waiting, action)) = self.pending_file_action else {
            return Task::none();
        };
        if waiting != id {
            return Task::none();
        }
        self.pending_file_action = None;
        if refused {
            return Task::none();
        }
        Task::done(Message::RunFileAction(id, action))
    }

    /// Run `action` on the file `id` as it is now.
    pub(super) fn run_file_action(&mut self, id: FileId, action: FileAction) -> Task<Message> {
        let Some(path) = self
            .directory
            .as_ref()
            .and_then(|dir| dir.file_by_id(id))
            .map(|file| file.file_path().to_path_buf())
        else {
            return Task::none();
        };
        let Some(text) = system::clipboard_text(action, &path) else {
            // Where the file really is (debug builds rename only in memory). Explorer and the
            // file manager are other programs: waited for off the UI thread.
            let path = FileTagger::disk_path(&path);
            return Task::perform(system::show_in_file_manager(path), |result| match result {
                Ok(()) => Message::Noop,
                Err(e) => {
                    log::warn!("could not show the file in the file manager: {e}");
                    Self::notice_message(&fl!("file-menu-not-shown"))
                }
            });
        };
        match self.set_clipboard_text(text) {
            Ok(()) => Self::notice(&fl!("file-menu-copied")),
            Err(e) => {
                log::warn!("could not copy to the clipboard: {e}");
                Self::notice(&fl!("file-menu-not-copied"))
            }
        }
    }

    /// Put `text` on the system clipboard, which stays open from the first copy on: on Linux
    /// the copied text is served by the clipboard object, and a clipboard manager may not have
    /// taken it yet when it closes right away.
    pub(super) fn set_clipboard_text(&mut self, text: String) -> Result<(), arboard::Error> {
        let clipboard = match self.clipboard.as_mut() {
            Some(clipboard) => clipboard,
            None => self.clipboard.insert(arboard::Clipboard::new()?),
        };
        clipboard.set_text(text)
    }
}
