//! How wide the tag grid's cells are and how many fit in a row (design system §13.5.2, §13.9):
//! every column is as wide as the widest chip of the whole tag list, at most `CHIP_MAX_WIDTH`,
//! and there are never fewer than two. The one estimate of a chip's width lives here; the grid
//! and the starred strip both use it, so their cells line up.

use std::borrow::Cow;

use frename_core::{StoredTagStore, TagList};

use crate::features::tag_panel::{GRID_CELL_INSET, GRID_GAP};
use crate::ui::text;
use crate::ui::tokens::*;

/// What a chip takes besides its label: its padding, the checkbox, the gaps, the marks.
const CHIP_CHROME: f32 = 2.0 * SPACE_S + CHECK_SIZE + 2.0 * SPACE_TIGHT + CHIP_MARKS_WIDTH;

/// How wide `name` draws on a chip, checked (SemiBold) or not, measured with the bundled font:
/// a CJK character counts as the em it draws, not an average Latin letter (#528).
pub fn label_width(name: &str) -> f32 {
    text::strong_width(name, TEXT_BODY)
}

/// The width of a chip whose label is `label_width` px wide (see [`label_width`]).
pub fn chip_width(label_width: f32) -> f32 {
    CHIP_CHROME + label_width
}

/// `name` cut with "…" so its chip fits `chip_width`; unchanged when it fits. Measured like
/// [`label_width`], so a CJK name is cut where it really ends (#528).
pub fn fit_label(name: &str, chip_width: f32) -> Cow<'_, str> {
    // A hair over, so a chip sized by `chip_width(label_width(name))` keeps `name` whole
    // despite float rounding.
    let room = (chip_width - CHIP_CHROME).max(0.0) + 0.01;
    text::fit_strong_or_body(name, room, TEXT_BODY)
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

/// The columns of an area `area_width` wide whose widest tag name is `widest_label` px wide
/// (see [`label_width`]).
pub fn columns(area_width: f32, widest_label: f32) -> Columns {
    let widest_cell = chip_width(widest_label).min(CHIP_MAX_WIDTH) + 2.0 * GRID_CELL_INSET;
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

/// The columns for `tag_list` in an area `area_width` wide, sized by its widest tag name, so
/// the grid does not reflow while a search narrows it.
pub fn columns_for<S: StoredTagStore + Clone>(tag_list: &TagList<S>, area_width: f32) -> Columns {
    columns(area_width, tag_list.widest_tag_name(label_width))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_area_fits_many_columns_of_the_widest_chip() {
        let widest = label_width("golden");
        let c = columns(1000.0, widest);
        assert!(c.count > 2, "{c:?}");
        assert_eq!(c.cell_width, chip_width(widest) + 2.0 * GRID_CELL_INSET);
        let used = c.count as f32 * c.cell_width + (c.count - 1) as f32 * GRID_GAP;
        assert!(used <= 1000.0);
    }

    #[test]
    fn a_narrow_area_still_has_two_columns_that_fit_it() {
        let c = columns(280.0, label_width("a tag name of thirty letters!!"));
        assert_eq!(c.count, 2);
        assert!(2.0 * c.cell_width + GRID_GAP <= 280.0);
    }

    #[test]
    fn a_long_name_is_capped_at_the_widest_chip() {
        let c = columns(2000.0, label_width(&"a".repeat(200)));
        assert_eq!(c.chip_width(), CHIP_MAX_WIDTH);
    }

    #[test]
    fn a_name_is_cut_only_when_it_does_not_fit() {
        assert_eq!(fit_label("pick", chip_width(label_width("pick"))), "pick");
        let long = "a very long tag name that goes on";
        let room = chip_width(label_width("a very lon"));
        let cut = fit_label(long, room);
        assert!(cut.ends_with('…'), "{cut}");
        assert!(label_width(&cut) <= room - CHIP_CHROME + 0.1, "{cut}");
    }

    /// A checked label is SemiBold, wider than Regular most of all in narrow letters: its chip
    /// has room for that, so checking the widest tag never pushes its marks.
    #[test]
    fn a_chip_has_room_for_its_label_in_semibold() {
        for name in ["still", "fill-light", "lit"] {
            assert!(
                label_width(name) > text::text_width(name, TEXT_BODY),
                "{name}"
            );
            assert_eq!(fit_label(name, chip_width(label_width(name))), name);
        }
    }

    /// #528: a CJK name draws about twice as wide as a Latin name of as many characters. Its
    /// chip is as wide as it draws, and in a narrower chip it is cut so it ends inside it.
    #[test]
    fn a_cjk_name_gets_the_room_it_draws_in() {
        let cjk = "記者会見の様子";
        assert!(label_width(cjk) >= 7.0 * TEXT_BODY, "{}", label_width(cjk));
        assert!(label_width(cjk) > 1.5 * label_width("abcdefg"));
        let c = columns(1000.0, label_width(cjk));
        assert_eq!(fit_label(cjk, c.chip_width()), cjk, "fits its own column");

        let narrow = chip_width(label_width("abcdefg"));
        let cut = fit_label(cjk, narrow);
        assert!(cut.ends_with('…'), "{cut}");
        assert!(label_width(&cut) <= narrow - CHIP_CHROME + 0.1, "{cut}");
    }
}
