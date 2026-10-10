use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::TagId;

/// Records checking several tags at once, as one step: the tags the AI suggests, added together.
/// Holds only the tags that were unchecked before; undo unchecks them, redo checks them again.
pub struct CheckTagsCommand {
    pub tag_ids: Vec<TagId>,
}

impl CheckTagsCommand {
    fn set_checked<SD, ST>(
        &self,
        ctx: &mut UndoContext<'_, SD, ST>,
        checked: bool,
    ) -> Result<(), UndoError>
    where
        SD: AppStateStore + Clone,
        ST: StoredTagStore + Clone,
    {
        for id in &self.tag_ids {
            let current = ctx
                .tag_list
                .get_tag(*id)
                .ok_or(UndoError::TagNotFound(*id))?
                .is_checked();
            if current != checked {
                ctx.tag_list.toggle_by_id(*id);
            }
        }
        Ok(())
    }
}

impl<SD, ST> Undoable<SD, ST> for CheckTagsCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set_checked(ctx, false)
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set_checked(ctx, true)
    }
}
