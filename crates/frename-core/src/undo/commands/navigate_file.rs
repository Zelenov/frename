use std::path::{Path, PathBuf};

use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::tags::target_name_taken_with;
use crate::{FileId, FileSnapshot};

/// Records a file navigation (next / previous / indexed) together with its deferred save.
/// One Ctrl+Z reverses both the rename on disk and the file selection change.
pub struct NavigateFileCommand {
    /// Stable id of the file that was renamed (navigated away from).
    pub file_id: FileId,
    /// Stable id of the file that was navigated to (restored on redo).
    pub to_file_id: FileId,
    /// Path of the file before the deferred rename.
    pub path_before: PathBuf,
    /// Path after save_and_reparse (may equal path_before when no tags were checked).
    pub path_after: PathBuf,
    pub snapshot_before: FileSnapshot,
    pub snapshot_after: FileSnapshot,
    /// The save moved the clip's files named without the extension (subtitles, CAPT,
    /// transcript): it had some and no other video shared its name ([`crate::moves_stem_files`],
    /// taken before the save). Undo and redo move them back and forth again. When it did not,
    /// they decide as a save would at that moment, so files made since are never taken from the
    /// video that shares the name (#417).
    pub stem_files_moved: bool,
}

/// Rename `from` to `to` with the comment, subtitle and transcript files that travel with it.
/// Refuses, changing nothing, when `to` or one of its sidecars exists: the same guard as the
/// save that made the rename, so Ctrl+Z / Ctrl+Y never replace another clip.
/// `keep_stem_files`: whether the files named without the extension stay where they are, as the
/// save that made the rename decided (#417).
fn rename_with_sidecars(from: &Path, to: &Path, keep_stem_files: bool) -> Result<(), UndoError> {
    if target_name_taken_with(from, to, keep_stem_files) {
        return Err(UndoError::NameTaken(to.to_path_buf()));
    }
    crate::recovery::rename_clip(from, to, false, keep_stem_files)?;
    Ok(())
}

impl<SD, ST> Undoable<SD, ST> for NavigateFileCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if self.path_after != self.path_before {
            if !self.path_after.exists() {
                return Err(UndoError::FileNotFound(self.path_after.clone()));
            }
            let keep = !self.stem_files_moved && crate::keeps_stem_files(&self.path_after);
            rename_with_sidecars(&self.path_after, &self.path_before, keep)?;
        }
        ctx.directory
            .rename_file(self.file_id, &self.path_before, &self.snapshot_before);
        ctx.directory
            .select_by_id(self.file_id)
            .ok_or(UndoError::FileNotFound(self.path_before.clone()))?;
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if self.path_before != self.path_after {
            if !self.path_before.exists() {
                return Err(UndoError::FileNotFound(self.path_before.clone()));
            }
            let keep = !self.stem_files_moved && crate::keeps_stem_files(&self.path_before);
            rename_with_sidecars(&self.path_before, &self.path_after, keep)?;
        }
        ctx.directory
            .rename_file(self.file_id, &self.path_after, &self.snapshot_after);
        ctx.directory
            .select_by_id(self.to_file_id)
            .ok_or(UndoError::FileNotFound(self.path_after.clone()))?;
        Ok(())
    }

    fn switches_file(&self) -> bool {
        true
    }
}
