//! The two groups of the tag grid (`docs/design/design-system.md` §13.5.2): the tags in this
//! file's name that are not in the folder's tags come first, under a caption, then the folder's
//! tags under theirs. Each group starts on a row of its own, so a tag's place on screen is not
//! `index / columns` any more: this is the one place that knows where each tag is, for the view,
//! the arrow keys and scrolling the cursor into view.

use frename_core::{StoredTagStore, TagList};

use crate::features::tag_panel::GRID_ROW_STRIDE;
use crate::ui::tokens::*;

/// How the filtered tags lie in the grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shape {
    /// The tags not in the folder's tags, which come first in the filtered order.
    pub unsaved: usize,
    /// Every filtered tag.
    pub total: usize,
    pub columns: usize,
}

impl Shape {
    /// The shape of `tag_list`'s filtered tags in `columns` columns.
    pub fn of<S: StoredTagStore + Clone>(tag_list: &TagList<S>, columns: usize) -> Self {
        let ids = tag_list.filtered_display_tag_ids();
        let unsaved = ids
            .iter()
            .take_while(|&&id| tag_list.get_tag(id).is_some_and(|t| !t.is_stored()))
            .count();
        Self {
            unsaved,
            total: ids.len(),
            columns: columns.max(1),
        }
    }

    /// Whether the groups have captions: only when there are unsaved tags; otherwise the grid
    /// is one group and needs none.
    pub fn captioned(&self) -> bool {
        self.unsaved > 0
    }

    /// The groups as (first index, length), the empty ones left out.
    pub fn groups(&self) -> impl Iterator<Item = (usize, usize)> {
        [(0, self.unsaved), (self.unsaved, self.total - self.unsaved)]
            .into_iter()
            .filter(|&(_, len)| len > 0)
    }

    fn rows_of(&self, len: usize) -> usize {
        len.div_ceil(self.columns)
    }

    /// Rows of the first group (0 without unsaved tags).
    fn unsaved_rows(&self) -> usize {
        self.rows_of(self.unsaved)
    }

    /// Every row of the grid.
    pub fn rows(&self) -> usize {
        self.unsaved_rows() + self.rows_of(self.total - self.unsaved)
    }

    /// The row and column of tag `index`.
    pub fn position(&self, index: usize) -> (usize, usize) {
        if index < self.unsaved {
            (index / self.columns, index % self.columns)
        } else {
            let j = index - self.unsaved;
            (self.unsaved_rows() + j / self.columns, j % self.columns)
        }
    }

    /// The tags of `row`: its first index and how many.
    fn row_span(&self, row: usize) -> (usize, usize) {
        let unsaved_rows = self.unsaved_rows();
        let (start, group_end, first_row) = if row < unsaved_rows {
            (0, self.unsaved, 0)
        } else {
            (self.unsaved, self.total, unsaved_rows)
        };
        let first = start + (row - first_row) * self.columns;
        (first, self.columns.min(group_end.saturating_sub(first)))
    }

    /// The tag in `row` nearest to `column` (a shorter row has its last tag there).
    pub fn index_at(&self, row: usize, column: usize) -> Option<usize> {
        let (first, len) = self.row_span(row);
        (len > 0).then(|| first + column.min(len - 1))
    }

    /// One row up from tag `index`, the top row wrapping to the last.
    pub fn up(&self, index: usize) -> Option<usize> {
        let (row, column) = self.position(index);
        let rows = self.rows();
        let target = if row == 0 {
            rows.checked_sub(1)?
        } else {
            row - 1
        };
        self.index_at(target, column)
    }

    /// One row down from tag `index`, the last row wrapping to the top.
    pub fn down(&self, index: usize) -> Option<usize> {
        let (row, column) = self.position(index);
        let rows = self.rows();
        let target = if row + 1 >= rows { 0 } else { row + 1 };
        self.index_at(target, column)
    }

    /// The top of `row` within the scroll area: the rows above, and each group's caption and
    /// the gap between the groups when there are captions.
    pub fn row_top(&self, row: usize) -> f32 {
        let mut top = row as f32 * GRID_ROW_STRIDE;
        if self.captioned() {
            top += GRID_CAPTION_HEIGHT;
            if row >= self.unsaved_rows() {
                // The last row of the first group has no gap under it, the group gap instead.
                top += GRID_GROUP_GAP - GRID_GAP + GRID_CAPTION_HEIGHT;
            }
        }
        top
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shape(unsaved: usize, total: usize, columns: usize) -> Shape {
        Shape {
            unsaved,
            total,
            columns,
        }
    }

    #[test]
    fn the_folder_tags_start_on_a_row_of_their_own() {
        let grid = shape(3, 8, 2);
        assert_eq!(grid.position(2), (1, 0), "the last unsaved tag");
        assert_eq!(grid.position(3), (2, 0), "the first folder tag: a new row");
        assert_eq!(grid.rows(), 5);
    }

    #[test]
    fn without_unsaved_tags_the_grid_is_one_group_without_captions() {
        let grid = shape(0, 5, 2);
        assert!(!grid.captioned());
        assert_eq!(grid.position(4), (2, 0));
        assert_eq!(grid.row_top(1), GRID_ROW_STRIDE);
        assert_eq!(grid.groups().count(), 1);
    }

    #[test]
    fn up_and_down_move_by_rows_across_the_groups_and_wrap() {
        let grid = shape(3, 8, 2);
        assert_eq!(grid.down(1), Some(2), "into the short row: its only tag");
        assert_eq!(
            grid.down(2),
            Some(3),
            "from the unsaved into the folder's tags"
        );
        assert_eq!(grid.up(3), Some(2));
        assert_eq!(grid.up(0), Some(7), "the top wraps to the last row");
        assert_eq!(grid.down(7), Some(0), "the last row wraps to the top");
    }

    #[test]
    fn the_folder_group_sits_below_both_captions_and_the_gap() {
        let grid = shape(3, 8, 2);
        assert_eq!(grid.row_top(0), GRID_CAPTION_HEIGHT);
        // Two rows of unsaved tags (the second without its gap), the gap, the second caption.
        assert_eq!(
            grid.row_top(2),
            2.0 * GRID_ROW_STRIDE - GRID_GAP + GRID_GROUP_GAP + 2.0 * GRID_CAPTION_HEIGHT
        );
    }
}
