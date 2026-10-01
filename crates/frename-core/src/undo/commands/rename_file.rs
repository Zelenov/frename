use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::FileSnapshot;

/// Records renaming the open file in place (double-click, context menu). Undo and redo put the
/// old or the new name back in the open file's tags, like any other edit of it; the rename on
/// disk happens with the next save, so a playing video is unloaded first, as for a tag change.
/// The selection stays.
pub struct RenameFileCommand {
    /// Tags and name before the rename.
    pub snapshot_before: FileSnapshot,
    /// Tags and name after it.
    pub snapshot_after: FileSnapshot,
}

impl<SD, ST> Undoable<SD, ST> for RenameFileCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .reinitialize_from_snapshot(self.snapshot_before.clone());
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .reinitialize_from_snapshot(self.snapshot_after.clone());
        Ok(())
    }
}
