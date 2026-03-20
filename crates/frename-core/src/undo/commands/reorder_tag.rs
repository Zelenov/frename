use crate::db::{AppStateStore, StoredTagStore};
use crate::TagId;
use super::super::{UndoContext, UndoError};
use super::super::traits::Undoable;

/// Records a tag reorder in the file-name panel (drag-drop).
pub struct ReorderTagCommand {
    pub moved_id: TagId,
    /// Position in the checked-tag list before the drag.
    pub from_index: usize,
    /// Position in the checked-tag list after the drag.
    pub to_index: usize,
}

impl<SD, ST> Undoable<SD, ST> for ReorderTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if ctx.tag_list.checked_index_of(self.moved_id).is_none() {
            return Err(UndoError::TagNotFound(self.moved_id));
        }
        ctx.tag_list.reorder_tag_to_index(self.moved_id, self.from_index);
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if ctx.tag_list.checked_index_of(self.moved_id).is_none() {
            return Err(UndoError::TagNotFound(self.moved_id));
        }
        ctx.tag_list.reorder_tag_to_index(self.moved_id, self.to_index);
        Ok(())
    }
}
