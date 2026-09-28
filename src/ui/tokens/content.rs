//! Colors of the content, not the chrome (`docs/design/design-system.md` §3.3, §13.8): the
//! video and what is drawn over it, the tag chips, the Premiere marker colors.

use iced::Color;

// Video.
/// Behind the picture: black bars read as part of the video, gray ones as a gap.
pub const VIDEO_BG: Color = Color::BLACK;
/// The timeline's and the volume's track (3 : 1 on the panel).
pub const VIDEO_TRACK: Color = Color::from_rgb8(0x3A, 0x40, 0x4A);
/// The in/out segment over the track.
pub const VIDEO_SEGMENT: Color = Color::from_rgba(0.949, 0.788, 0.298, 0.7);
/// The in/out segment's end lines.
pub const VIDEO_SEGMENT_EDGE: Color = Color::from_rgb8(0xF2, 0xC9, 0x4C);
/// The side list over the picture.
pub const OVERLAY_LIST: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.72);
/// The fullscreen caption's pill.
pub const OVERLAY_CAPTION: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.62);
/// Hover on the side list's rows: `HOVER` disappears on the dark overlay.
pub const OVERLAY_HOVER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.10);
/// A range being drawn on the timeline.
pub const NEW_RANGE: Color = Color::from_rgba(0.902, 0.910, 0.922, 0.35);
/// Under the fullscreen controls, so they read over the picture: from clear to this.
pub const FULLSCREEN_SHADE: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.6);
/// Idle range bands are drawn at this opacity, the active one fully.
pub const IDLE_BAND_ALPHA: f32 = 0.6;

// Tag chips: light colors with black text (8–14 : 1).
/// Chip labels.
pub const TAG_TEXT: Color = Color::BLACK;
/// Icons on a chip: star, action, grip.
pub const TAG_ICON: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.7);
/// Hover on a chip: `HOVER` does not show on light chips.
pub const CHIP_HOVER: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.22);
/// The checkbox on a chip: unchecked box and edge, checked box, tick.
pub const CHIP_CHECK_OFF: Color = Color::from_rgba(1.0, 1.0, 1.0, 0.7);
pub const CHIP_CHECK_EDGE: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.45);
pub const CHIP_CHECK_ON: Color = Color::from_rgba(0.0, 0.0, 0.0, 0.85);
pub const CHIP_CHECK_TICK: Color = Color::WHITE;

/// The 16 tag colors, in palette order; index 0 is also the gray of unknown tags.
pub const TAG_PALETTE: [Color; 16] = [
    Color::from_rgb(0.75, 0.75, 0.78), // light gray
    Color::from_rgb(0.45, 0.75, 0.95), // light blue
    Color::from_rgb(0.65, 0.85, 0.55), // soft green
    Color::from_rgb(0.90, 0.75, 0.45), // amber
    Color::from_rgb(0.75, 0.55, 0.90), // lavender
    Color::from_rgb(0.55, 0.85, 0.80), // teal
    Color::from_rgb(0.95, 0.65, 0.75), // pink
    Color::from_rgb(0.70, 0.80, 0.95), // pale blue
    Color::from_rgb(0.85, 0.70, 0.50), // tan
    Color::from_rgb(0.60, 0.90, 0.70), // mint
    Color::from_rgb(0.90, 0.60, 0.55), // salmon
    Color::from_rgb(0.55, 0.70, 0.95), // periwinkle
    Color::from_rgb(0.80, 0.65, 0.90), // violet
    Color::from_rgb(0.65, 0.90, 0.60), // lime
    Color::from_rgb(0.95, 0.80, 0.50), // gold
    Color::from_rgb(0.70, 0.85, 0.85), // cyan
];

// Premiere Pro's marker colors: they must match Premiere, not the theme.
pub const MARKER_GREEN: Color = Color::from_rgb(0.36, 0.76, 0.36);
pub const MARKER_RED: Color = Color::from_rgb(0.86, 0.22, 0.22);
pub const MARKER_ORANGE: Color = Color::from_rgb(0.93, 0.55, 0.15);
pub const MARKER_YELLOW: Color = Color::from_rgb(0.93, 0.85, 0.20);
pub const MARKER_WHITE: Color = Color::from_rgb(0.95, 0.95, 0.95);
pub const MARKER_BLUE: Color = Color::from_rgb(0.28, 0.48, 0.96);
pub const MARKER_CYAN: Color = Color::from_rgb(0.15, 0.78, 0.86);
pub const MARKER_LAVENDER: Color = Color::from_rgb(0.70, 0.58, 0.94);
pub const MARKER_MAGENTA: Color = Color::from_rgb(0.92, 0.28, 0.78);

#[cfg(test)]
mod tests {
    use super::super::color::tests::contrast;
    use super::*;

    #[test]
    fn chip_labels_are_readable_on_every_tag_color() {
        for color in TAG_PALETTE {
            assert!(contrast(TAG_TEXT, color) >= 7.0, "{color:?}");
        }
    }
}
