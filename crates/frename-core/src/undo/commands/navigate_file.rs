use std::path::{Path, PathBuf};

use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::tags::target_name_taken;
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
}

/// Rename `from` to `to` with the comment, subtitle and transcript files that travel with it.
/// Refuses, changing nothing, when `to` or one of its sidecars exists: the same guard as the
/// save that made the rename, so Ctrl+Z / Ctrl+Y never replace another clip.
fn rename_with_sidecars(from: &Path, to: &Path) -> Result<(), UndoError> {
    if target_name_taken(from, to) {
        return Err(UndoError::NameTaken(to.to_path_buf()));
    }
    std::fs::rename(from, to)?;
    crate::tags::rename_sidecars(from, to);
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
            rename_with_sidecars(&self.path_after, &self.path_before)?;
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
            rename_with_sidecars(&self.path_before, &self.path_after)?;
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
