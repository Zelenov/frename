//! How the controls bar gives way on a narrow pane (design system §13.9, "Video pane"): it never
//! wraps and never shrinks a control; it gives up whole groups, least used first, into **More**.

use crate::ui::tokens::*;

/// What the controls bar shows at one pane width.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fold {
    /// The frame step buttons around play; otherwise in More.
    pub frame_step: bool,
    /// ↺ ↻ in the bar; otherwise in More.
    pub rotate: bool,
    /// The `00:10 / 00:30` readout; hidden otherwise (the timeline still shows the playhead).
    pub time: bool,
    /// The volume slider in the bar; otherwise only its icon, which opens it in More.
    pub volume_slider: bool,
    /// `camera` and `map-pin` in the bar; otherwise in More.
    pub mark: bool,
    /// The subtitle and marker list buttons in the bar; otherwise in More.
    pub lists: bool,
    /// The notice fits in the bar's free space; otherwise it floats over the picture.
    pub notice_in_bar: bool,
}

impl Fold {
    /// Everything in the bar.
    const ALL: Fold = Fold {
        frame_step: true,
        rotate: true,
        time: true,
        volume_slider: true,
        mark: true,
        lists: true,
        notice_in_bar: true,
    };

    /// Whether something went into More, so the bar needs its button.
    pub fn has_more(self) -> bool {
        !(self.frame_step && self.rotate && self.mark && self.lists && self.volume_slider)
    }

    /// The steps from everything shown to the least, each giving up one more group.
    fn steps() -> [Fold; 7] {
        let all = Fold::ALL;
        let no_rotate = Fold {
            rotate: false,
            ..all
        };
        let no_time = Fold {
            time: false,
            ..no_rotate
        };
        let no_slider = Fold {
            volume_slider: false,
            ..no_time
        };
        let no_frame_step = Fold {
            frame_step: false,
            ..no_slider
        };
        let no_mark = Fold {
            mark: false,
            ..no_frame_step
        };
        let no_lists = Fold {
            lists: false,
            ..no_mark
        };
        [
            all,
            no_rotate,
            no_time,
            no_slider,
            no_frame_step,
            no_mark,
            no_lists,
        ]
    }

    /// The bar's width with this fold, `list_buttons` being how many list buttons the clip has
    /// (the subtitle list only with subtitles).
    fn width(self, list_buttons: usize) -> f32 {
        let buttons = |n: usize| n as f32 * BAR_HEIGHT;
        let volume = if self.volume_slider {
            ICON_M + SPACE_S + VOLUME_WIDTH
        } else {
            buttons(1)
        };
        let views =
            buttons(usize::from(self.lists) * list_buttons + usize::from(self.has_more()) + 1);
        let groups = [
            Some(buttons(if self.frame_step { 5 } else { 3 })),
            Some(buttons(2)),
            self.mark.then(|| buttons(2)),
            self.rotate.then(|| buttons(2)),
            self.time.then_some(TIME_READOUT_WIDTH),
            Some(volume),
            Some(views),
        ];
        let shown: Vec<f32> = groups.into_iter().flatten().collect();
        // The free space between the groups is one more item of the row: one more gap.
        2.0 * SPACE_S + shown.iter().sum::<f32>() + shown.len() as f32 * SPACE_M
    }

    /// The most the bar can show in a pane `width` wide; the last step when nothing fits.
    pub fn for_width(width: f32, list_buttons: usize) -> Fold {
        let steps = Fold::steps();
        let fitting = steps
            .iter()
            .find(|fold| fold.width(list_buttons) <= width)
            .copied();
        let fold = fitting.unwrap_or(steps[steps.len() - 1]);
        Fold {
            notice_in_bar: width - fold.width(list_buttons) >= NOTICE_SLOT_MIN_WIDTH,
            ..fold
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_wide_pane_shows_everything() {
        let fold = Fold::for_width(2000.0, 2);
        assert_eq!(fold, Fold::ALL);
        assert!(!fold.has_more());
    }

    #[test]
    fn groups_give_way_least_used_first() {
        let full = Fold::ALL.width(2);
        let fold = Fold::for_width(full - 1.0, 2);
        assert!(!fold.rotate && fold.time && fold.has_more());
        let fold = Fold::for_width(Fold::steps()[2].width(2), 2);
        assert!(!fold.time && fold.volume_slider && fold.mark);
    }

    #[test]
    fn frame_steps_give_way_after_the_volume_slider_and_before_mark() {
        let fold = Fold::for_width(Fold::steps()[3].width(2), 2);
        assert!(!fold.volume_slider && fold.frame_step && fold.mark);
        let fold = Fold::for_width(Fold::steps()[4].width(2), 2);
        assert!(!fold.frame_step && fold.mark && fold.has_more());
        assert_eq!(
            Fold::steps()[3].width(2) - Fold::steps()[4].width(2),
            2.0 * BAR_HEIGHT
        );
    }

    #[test]
    fn the_narrowest_pane_keeps_transport_in_out_and_fullscreen() {
        let fold = Fold::for_width(VIDEO_MIN_WIDTH, 2);
        assert!(!fold.lists && !fold.mark && !fold.volume_slider && !fold.time);
        assert!(!fold.frame_step);
        assert!(fold.width(2) <= VIDEO_MIN_WIDTH, "{}", fold.width(2));
    }

    #[test]
    fn without_subtitles_the_bar_needs_one_button_less() {
        assert_eq!(Fold::ALL.width(2) - Fold::ALL.width(1), BAR_HEIGHT);
    }

    #[test]
    fn a_notice_floats_when_the_free_space_is_too_small() {
        let full = Fold::ALL.width(2);
        assert!(Fold::for_width(full + NOTICE_SLOT_MIN_WIDTH, 2).notice_in_bar);
        assert!(!Fold::for_width(full + NOTICE_SLOT_MIN_WIDTH - 1.0, 2).notice_in_bar);
    }
}
