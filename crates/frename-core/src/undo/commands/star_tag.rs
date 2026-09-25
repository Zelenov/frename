use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::TagId;

/// Records a star / unstar action (ToggleStar message).
pub struct StarTagCommand {
    pub tag_id: TagId,
    /// Star state immediately before the action.
    pub was_starred: bool,
}

impl<SD, ST> Undoable<SD, ST> for StarTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let result = if self.was_starred {
            ctx.tag_list.star_tag(self.tag_id)
        } else {
            ctx.tag_list.unstar_tag(self.tag_id)
        };
        result.map_err(|_| UndoError::TagNotFound(self.tag_id))
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        let result = if self.was_starred {
            ctx.tag_list.unstar_tag(self.tag_id)
        } else {
            ctx.tag_list.star_tag(self.tag_id)
        };
        result.map_err(|_| UndoError::TagNotFound(self.tag_id))
    }
}
