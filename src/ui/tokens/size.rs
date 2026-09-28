//! Sizes of controls, rows, icons, lines and corners (`docs/design/design-system.md` §5.2, §6,
//! §7). The sizes of the window's regions are in `region.rs`.

use super::space::*;
use super::typography::LINE_BODY;

// Controls.
/// Buttons, fields, dropdowns, segmented control, tag chips.
pub const CONTROL_HEIGHT: f32 = 28.0;
/// Toolbars (folder controls, video controls); their icon buttons are this square.
pub const BAR_HEIGHT: f32 = 32.0;
/// Icon buttons inside rows and chips.
pub const ICON_BUTTON_SMALL: f32 = 24.0;
/// The button bar at the bottom of a window or titled panel.
pub const BUTTON_BAR_HEIGHT: f32 = 56.0;
/// A navigation item, a one-line list row.
pub const ROW_HEIGHT: f32 = 32.0;
/// A two-line row: a name and a line of meta.
pub const ROW_TALL: f32 = 44.0;
/// A menu item.
pub const MENU_ITEM_HEIGHT: f32 = 28.0;
/// Above and below a line of text in a control, so the control is `CONTROL_HEIGHT` high.
pub const CONTROL_PADDING_Y: f32 = (CONTROL_HEIGHT - LINE_BODY) / 2.0;
/// Above and below a line of text in a row, so the row is `ROW_HEIGHT` high.
pub const ROW_PADDING_Y: f32 = (ROW_HEIGHT - LINE_BODY) / 2.0;
/// Checkbox and radio.
pub const CHECK_SIZE: f32 = 16.0;
/// Content under a checkbox or radio starts level with its label.
pub const CHOICE_INDENT: f32 = CHECK_SIZE + SPACE_S;
/// Field widths that show the expected input: a tag, a language, a model with its prices.
pub const FIELD_WIDTH_S: f32 = 160.0;
pub const FIELD_WIDTH_M: f32 = 240.0;
pub const FIELD_WIDTH_L: f32 = 300.0;
/// One checkbox of a wrapping grid of short choices (languages).
pub const CHOICE_WIDTH: f32 = 120.0;
/// The dot that says "an update is ready", and the recording dot of a held marker button.
pub const DOT_SIZE: f32 = 7.0;

// Lines.
/// A line: dividers, borders.
pub const LINE: f32 = 1.0;
/// The focus ring, the selection bar, the cursor ring of a chip.
pub const RING: f32 = 2.0;
/// Width of the notice's colored edge.
pub const NOTICE_BAR: f32 = 3.0;
/// Width of the selected item's bar.
pub const SELECTION_BAR: f32 = RING;
/// The scrollbar's rail and scroller, and the gap between it and the content.
pub const SCROLLBAR_WIDTH: f32 = 6.0;
pub const SCROLLBAR_GAP: f32 = 8.0;
/// The room a scroll area keeps on its right, always, so nothing jumps when it starts to scroll.
pub const SCROLL_GUTTER: f32 = SCROLLBAR_WIDTH + SCROLLBAR_GAP;
/// A scroller is never shorter than this.
pub const SCROLLER_MIN_LENGTH: f32 = 32.0;

// Popups.
/// Widest tooltip.
pub const TOOLTIP_MAX_WIDTH: f32 = 280.0;
/// Menus.
pub const MENU_MIN_WIDTH: f32 = 200.0;
pub const MENU_MAX_WIDTH: f32 = 360.0;
/// Modal dialogs.
pub const DIALOG_WIDTH: f32 = 420.0;
pub const DIALOG_MAX_WIDTH: f32 = 560.0;
/// The shadow under popups.
pub const SHADOW_OFFSET_Y: f32 = 8.0;
pub const SHADOW_BLUR: f32 = 24.0;
/// The shadow under a dragged chip: its lift.
pub const LIFT_OFFSET_Y: f32 = 1.0;
pub const LIFT_BLUR: f32 = 10.0;

// Icons: drawn at these sizes, never scaled from another.
/// Inside badges and row meta; ⓘ.
pub const ICON_S: f32 = 12.0;
/// Small marks inside chips and rows: star, chip action, status column.
pub const ICON_MARK: f32 = 14.0;
/// In buttons, menus and rows.
pub const ICON_M: f32 = 16.0;
/// The trash drop zone.
pub const ICON_DROP: f32 = 20.0;
/// Spinners; empty states of small panels.
pub const ICON_L: f32 = 24.0;
/// Empty states of panes and windows.
pub const ICON_XL: f32 = 48.0;

// Radii.
/// Controls, chips, rows, badges.
pub const RADIUS_S: f32 = 4.0;
/// Menus, tooltips, notices, cards.
pub const RADIUS_M: f32 = 6.0;
/// Dialogs, the fullscreen caption.
pub const RADIUS_L: f32 = 8.0;
/// Checkbox corners, key caps, scrollbars, tracks.
pub const RADIUS_CHECK: f32 = 3.0;
/// Dots and marker heads.
pub const RADIUS_FULL: f32 = 999.0;
