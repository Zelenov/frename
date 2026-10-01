//! The recovery journal of the open clip (see `frename_core::recovery`): its unsaved edits are
//! written to disk a second after they change, so a crash, a kill or a power cut loses at most
//! that second, and deleted once the edits are really applied to the clip.

use frename_core::recovery::{self, Entry};
use frename_core::FileId;
use iced::Task;

use super::FolderWorkspace;
use crate::features::folder_workspace::Message;

impl FolderWorkspace {
    /// The open clip as it is now, as a journal entry.
    fn journal_entry(&self) -> Option<(FileId, Entry)> {
        let (id, snapshot) = self.file_workspace.get_snapshot()?;
        let path = self.file_workspace.file()?.file_path().to_path_buf();
        Some((id, Entry::new(&path, &snapshot)?))
    }

    /// The clip just opened is the reference: only what differs from it is unsaved.
    pub(super) fn journal_reset_baseline(&mut self) {
        self.journal_baseline = self.journal_entry();
        self.journal_written = None;
    }

    /// Once a second: write the open clip's edits into the journal when they changed, and take
    /// the entry out again when they are gone (undone, or saved).
    pub(super) fn journal_tick(&mut self) {
        // A running job locks the folder and writes its files itself.
        if self.batch.is_running() {
            return;
        }
        let Some((id, now)) = self.journal_entry() else {
            return;
        };
        let Some((baseline_id, baseline)) = self.journal_baseline.as_ref() else {
            self.journal_baseline = Some((id, now));
            return;
        };
        if *baseline_id != id {
            self.journal_baseline = Some((id, now));
            self.journal_written = None;
            return;
        }
        let dirty = !baseline.same_state(&now) || self.unsaved_markers.contains_key(&id);
        let written = self
            .journal_written
            .as_ref()
            .is_some_and(|(written_id, entry)| *written_id == id && *entry == now);
        if dirty && !written {
            match recovery::write(&now) {
                Ok(()) => self.journal_written = Some((id, now)),
                Err(e) => log::warn!("recovery: the journal could not be written: {e}"),
            }
        } else if !dirty && self.journal_written.is_some() {
            recovery::remove(&now.path);
            self.journal_written = None;
        }
    }

    /// The edits of the clip at `path` are applied (saved): its journal entry has done its job.
    /// When it is the open clip, what it is now is the new reference.
    pub(super) fn journal_saved(&mut self, id: FileId, path_before: &std::path::Path) {
        recovery::remove(path_before);
        if self.journal_written.as_ref().is_some_and(|(w, _)| *w == id) {
            self.journal_written = None;
        }
        if self.file_workspace.file().is_some_and(|f| f.id() == id) {
            self.journal_baseline = self.journal_entry();
        }
    }

    /// A message from the start (edits restored after a crash) to show once a clip is open.
    pub fn add_startup_note(&mut self, note: String) {
        self.startup_notes.push(note);
    }

    /// Show the start-up messages over the open clip's picture, once a clip is open.
    pub(super) fn show_startup_notes(&mut self) -> Task<Message> {
        if self.file_workspace.file().is_none() {
            return Task::none();
        }
        let notes: Vec<String> = std::mem::take(&mut self.startup_notes);
        Task::batch(notes.iter().map(|note| Self::notice(note)))
    }
}
