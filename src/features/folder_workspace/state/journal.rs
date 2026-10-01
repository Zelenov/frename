//! The recovery journal of the open clip (see `frename_core::recovery`): its unsaved edits are
//! written to disk a second after they change, so a crash, a kill or a power cut loses at most
//! that second, and deleted once the edits are really applied to the clip.

use std::path::Path;

use frename_core::recovery::{self, Entry};
use frename_core::FileId;
use iced::Task;

use super::FolderWorkspace;
use crate::features::folder_workspace::Message;
use crate::features::media_viewer::{self, video};

impl FolderWorkspace {
    /// The open clip as it is now, as a journal entry.
    fn journal_entry(&self) -> Option<(FileId, Entry)> {
        let path = self.file_workspace.file()?.file_path().to_path_buf();
        self.journal_entry_at(&path)
    }

    /// The open clip's state as an entry for the clip at `path`.
    fn journal_entry_at(&self, path: &Path) -> Option<(FileId, Entry)> {
        let (id, snapshot) = self.file_workspace.get_snapshot()?;
        Some((id, Entry::new(path, &snapshot)?))
    }

    /// The clip just opened is the reference: only what differs from it is unsaved. A clip
    /// opened again while its save still waits for the video to unload has edits that are not
    /// on disk: they count as unsaved from the start.
    pub(super) fn journal_reset_baseline(&mut self) {
        // A write still running belongs to the clip left: its result is dropped when it lands.
        self.journal_epoch += 1;
        self.journal_baseline = self.journal_entry();
        self.journal_written = None;
        self.journal_force = self
            .journal_baseline
            .as_ref()
            .is_some_and(|(id, _)| self.pending_file_updates.iter().any(|(p, _)| p == id));
    }

    /// Once a second: write the open clip's edits into the journal when they changed, and take
    /// the entry out again when they are gone (undone, or saved). The write (it ends in an
    /// `fsync`) runs off the interface's thread.
    pub(super) fn journal_tick(&mut self) -> Task<Message> {
        // One write at a time: a slow disk must not pile up writes to the same file.
        if self.journal_in_flight {
            return Task::none();
        }
        let Some((id, entry)) = self.journal_next_write() else {
            return Task::none();
        };
        self.journal_in_flight = true;
        let epoch = self.journal_epoch;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let result = recovery::write(&entry).map_err(|e| e.to_string());
                    (entry, result)
                })
                .await
                .ok()
            },
            move |outcome| Message::JournalWritten(id, epoch, outcome),
        )
    }

    /// What the journal needs written now, if anything; an entry that is not needed any more is
    /// removed here.
    pub(super) fn journal_next_write(&mut self) -> Option<(FileId, Entry)> {
        // A running job locks the folder and writes its files itself.
        if self.batch.is_running() {
            return None;
        }
        let (id, now) = self.journal_entry()?;
        let Some((baseline_id, baseline)) = self.journal_baseline.as_ref() else {
            self.journal_baseline = Some((id, now));
            return None;
        };
        if *baseline_id != id {
            self.journal_baseline = Some((id, now));
            self.journal_written = None;
            self.journal_force = false;
            return None;
        }
        let dirty = self.journal_force
            || !baseline.same_state(&now)
            || self.unsaved_markers.contains_key(&id);
        let written = self
            .journal_written
            .as_ref()
            .is_some_and(|(written_id, entry)| *written_id == id && *entry == now);
        if dirty {
            return (!written).then_some((id, now));
        }
        if let Some((_, entry)) = self.journal_written.take() {
            recovery::remove(&entry.path);
        }
        None
    }

    /// A journal write finished: remember what the journal holds now.
    pub(super) fn journal_written(
        &mut self,
        id: FileId,
        epoch: u64,
        outcome: Option<(Entry, Result<(), String>)>,
    ) {
        self.journal_in_flight = false;
        match outcome {
            // The clip was saved, or another one opened, while this wrote: its edits are
            // applied, so the entry it just wrote must not outlive them.
            Some((entry, Ok(()))) if epoch != self.journal_epoch => recovery::remove(&entry.path),
            Some((entry, Ok(()))) => {
                // The clip may have changed again while it wrote; the next tick sees that.
                self.journal_written = Some((id, entry));
            }
            Some((_, Err(e))) => log::warn!("recovery: the journal could not be written: {e}"),
            None => log::warn!("recovery: the journal write did not finish"),
        }
    }

    /// The edits of the clip `id` were applied (saved and read back): its journal entry has done
    /// its job. `path_before` is where the clip was, `new_path` where it is. When it is the open
    /// clip, what it is now is the new reference.
    pub(super) fn journal_saved(&mut self, id: FileId, path_before: &Path, new_path: &Path) {
        self.journal_epoch += 1;
        recovery::remove(path_before);
        if let Some((written_id, entry)) = self.journal_written.take() {
            if written_id == id {
                recovery::remove(&entry.path);
            } else {
                self.journal_written = Some((written_id, entry));
            }
        }
        self.journal_force = false;
        if self.file_workspace.file().is_some_and(|f| f.id() == id) {
            // The open clip's own path still names the clip as it was (the rename reaches it
            // with the next open), so the reference is taken for the new name.
            self.journal_baseline = self.journal_entry_at(new_path);
        }
    }

    /// A message from the start (edits restored after a crash) to show once a clip is open.
    pub fn add_startup_note(&mut self, note: String) {
        self.startup_notes.push(note);
    }

    /// Show the start-up messages over the open clip's picture, once a clip is open: all in one
    /// long note (a note replaces the one before it, and this one must be read).
    pub(super) fn show_startup_notes(&mut self) -> Task<Message> {
        if self.file_workspace.file().is_none() || self.startup_notes.is_empty() {
            return Task::none();
        }
        let text = std::mem::take(&mut self.startup_notes).join(" ");
        Task::done(Message::MediaViewer(media_viewer::Message::Video(
            video::Message::ShowLongNotice(text),
        )))
    }
}
