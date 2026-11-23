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
    /// Key in the file name order at deletion time (it differs from `sort_order` when the name's
    /// tag order differs from the grid's).
    pub selected_order: i64,
    /// For a tag that was in the clip's name: the checked tag just before it there (`None` when
    /// it was first). A list built again in between (an undone comment, the clip opened again)
    /// has other order keys; when they do not put the tag back after this one, the undo does
    /// (#636).
    pub name_after: Option<String>,
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
                self.selected_order,
            )
            .map_err(|_| UndoError::TagNotFound(self.tag_id))?;
        // The keys are exact when the list was not built again: only a wrong place is moved.
        if self.was_checked && ctx.tag_list.checked_name_before(self.tag_id) != self.name_after {
            ctx.tag_list.place_after_in_name(
                self.tag_id,
                self.name_after.as_deref(),
                !self.was_stored,
            );
        }
        Ok(())
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        // A list built again gives a tag that is only in the clip's name a new id (#627). Only
        // such a tag is followed, and only to another unsaved one: a folder tag of that name
        // saved meanwhile is not this step's to delete.
        if !self.was_stored {
            let id = ctx.tag_list.follow(self.tag_id, &self.tag_name);
            if ctx.tag_list.get_tag(id).is_some_and(|t| !t.is_stored()) {
                self.tag_id = id;
            }
        }
        ctx.tag_list
            .remove_stored_tag_by_id(self.tag_id)
            .map_err(|_| UndoError::TagNotFound(self.tag_id))
    }
}
