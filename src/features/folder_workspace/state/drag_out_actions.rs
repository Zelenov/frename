//! Dragging files from the file list out of the window (into Premiere, Explorer): a press on a
//! row arms the drag, a move past the threshold asks for it, and the open file's edits are on
//! disk before the operating system's drag starts. See `docs/design/drag-to-premiere.md`.

use std::path::PathBuf;

use frename_core::{FileId, FileSnapshot, FileTagger};
use iced::Task;

use super::FolderWorkspace;
use crate::features::drag_out::{self, DragCheck, Readiness, Save};
use crate::features::folder_workspace::Message;

impl FolderWorkspace {
    /// The row at `index` of the file list was pressed (and is being selected): holding the
    /// button and moving may drag its file out of the window. A press whose button is already
    /// up again (a tap) arms nothing, so a later drag elsewhere cannot pick this file up.
    pub(super) fn arm_drag_out(&mut self, index: usize) {
        if !drag_out::SUPPORTED || !drag_out::primary_button_down() {
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
    pub(super) fn drag_out_finished(&mut self) -> Task<Message> {
        self.drag_out.finish();
        Task::none()
    }

    /// A file was saved to `saved` (reparsed after the save) while `wanted` was asked for: a
    /// drag waiting for that save learns whether it worked. It failed when the rename did not
    /// happen or the markers could not be written.
    pub(super) fn drag_out_saved(
        &mut self,
        id: FileId,
        wanted: &FileSnapshot,
        saved: &FileSnapshot,
    ) {
        let failed = drag_out::save_failed(wanted, saved) || self.unsaved_markers.contains_key(&id);
        self.drag_out.saved(id, failed);
    }

    /// The pointer went past the threshold: start the drag when the files are ready, save the
    /// open file first when its edits may not be on disk, or wait for a save under way.
    fn ask_drag_out(&mut self, pressed: FileId) -> Task<Message> {
        // The release may have come in the same batch as the press (a touchpad tap), before
        // anything listened for it: with the button up there is no drag, and no save for one.
        // A double-click opened the rename editor: the held button is for its text now.
        if !drag_out::primary_button_down() || self.inline_rename.is_some() {
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
        // Every dragged file counts: in batch mode the press saved the file it left, which may
        // be among the checked ones too.
        let save_of = |id: FileId| self.drag_out.save_of(id);
        let check = DragCheck {
            waiting: !self.pending_file_updated.is_empty()
                || ids.iter().any(|id| save_of(*id) == Some(Save::Pending)),
            failed: ids.iter().any(|id| {
                save_of(*id) == Some(Save::Failed)
                    // The open file's markers are retried by a save first; another file's
                    // wait for that file to be opened again.
                    || (Some(*id) != open && self.unsaved_markers.contains_key(id))
            }),
            open_unsaved: open.is_some_and(|id| self.open_file_unsaved(id)),
            open_saved: open.is_some_and(|id| save_of(id) == Some(Save::Worked)),
        };
        match drag_out::readiness(check) {
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
                    // Where the file really is (debug builds rename only in memory).
                    .map(|file| FileTagger::disk_path(file.file_path()))
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

    /// Whether the open file `id` may have edits that are not on disk: its name on disk is not
    /// the one its edits make, or its markers could not be written. Only decides whether to
    /// save before the drag; whether that save worked is the save's own outcome.
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
