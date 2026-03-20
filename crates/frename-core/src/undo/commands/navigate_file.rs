use std::path::PathBuf;

use crate::db::{AppStateStore, StoredTagStore};
use crate::FileSnapshot;
use super::super::{UndoContext, UndoError};
use super::super::traits::Undoable;

/// Records a file navigation (next / previous / indexed) together with its deferred save.
/// One Ctrl+Z reverses both the rename on disk and the file selection change.
pub struct NavigateFileCommand {
    pub from_index: usize,
    pub to_index: usize,
    /// Path of the file at from_index before the deferred rename.
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
        ctx.directory.update_file(&self.path_before, &self.snapshot_before);
        ctx.directory
            .select_index(self.from_index)
            .ok_or(UndoError::IndexOutOfRange(self.from_index))?;
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if self.path_before != self.path_after {
            if !self.path_before.exists() {
                return Err(UndoError::FileNotFound(self.path_before.clone()));
            }
            std::fs::rename(&self.path_before, &self.path_after)?;
        }
        ctx.directory.update_file(&self.path_after, &self.snapshot_after);
        ctx.directory
            .select_index(self.to_index)
            .ok_or(UndoError::IndexOutOfRange(self.to_index))?;
        Ok(())
    }
}
