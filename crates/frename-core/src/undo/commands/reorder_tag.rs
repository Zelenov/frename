use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::{TagId, TagList};

/// Records a tag reorder in the file-name panel (drag-drop).
pub struct ReorderTagCommand {
    pub moved_id: TagId,
    /// The moved tag's name: a list built again gives a tag that is only in the clip's name a new
    /// id, and the step finds it by this name (#627). `None` when not recorded.
    pub name: Option<String>,
    /// Position in the checked-tag list before the drag.
    pub from_index: usize,
    /// Position in the checked-tag list after the drag.
    pub to_index: usize,
}

impl ReorderTagCommand {
    /// The step for moving `moved_id` in `tag_list`'s file name order from `from_index` to
    /// `to_index`.
    pub fn new<S: StoredTagStore + Clone>(
        tag_list: &TagList<S>,
        moved_id: TagId,
        from_index: usize,
        to_index: usize,
    ) -> Self {
        Self {
            moved_id,
            name: tag_list.get_tag(moved_id).map(|t| t.tag().to_string()),
            from_index,
            to_index,
        }
    }

    /// Put the moved tag at `index` among the checked tags, following it by name when the list
    /// was built again.
    fn move_to<SD, ST>(
        &mut self,
        ctx: &mut UndoContext<'_, SD, ST>,
        index: usize,
    ) -> Result<(), UndoError>
    where
        SD: AppStateStore + Clone,
        ST: StoredTagStore + Clone,
    {
        if ctx.tag_list.checked_index_of(self.moved_id).is_none() {
            if let Some(id) = self
                .name
                .as_deref()
                .and_then(|name| ctx.tag_list.tag_id_named_exactly(name))
            {
                self.moved_id = id;
            }
        }
        if ctx.tag_list.checked_index_of(self.moved_id).is_none() {
            return Err(UndoError::TagNotFound(self.moved_id));
        }
        ctx.tag_list.reorder_tag_to_index(self.moved_id, index);
        Ok(())
    }
}

impl<SD, ST> Undoable<SD, ST> for ReorderTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.move_to(ctx, self.from_index)
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.move_to(ctx, self.to_index)
    }
}
