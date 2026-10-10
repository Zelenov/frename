//! The recovery journal of the open clip (see `frename_core::recovery`): its unsaved edits are
//! written to disk a second after they change, so a crash, a kill or a power cut loses at most
//! that second, and deleted once the edits are really applied to the clip.

use std::path::Path;

use frename_core::recovery::{self, Entry};
use frename_core::FileId;
use iced::Task;

use super::FolderWorkspace;
use crate::features::file_name_panel::{save_status, SaveStatus};
use crate::features::folder_workspace::Message;
use crate::features::media_viewer::{self, video};

impl FolderWorkspace {
    /// The open clip as it is now, as a journal entry.
    fn journal_entry(&self) -> Option<(FileId, Entry)> {
        let path = self.open_clip_path()?;
        self.journal_entry_at(&path)
    }

    /// Where the open clip is on disk now: an in-place rename reaches the open file's own path
    /// only with its next open, but the directory knows at once.
    fn open_clip_path(&self) -> Option<std::path::PathBuf> {
        let id = self.file_workspace.file()?.id();
        self.directory
            .as_ref()
            .and_then(|dir| dir.file_by_id(id))
            .map(|file| file.file_path().to_path_buf())
            .or_else(|| {
                self.file_workspace
                    .file()
                    .map(|f| f.file_path().to_path_buf())
            })
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
        if self.journal_in_flight.is_some() {
            return Task::none();
        }
        let Some((id, entry)) = self.journal_next_write() else {
            return Task::none();
        };
        self.journal_in_flight = Some(id);
        self.journal_saved_while_writing = false;
        Task::perform(
            async move {
                tokio::task::spawn_blocking(move || {
                    let result = recovery::write(&entry).map_err(|e| e.to_string());
                    (entry, result)
                })
                .await
                .ok()
            },
            move |outcome| Message::JournalWritten(id, outcome),
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
        // The baseline is set when a clip opens; these only catch a tick before that.
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
        outcome: Option<(Entry, Result<(), String>)>,
    ) {
        self.journal_in_flight = None;
        let saved_meanwhile = std::mem::take(&mut self.journal_saved_while_writing);
        match outcome {
            // The clip was saved while its entry was being written: the edits are applied, so
            // the entry just written must not outlive them.
            Some((entry, Ok(()))) if saved_meanwhile => recovery::remove(&entry.path),
            Some((entry, Ok(()))) => {
                // Another clip may be open by now: this one's entry stays on disk until its
                // own save removes it (that save may still be waiting for the video to unload).
                // The clip may have changed again while it wrote; the next tick sees that.
                if self
                    .journal_baseline
                    .as_ref()
                    .is_some_and(|(open, _)| *open == id)
                {
                    self.journal_written = Some((id, entry));
                }
            }
            Some((_, Err(e))) => log::warn!("recovery: the journal could not be written: {e}"),
            None => log::warn!("recovery: the journal write did not finish"),
        }
    }

    /// The edits of the clip `id` were applied (saved and read back): its journal entry has done
    /// its job. `path_before` is where the clip was, `new_path` where it is. When it is the open
    /// clip, what it is now is the new reference.
    pub(super) fn journal_saved(
        &mut self,
        id: FileId,
        path_before: &Path,
        new_path: &Path,
        saved: &frename_core::FileSnapshot,
    ) {
        if self.journal_in_flight == Some(id) {
            self.journal_saved_while_writing = true;
        }
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
            // What was saved is the reference, not what the clip is now: edits made since are
            // not on disk, so they are unsaved. The new name, as the rename reaches the open
            // clip only with its next open.
            match Entry::new(new_path, saved) {
                Some(entry) => self.journal_baseline = Some((id, entry)),
                // The clip cannot be read just now: keep the old reference, and journal what
                // differs from it rather than treat everything since as clean.
                None => self.journal_force = true,
            }
        }
    }

    /// What the line at the foot of the file name card says about the open clip: whether its
    /// edits are in its file, or only in the recovery journal. Asked on every frame, so it
    /// compares without touching the disk.
    pub fn save_status(&self) -> Option<SaveStatus> {
        let file = self.file_workspace.file()?;
        let id = file.id();
        let (baseline_id, baseline) = self.journal_baseline.as_ref()?;
        if *baseline_id != id || self.batch.is_running() {
            return None;
        }
        let path = self.open_clip_path()?;
        let snapshot = self.file_workspace.tag_list().file_snapshot();
        let unsaved = self.journal_force
            || !baseline.same_state_as(&snapshot)
            || self.unsaved_markers.contains_key(&id);
        // Size and modification time are not compared (that needs the disk): a clip changed on
        // disk keeps "in recovery" for at most a tick.
        let in_journal = self
            .journal_written
            .as_ref()
            .is_some_and(|(written_id, entry)| {
                // The journal keeps an entry per path: a clip renamed since has none yet.
                *written_id == id && entry.path == path && entry.same_state_as(&snapshot)
            });
        save_status(unsaved, in_journal)
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
