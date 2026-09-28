//! How wide the tag grid's cells are and how many fit in a row (design system §13.5.2, §13.9):
//! every column is as wide as the widest chip of the whole tag list, at most `CHIP_MAX_WIDTH`,
//! and there are never fewer than two. The one estimate of a chip's width lives here; the grid
//! and the starred strip both use it, so their cells line up.

use std::borrow::Cow;

use frename_core::{StoredTagStore, TagList};

use crate::features::tag_panel::{GRID_CELL_INSET, GRID_GAP};
use crate::ui::tokens::*;

/// What a chip takes besides its label: its padding, the checkbox, the gaps, the marks.
const CHIP_CHROME: f32 = 2.0 * SPACE_S + CHECK_SIZE + 2.0 * SPACE_TIGHT + CHIP_MARKS_WIDTH;

/// Estimated width of a chip whose label has `chars` characters.
pub fn chip_width(chars: usize) -> f32 {
    CHIP_CHROME + CHIP_CHAR_WIDTH * chars as f32
}

/// `name` cut with "…" so its chip fits `chip_width`; unchanged when it fits.
pub fn fit_label(name: &str, chip_width: f32) -> Cow<'_, str> {
    let room = ((chip_width - CHIP_CHROME) / CHIP_CHAR_WIDTH)
        .floor()
        .max(1.0) as usize;
    if name.chars().count() <= room {
        return Cow::Borrowed(name);
    }
    let cut: String = name.chars().take(room.saturating_sub(1)).collect();
    Cow::Owned(format!("{}…", cut.trim_end()))
}

/// The grid's columns: how many, and how wide each cell is (the chip plus room for the ring).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Columns {
    pub count: u32,
    pub cell_width: f32,
}

impl Columns {
    /// The widest a chip in a cell gets.
    pub fn chip_width(self) -> f32 {
        (self.cell_width - 2.0 * GRID_CELL_INSET).max(0.0)
    }
}

/// The columns of an area `area_width` wide whose longest tag name has `widest_chars`
/// characters.
pub fn columns(area_width: f32, widest_chars: usize) -> Columns {
    let widest_cell = chip_width(widest_chars).min(CHIP_MAX_WIDTH) + 2.0 * GRID_CELL_INSET;
    let fit = ((area_width + GRID_GAP) / (widest_cell + GRID_GAP)).floor();
    if fit >= GRID_MIN_COLUMNS as f32 {
        return Columns {
            count: fit as u32,
            cell_width: widest_cell,
        };
    }
    let min = GRID_MIN_COLUMNS as f32;
    let shared = ((area_width - (min - 1.0) * GRID_GAP) / min).max(0.0);
    Columns {
        count: GRID_MIN_COLUMNS,
        cell_width: widest_cell.min(shared),
    }
}

/// The columns for `tag_list` in an area `area_width` wide, sized by its longest tag name, so
/// the grid does not reflow while a search narrows it.
pub fn columns_for<S: StoredTagStore + Clone>(tag_list: &TagList<S>, area_width: f32) -> Columns {
    columns(area_width, tag_list.longest_tag_name_chars())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_area_fits_many_columns_of_the_widest_chip() {
        let c = columns(1000.0, 6);
        assert!(c.count > 2, "{c:?}");
        assert_eq!(c.cell_width, chip_width(6) + 2.0 * GRID_CELL_INSET);
        let used = c.count as f32 * c.cell_width + (c.count - 1) as f32 * GRID_GAP;
        assert!(used <= 1000.0);
    }

    #[test]
    fn a_narrow_area_still_has_two_columns_that_fit_it() {
        let c = columns(280.0, 30);
        assert_eq!(c.count, 2);
        assert!(2.0 * c.cell_width + GRID_GAP <= 280.0);
    }

    #[test]
    fn a_long_name_is_capped_at_the_widest_chip() {
        let c = columns(2000.0, 200);
        assert_eq!(c.chip_width(), CHIP_MAX_WIDTH);
    }

    #[test]
    fn a_name_is_cut_only_when_it_does_not_fit() {
        assert_eq!(fit_label("pick", chip_width(4)), "pick");
        let long = "a very long tag name that goes on";
        let cut = fit_label(long, chip_width(10));
        assert!(cut.ends_with('…') && cut.chars().count() <= 10, "{cut}");
    }
}
