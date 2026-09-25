use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::FileSnapshot;

/// Records a paste-tags operation (replaces all checked tags on the current file).
pub struct PasteTagsCommand {
    /// Tag state before the paste.
    pub snapshot_before: FileSnapshot,
    /// Tag state after the paste.
    pub snapshot_after: FileSnapshot,
}

impl<SD, ST> Undoable<SD, ST> for PasteTagsCommand
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
