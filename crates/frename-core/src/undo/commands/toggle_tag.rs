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
    /// For a tag that is not among the folder's tags (only in the clip's name): what finds it, or
    /// puts it back, once the tag list was built again (the clip opened again, or an undone
    /// comment or paste), which gives such a tag a new id or leaves it out (#619). `None` for the
    /// folder's own tags.
    pub unsaved: Option<UnsavedTag>,
}

/// A tag that is only in the clip's name, as [`ToggleTagCommand`] finds or puts it back.
#[derive(Debug, Clone)]
pub struct UnsavedTag {
    pub name: String,
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
                after: tag_list.checked_name_before(tag_id),
            });
        Self {
            tag_id,
            was_checked,
            unsaved,
        }
    }

    /// Put the tag in the state `checked`.
    fn set<SD, ST>(
        &mut self,
        ctx: &mut UndoContext<'_, SD, ST>,
        checked: bool,
    ) -> Result<(), UndoError>
    where
        SD: AppStateStore + Clone,
        ST: StoredTagStore + Clone,
    {
        // A list built again has the name-only tag under a new id: follow it.
        if ctx.tag_list.get_tag(self.tag_id).is_none() {
            if let Some(id) = self
                .unsaved
                .as_ref()
                .and_then(|tag| ctx.tag_list.unsaved_tag_id_named(&tag.name))
            {
                self.tag_id = id;
            }
        }
        let current = ctx.tag_list.get_tag(self.tag_id).map(|t| t.is_checked());
        match (current, &self.unsaved, checked) {
            (Some(current), _, _) => {
                if current != checked {
                    ctx.tag_list.restore_toggle_by_id(self.tag_id);
                }
                Ok(())
            }
            (None, None, _) => Err(UndoError::TagNotFound(self.tag_id)),
            // Left out of a list built without it: unchecked already.
            (None, Some(_), false) => Ok(()),
            (None, Some(tag), true) => {
                let after = tag.after.clone();
                let name = tag.name.clone();
                ctx.tag_list
                    .restore_unsaved_tag(self.tag_id, &name, after.as_deref())
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
