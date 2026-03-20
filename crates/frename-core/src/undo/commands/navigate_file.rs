use std::path::PathBuf;

use crate::db::{AppStateStore, StoredTagStore};
use crate::{FileId, FileSnapshot};
use super::super::{UndoContext, UndoError};
use super::super::traits::Undoable;

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
            std::fs::rename(&self.path_after, &self.path_before)?;
        }
        ctx.directory.rename_file(self.file_id, &self.path_before, &self.snapshot_before);
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
            std::fs::rename(&self.path_before, &self.path_after)?;
        }
        ctx.directory.rename_file(self.file_id, &self.path_after, &self.snapshot_after);
        ctx.directory
            .select_by_id(self.to_file_id)
            .ok_or(UndoError::FileNotFound(self.path_after.clone()))?;
        Ok(())
    }
}
