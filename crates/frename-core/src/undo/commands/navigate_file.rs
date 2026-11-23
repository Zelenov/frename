use std::path::{Path, PathBuf};

use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::tags::{sidecar_name_too_long, target_name_taken};
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
/// Refuses, changing nothing, when `to` or one of its sidecars exists, or a sidecar's name under
/// `to` would be too long: the same guards as the save that made the rename, so Ctrl+Z / Ctrl+Y
/// never replace another clip nor leave a sidecar behind (#526).
fn rename_with_sidecars(from: &Path, to: &Path) -> Result<(), UndoError> {
    if target_name_taken(from, to) {
        return Err(UndoError::NameTaken(to.to_path_buf()));
    }
    if sidecar_name_too_long(from, to).is_some() {
        return Err(UndoError::NameTooLong(to.to_path_buf()));
    }
    crate::recovery::rename_clip(from, to, false)?;
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

    fn redo_renames_file(&self) -> bool {
        self.path_before != self.path_after
    }

    /// The save renamed the clip: its undo renames it back, but the clip's row keeps
    /// `snapshot_before`, the edits that rename applied.
    fn undo_leaves_unsaved(&self) -> Option<FileId> {
        (self.path_before != self.path_after).then_some(self.file_id)
    }
}
