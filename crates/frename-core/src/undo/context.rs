use crate::{Directory, TagList};

/// Transient context built per undo/redo call. Holds mutable references to the two core
/// structures that commands may need to mutate. Never stored globally.
///
/// SD = store type used by Directory (AppStateStore + Clone).
/// ST = store type used by TagList (StoredTagStore + Clone).
pub struct UndoContext<'a, SD, ST> {
    pub directory: &'a mut Directory<SD>,
    pub tag_list: &'a mut TagList<ST>,
}
