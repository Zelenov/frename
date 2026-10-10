//! How the toolbar under the file list gives way on a narrow list (design system §13.9, "File
//! list"): it never wraps and never shrinks a button; the gaps tighten first, then whole buttons,
//! least used first, go into **More**.

use crate::ui::tokens::*;

/// What the toolbar shows at one file list width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fold {
    /// The gaps between groups are 4 px (otherwise 12).
    pub tight: bool,
    /// `locate-fixed` in the bar; otherwise in More.
    pub locate: bool,
    /// The batch toggle in the bar; otherwise in More.
    pub batch: bool,
    /// The recent folders `chevron-down` in the bar; otherwise in More (`Ctrl`+`R` still opens it).
    pub recent: bool,
}

impl Fold {
    /// Everything in the bar.
    const ALL: Fold = Fold {
        tight: false,
        locate: true,
        batch: true,
        recent: true,
    };

    /// Whether something went into More, so the bar needs its button.
    pub fn has_more(self) -> bool {
        !(self.locate && self.batch && self.recent)
    }

    /// The steps from everything shown to the least, each giving up one more thing.
    fn steps() -> [Fold; 4] {
        let all = Fold::ALL;
        let tight = Fold { tight: true, ..all };
        // Locate and batch go together: they are the least used (§13.9).
        let no_locate_batch = Fold {
            locate: false,
            batch: false,
            ..tight
        };
        let no_recent = Fold {
            recent: false,
            ..no_locate_batch
        };
        [all, tight, no_locate_batch, no_recent]
    }

    /// The toolbar's width with this fold.
    fn width(self) -> f32 {
        let buttons = |n: usize| n as f32 * BAR_HEIGHT;
        let gap = if self.tight { SPACE_XS } else { SPACE_M };
        let files = buttons(2 + usize::from(self.locate));
        let opening = buttons(1 + usize::from(self.recent));
        let app = buttons(1 + usize::from(self.batch) + usize::from(self.has_more()));
        // The free space between the groups is one more item of the row: one more gap.
        2.0 * SPACE_S + files + opening + app + 3.0 * gap
    }

    /// The most the toolbar can show in a file list `width` wide; the last step when nothing fits.
    pub fn for_width(width: f32) -> Fold {
        let steps = Fold::steps();
        steps
            .iter()
            .find(|fold| fold.width() <= width)
            .copied()
            .unwrap_or(steps[steps.len() - 1])
    }

    /// The gap between the groups.
    pub fn gap(self) -> f32 {
        if self.tight {
            SPACE_XS
        } else {
            SPACE_M
        }
    }

    /// What went into More, in the order the menu lists it.
    pub fn in_more(self) -> Vec<Folded> {
        let mut folded = Vec::new();
        if !self.locate {
            folded.push(Folded::Locate);
        }
        if !self.recent {
            folded.push(Folded::Recent);
        }
        if !self.batch {
            folded.push(Folded::Batch);
        }
        folded
    }
}

/// A button that can go into More.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Folded {
    Locate,
    Recent,
    Batch,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_list_shows_everything() {
        let fold = Fold::for_width(FILE_LIST_MAX_WIDTH);
        assert_eq!(fold, Fold::ALL);
        assert!(!fold.has_more());
    }

    /// The widths of each step, for the table in the design system (§13.9).
    #[test]
    fn the_fold_widths_match_the_design_system() {
        let widths: Vec<f32> = Fold::steps().iter().map(|fold| fold.width()).collect();
        assert_eq!(widths, [276.0, 252.0, 220.0, 188.0]);
    }

    #[test]
    fn buttons_give_way_least_used_first() {
        let fold = Fold::for_width(275.0);
        assert!(fold.tight && fold.locate && fold.batch && fold.recent && !fold.has_more());
        let fold = Fold::for_width(251.0);
        assert!(!fold.locate && !fold.batch && fold.recent && fold.has_more());
        let fold = Fold::for_width(219.0);
        assert!(!fold.recent && fold.has_more());
    }

    /// The bug (#228): at 200 the toolbar was 276 wide. At every width the list can have, the
    /// bar fits and every one of the seven buttons is in the bar or in More.
    #[test]
    fn every_button_is_reachable_and_the_bar_fits_at_every_list_width() {
        let mut width = FILE_LIST_MIN_WIDTH;
        while width <= FILE_LIST_MAX_WIDTH {
            let fold = Fold::for_width(width);
            assert!(
                fold.width() <= width,
                "{width}: the bar is {}",
                fold.width()
            );
            // Each of the three that can fold is in the bar or in More, never both or neither;
            // More has its button exactly when something is in it.
            for (in_bar, item) in [
                (fold.locate, Folded::Locate),
                (fold.recent, Folded::Recent),
                (fold.batch, Folded::Batch),
            ] {
                assert_eq!(in_bar, !fold.in_more().contains(&item), "{width}");
            }
            assert_eq!(fold.has_more(), !fold.in_more().is_empty(), "{width}");
            width += 1.0;
        }
    }

    #[test]
    fn the_minimum_list_keeps_navigation_open_and_settings() {
        let fold = Fold::for_width(FILE_LIST_MIN_WIDTH);
        assert!(fold.tight && !fold.locate && !fold.batch && !fold.recent);
        assert!(fold.width() <= FILE_LIST_MIN_WIDTH);
    }
}
