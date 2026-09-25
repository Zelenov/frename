use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::TagId;

/// Records a single tag toggle (check ↔ uncheck).
/// Covers ToggleTag, ToggleSelectedTag (Space key), and UnselectTag (chip X click).
pub struct ToggleTagCommand {
    pub tag_id: TagId,
    /// Checked state immediately before the toggle.
    pub was_checked: bool,
}

impl<SD, ST> Undoable<SD, ST> for ToggleTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let current = ctx
            .tag_list
            .get_tag(self.tag_id)
            .ok_or(UndoError::TagNotFound(self.tag_id))?
            .is_checked();
        if current != self.was_checked {
            ctx.tag_list.toggle_by_id(self.tag_id);
        }
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let current = ctx
            .tag_list
            .get_tag(self.tag_id)
            .ok_or(UndoError::TagNotFound(self.tag_id))?
            .is_checked();
        if current == self.was_checked {
            ctx.tag_list.toggle_by_id(self.tag_id);
        }
        Ok(())
    }
}
