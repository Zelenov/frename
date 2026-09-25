use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::TagId;

/// Records a tag deletion (stored or snapshot-only).
/// Undo re-inserts the tag at its original position; redo removes it again.
pub struct DeleteTagCommand {
    pub tag_id: TagId,
    pub tag_name: String,
    pub color_index: u8,
    pub was_stored: bool,
    pub was_starred: bool,
    pub was_checked: bool,
    /// Display sort_order at deletion time (used to re-insert at the same position).
    pub sort_order: i64,
}

impl<SD, ST> Undoable<SD, ST> for DeleteTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .restore_deleted_tag(
                self.tag_id,
                &self.tag_name,
                self.color_index,
                self.was_stored,
                self.was_starred,
                self.was_checked,
                self.sort_order,
            )
            .map_err(|_| UndoError::TagNotFound(self.tag_id))
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .remove_stored_tag_by_id(self.tag_id)
            .map_err(|_| UndoError::TagNotFound(self.tag_id))
    }
}
