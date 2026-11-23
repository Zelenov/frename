use super::super::traits::Undoable;
use super::super::{UndoContext, UndoError};
use crate::db::{AppStateStore, StoredTagStore};
use crate::{TagId, TagList};

/// Records a single tag toggle (check ↔ uncheck).
/// Covers ToggleTag, ToggleSelectedTag (Space key), and UnselectTag (chip X click).
pub struct ToggleTagCommand {
    pub tag_id: TagId,
    /// Checked state immediately before the toggle.
    pub was_checked: bool,
    /// For a tag that is not among the folder's tags (only in the clip's name): what puts it back
    /// when the tag list was built again without it (the clip opened again, or an undone comment
    /// or paste), so the step is still undone (#619). `None` for the folder's own tags.
    pub unsaved: Option<UnsavedTag>,
}

/// A tag that is only in the clip's name, as [`ToggleTagCommand`] puts it back.
#[derive(Debug, Clone)]
pub struct UnsavedTag {
    pub name: String,
    pub color_index: u8,
    /// The checked tag just before it in the name when the step was recorded: it goes back after
    /// that one (order keys do not carry over to a list built again).
    pub after: Option<String>,
}

impl ToggleTagCommand {
    /// The step for toggling `tag_id` in `tag_list`, recorded before or after the toggle (the
    /// tag's name and places do not change with it).
    pub fn new<S: StoredTagStore + Clone>(
        tag_list: &TagList<S>,
        tag_id: TagId,
        was_checked: bool,
    ) -> Self {
        let unsaved = tag_list
            .get_tag(tag_id)
            .filter(|tag| !tag.is_stored())
            .map(|tag| UnsavedTag {
                name: tag.tag().to_string(),
                color_index: tag.color_index(),
                after: tag_list.checked_name_before(tag_id),
            });
        Self {
            tag_id,
            was_checked,
            unsaved,
        }
    }

    /// Put the tag in the state `checked`.
    fn set<SD, ST>(&self, ctx: &mut UndoContext<'_, SD, ST>, checked: bool) -> Result<(), UndoError>
    where
        SD: AppStateStore + Clone,
        ST: StoredTagStore + Clone,
    {
        match ctx.tag_list.get_tag(self.tag_id).map(|t| t.is_checked()) {
            Some(current) => {
                if current != checked {
                    ctx.tag_list.restore_toggle_by_id(self.tag_id);
                }
                Ok(())
            }
            // A tag only in the name is not in a list built without it: unchecked already.
            None if !checked && self.unsaved.is_some() => Ok(()),
            None => {
                let Some(tag) = self.unsaved.as_ref().filter(|_| checked) else {
                    return Err(UndoError::TagNotFound(self.tag_id));
                };
                ctx.tag_list
                    .restore_unsaved_tag(
                        self.tag_id,
                        &tag.name,
                        tag.color_index,
                        tag.after.as_deref(),
                    )
                    .then_some(())
                    .ok_or(UndoError::TagNotFound(self.tag_id))
            }
        }
    }
}

impl<SD, ST> Undoable<SD, ST> for ToggleTagCommand
where
    SD: AppStateStore + Clone,
    ST: StoredTagStore + Clone,
{
    fn undo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set(ctx, self.was_checked)
    }

    fn redo(&mut self, ctx: &mut UndoContext<'_, SD, ST>) -> Result<(), UndoError> {
        self.set(ctx, !self.was_checked)
    }
}
