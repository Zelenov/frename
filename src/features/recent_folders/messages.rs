//! Messages of the recent folders list.

use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum Message {
    /// Open the dropdown, or close it (the ▾ button, `Ctrl+R`).
    Toggle,
    /// Close the dropdown (a click beside it).
    Close,
    /// Esc: leave the question about a missing folder first, then the dropdown.
    Escape,
    /// The pointer is over this row.
    Highlight(usize),
    /// An arrow key: the highlight moves by this many rows, wrapping round.
    Move(i32),
    /// A click on a folder's row.
    Choose(PathBuf),
    /// Enter: the highlighted row, as if clicked.
    ChooseHighlighted,
    /// The ✕ on a row, or Remove in the question about a missing folder: take it off the list.
    Remove(PathBuf),
    /// "Keep" in the question about a missing folder.
    Keep,
    /// Clear list.
    Clear,
    /// Which of the listed folders exist (internal): each folder with whether it does.
    Checked(Vec<(PathBuf, bool)>),
}
