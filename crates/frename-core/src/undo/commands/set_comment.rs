use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::FileSnapshot;

/// Records editing the open file's comment: one step per focus session of the comment box.
/// Writing a comment can check the commented tag and clearing it uncheck it, so the whole tag
/// state is kept, not only the text.
pub struct SetCommentCommand {
    /// Tags and comment when the box got the focus.
    pub snapshot_before: FileSnapshot,
    /// Tags and comment when it lost it.
    pub snapshot_after: FileSnapshot,
}

impl<SD, ST> Undoable<SD, ST> for SetCommentCommand
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
