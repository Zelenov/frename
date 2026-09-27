//! Dragging files from the file list out of the window (into Premiere, Explorer): a press on a
//! row arms the drag, a move past the threshold asks for it, and the open file's edits are on
//! disk before the operating system's drag starts. See `docs/design/drag-to-premiere.md`.

use std::path::PathBuf;

use frename_core::FileId;
use iced::Task;

use super::FolderWorkspace;
use crate::features::drag_out::{self, Readiness};
use crate::features::folder_workspace::Message;

impl FolderWorkspace {
    /// The row at `index` of the file list was pressed (and is being selected): holding the
    /// button and moving may drag its file out of the window.
    pub(super) fn arm_drag_out(&mut self, index: usize) {
        if !drag_out::SUPPORTED {
            return;
        }
        let pressed = self
            .directory
            .as_ref()
            .and_then(|dir| dir.files_in_order().nth(index))
            .map(|file| file.id());
        if let Some(id) = pressed {
            self.drag_out.press(id);
        }
    }

    /// Mouse events while a file row is pressed.
    pub(super) fn handle_drag_out(&mut self, msg: drag_out::Message) -> Task<Message> {
        match msg {
            drag_out::Message::Released => {
                self.drag_out.release();
                Task::none()
            }
            drag_out::Message::Moved(position) => match self.drag_out.moved(position) {
                Some(pressed) => self.ask_drag_out(pressed),
                None => Task::none(),
            },
        }
    }

    /// The drag loop returned (the platform code logged how): a new press may start another.
    pub(super) fn drag_out_finished(&mut self, _outcome: drag_out::Outcome) -> Task<Message> {
        self.drag_out.finish();
        Task::none()
    }

    /// The pointer went past the threshold: start the drag when the files are ready, save the
    /// open file first when its edits are not on disk, or wait for a save under way.
    fn ask_drag_out(&mut self, pressed: FileId) -> Task<Message> {
        // The release may have come in the same batch as the press (a touchpad tap), before
        // anything listened for it: with the button up there is no drag, and no save for one.
        if !drag_out::primary_button_down() {
            self.drag_out.release();
            return Task::none();
        }
        let Some(dir) = self.directory.as_ref() else {
            self.drag_out.release();
            return Task::none();
        };
        let listed = dir.files_in_order().map(|file| file.id());
        let ids = drag_out::files_to_drag(pressed, listed, self.batch.is_active(), |id| {
            self.batch.is_checked(id)
        });
        let open = self
            .file_workspace
            .file()
            .map(|file| file.id())
            .filter(|id| ids.contains(id));
        let unsaved = open.is_some_and(|id| self.open_file_unsaved(id));
        // A save counts once it ran, not when it was asked for: its message may still be
        // queued, or its video still unloading.
        let in_flight = self.pending_file_updated.is_some() || self.drag_out.save_pending();
        match drag_out::readiness(in_flight, unsaved, self.drag_out.save_ran()) {
            Readiness::Wait => Task::none(),
            Readiness::SaveFirst => {
                // Saved as a click on its row saves it: a same-file refresh, which unloads
                // its video first when one plays.
                match open.and_then(|id| dir.file_by_id(id)).cloned() {
                    Some(file) => {
                        self.drag_out.ask_save(file.id());
                        Task::done(Message::FileOpened(file))
                    }
                    None => Task::none(),
                }
            }
            Readiness::Refuse => {
                self.drag_out.release();
                Self::notice(&fl!("drag-out-not-saved"))
            }
            Readiness::Start => {
                let paths: Vec<PathBuf> = ids
                    .iter()
                    .filter_map(|id| dir.file_by_id(*id))
                    .map(|file| file.file_path().to_path_buf())
                    .collect();
                if paths.is_empty() {
                    self.drag_out.release();
                    return Task::none();
                }
                self.drag_out.start();
                Task::done(Message::StartDragOut(paths))
            }
        }
    }

    /// Whether the open file `id` has edits that are not on disk: its name on disk is not the
    /// one its tags, name and in/out make, or its markers could not be written.
    pub(super) fn open_file_unsaved(&self, id: FileId) -> bool {
        if self.unsaved_markers.contains_key(&id) {
            return true;
        }
        let Some((_, wanted)) = self.file_workspace.get_snapshot() else {
            return false;
        };
        let Some(on_disk) = self.directory.as_ref().and_then(|dir| dir.file_by_id(id)) else {
            return false;
        };
        wanted.file_name() != on_disk.snapshot().file_name()
    }
}
