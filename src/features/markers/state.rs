//! State of the marker list that is not the markers themselves: the row being edited, the
//! open color picker, and what `F2` did last.

use std::time::{Duration, Instant};

use iced::widget::text_editor;

/// A second `F2` within this time after one that added a marker opens that marker's row, so
/// `F2 F2` marks a moment and names it; later, `F2` adds again.
pub const NAME_WINDOW: Duration = Duration::from_millis(1500);

/// The row open for editing.
#[derive(Debug)]
pub struct MarkerEdit {
    pub guid: String,
    /// Backing state of the row's multi-line comment editor.
    pub comment: text_editor::Content,
}

#[derive(Debug, Default)]
pub struct MarkersState {
    /// At most one row is open, so only one editor exists at a time.
    edit: Option<MarkerEdit>,
    /// The marker whose color picker is shown.
    color_picker: Option<String>,
    /// The marker `F2` added last and when; see [`NAME_WINDOW`].
    last_added: Option<(String, Instant)>,
    /// Row the list scrolled to last when following playback.
    followed: Option<usize>,
}

impl MarkersState {
    pub fn edit(&self) -> Option<&MarkerEdit> {
        self.edit.as_ref()
    }

    pub fn edit_mut(&mut self) -> Option<&mut MarkerEdit> {
        self.edit.as_mut()
    }

    pub fn is_editing(&self) -> bool {
        self.edit.is_some()
    }

    /// Open the row of `guid` with its comment in the editor.
    pub fn open(&mut self, guid: String, comment: &str) {
        self.edit = Some(MarkerEdit {
            guid,
            comment: text_editor::Content::with_text(comment),
        });
        self.color_picker = None;
        self.last_added = None;
    }

    pub fn close(&mut self) {
        self.edit = None;
    }

    pub fn color_picker(&self) -> Option<&str> {
        self.color_picker.as_deref()
    }

    pub fn toggle_color_picker(&mut self, guid: String) {
        self.color_picker = if self.color_picker.as_deref() == Some(guid.as_str()) {
            None
        } else {
            Some(guid)
        };
    }

    pub fn close_color_picker(&mut self) {
        self.color_picker = None;
    }

    pub fn added(&mut self, guid: String) {
        self.last_added = Some((guid, Instant::now()));
    }

    /// The marker `F2` added less than [`NAME_WINDOW`] ago, if nothing else happened since.
    pub fn take_recently_added(&mut self) -> Option<String> {
        let (guid, at) = self.last_added.take()?;
        (at.elapsed() < NAME_WINDOW).then_some(guid)
    }

    /// Something other than `F2` happened: a second `F2` adds again.
    pub fn forget_added(&mut self) {
        self.last_added = None;
    }

    /// Record the row the list follows; returns whether it changed.
    pub fn follow(&mut self, row: Option<usize>) -> bool {
        std::mem::replace(&mut self.followed, row) != row
    }

    /// Start over for another file.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
