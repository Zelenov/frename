use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::TagId;

/// Records saving a snapshot-only tag to the store (SaveTag message).
/// Undo marks the tag as unsaved again; redo re-saves it.
pub struct SaveTagCommand {
    pub tag_id: TagId,
    /// Color index assigned when the tag was first saved (preserved for redo).
    pub color_index: u8,
}

impl<SD, ST> Undoable<SD, ST> for SaveTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .unsave_tag(self.tag_id)
            .map_err(|_| UndoError::TagNotFound(self.tag_id))
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .save_tag_with_color(self.tag_id, self.color_index)
            .map_err(|_| UndoError::TagNotFound(self.tag_id))
    }
}
