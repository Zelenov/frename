use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::tags::TagOrderState;

/// Records Sync up, Sync down or the lock: the tag orders and the lock before and after.
pub struct SyncTagOrderCommand {
    pub before: TagOrderState,
    pub after: TagOrderState,
}

impl<SD, ST> Undoable<SD, ST> for SyncTagOrderCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .restore_order_state(self.before.clone())
            .map_err(|e| UndoError::Io(std::io::Error::other(e.to_string())))
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        ctx.tag_list
            .restore_order_state(self.after.clone())
            .map_err(|e| UndoError::Io(std::io::Error::other(e.to_string())))
    }
}
