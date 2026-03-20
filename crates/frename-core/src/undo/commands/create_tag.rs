use crate::db::{AppStateStore, StoredTagStore};
use crate::TagId;
use super::super::{UndoContext, UndoError};
use super::super::traits::Undoable;

/// Records a tag creation (CreateTag message — new tag added and saved to store).
/// Undo removes the tag; redo re-creates with the same UUID and original color.
pub struct CreateTagCommand {
    pub tag_id: TagId,
    pub tag_name: String,
    pub color_index: u8,
}

impl<SD, ST> Undoable<SD, ST> for CreateTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .remove_stored_tag_by_id(self.tag_id)
            .map_err(|_| UndoError::TagNotFound(self.tag_id))
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        if !ctx.tag_list.create_tag_with_id(self.tag_id, self.tag_name.clone()) {
            return Err(UndoError::TagNotFound(self.tag_id));
        }
        ctx.tag_list
            .save_tag_with_color(self.tag_id, self.color_index)
            .map_err(|_| UndoError::TagNotFound(self.tag_id))
    }
}
