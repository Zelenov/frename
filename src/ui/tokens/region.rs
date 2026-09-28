//! Sizes of the windows and of the regions of the main window (`docs/design/design-system.md`
//! §13, §13.9, §14.1): their minimums, defaults and maximums, and the fixed heights of their bars.

use super::size::*;
use super::space::*;

// The main window (§13.9): the minimum fits a 1280×680 work area with room to spare.
pub const WINDOW_MIN_WIDTH: f32 = 900.0;
pub const WINDOW_MIN_HEIGHT: f32 = 560.0;
pub const WINDOW_WIDTH: f32 = 1440.0;
pub const WINDOW_HEIGHT: f32 = 800.0;

// Columns (§13.1).
pub const VIDEO_MIN_WIDTH: f32 = 320.0;
pub const FILE_LIST_MIN_WIDTH: f32 = 200.0;
pub const FILE_LIST_MAX_WIDTH: f32 = 720.0;
pub const TAGS_MIN_WIDTH: f32 = 320.0;
/// The column widths a window starts with: a wide one (1440 or more) and a narrower one.
pub const VIDEO_WIDTH_WIDE: f32 = 600.0;
pub const FILE_LIST_WIDTH_WIDE: f32 = 360.0;
pub const VIDEO_WIDTH: f32 = 440.0;
pub const FILE_LIST_WIDTH: f32 = 300.0;
/// A splitter's hit area; it draws a `LINE` in its middle.
pub const SPLITTER_HIT: f32 = 12.0;
/// The comment's height handle: its hit area, and the line it draws.
pub const HANDLE_HIT: f32 = 8.0;
pub const HANDLE_LINE: f32 = RING;

// Video pane (§13.3).
/// The strip of the windowed subtitle, so the picture never jumps between cues.
pub const SUBTITLE_STRIP_HEIGHT: f32 = 48.0;
/// The timeline row with no or one lane of ranges; each more lane adds `LANE_PITCH`.
pub const TIMELINE_HEIGHT: f32 = 24.0;
/// The side list over the picture: its widest, and the pane width from which it covers only part.
pub const SIDE_LIST_MAX_WIDTH: f32 = 360.0;
pub const SIDE_LIST_PARTIAL_FROM: f32 = 600.0;
/// Header of the side list.
pub const SIDE_LIST_HEADER_HEIGHT: f32 = 36.0;
/// The fullscreen caption: widest, and its distance above the controls.
pub const CAPTION_MAX_WIDTH: f32 = 1100.0;
pub const CAPTION_LIFT: f32 = 48.0;
/// The fullscreen caption's padding.
pub const CAPTION_PADDING_Y: f32 = SPACE_S;
pub const CAPTION_PADDING_X: f32 = 18.0;
/// The volume slider.
pub const VOLUME_WIDTH: f32 = 72.0;
/// The time readout: a fixed width so it never moves.
pub const TIME_READOUT_WIDTH: f32 = 104.0;
/// The notices' slot in the controls bar needs this much, otherwise they float over the picture.
pub const NOTICE_SLOT_MIN_WIDTH: f32 = 120.0;
/// A marker row's color dot.
pub const MARKER_DOT: f32 = 14.0;

// File list (§13.4).
/// The search bar over a list: a 28 px field with 6 px around it.
pub const SEARCH_BAR_HEIGHT: f32 = CONTROL_HEIGHT + 2.0 * SPACE_TIGHT;
/// A file row: the name line and the comment line.
pub const FILE_ROW_HEIGHT: f32 = 52.0;
/// The check column in batch mode, and the status column (SRT, lock).
pub const CHECK_COLUMN: f32 = 28.0;
pub const STATUS_COLUMN: f32 = 32.0;
/// The line over the list that says why it is locked.
pub const LOCK_LINE_HEIGHT: f32 = CONTROL_HEIGHT;
/// A file name keeps at least this much beside its chips.
pub const NAME_MIN_WIDTH: f32 = 80.0;

// Tags area (§13.5).
pub const CHIP_HEIGHT: f32 = CONTROL_HEIGHT;
pub const CHIP_MINI_HEIGHT: f32 = 20.0;
/// A grid column is at most this wide; longer names are cut inside the chip.
pub const CHIP_MAX_WIDTH: f32 = 240.0;
/// The order strip between the grid and the file name card.
pub const ORDER_STRIP_HEIGHT: f32 = BAR_HEIGHT;
/// The trash at the end of the file name card.
pub const TRASH_SIDE: f32 = 36.0;
/// The comment box: its lowest height.
pub const COMMENT_MIN_HEIGHT: f32 = 48.0;
/// The comment box: its height until it is resized, and the tallest the handle makes it
/// (taller than that, "Expand" is the way).
pub const COMMENT_HEIGHT: f32 = 80.0;
pub const COMMENT_MAX_HEIGHT: f32 = 600.0;
/// A dragged chip in the file name card floats this far above its line.
pub const CHIP_DRAG_LIFT: f32 = SPACE_TIGHT;
/// The width a chip of the file name card is taken to have when the drop place is worked out.
pub const CHIP_DROP_ESTIMATE_WIDTH: f32 = 64.0;
/// A generous average width of one character of a chip's 13-px label, to size grid columns.
pub const CHIP_CHAR_WIDTH: f32 = 7.5;
/// The chip's marks at its right end: the star and the action, 14 px each, 4 px apart.
pub const CHIP_MARKS_WIDTH: f32 = 2.0 * ICON_MARK + SPACE_XS;
/// Between the chip and the cursor ring drawn outside it.
pub const CHIP_RING_GAP: f32 = LINE;
/// Between the cells of the tag grid, across and down.
pub const GRID_GAP: f32 = SPACE_XS;
/// A grid cell: the chip and room around it for the cursor's ring, drawn outside the chip.
pub const GRID_CELL_INSET: f32 = RING + CHIP_RING_GAP;
pub const GRID_CELL_HEIGHT: f32 = CHIP_HEIGHT + 2.0 * GRID_CELL_INSET;
/// From the top of one grid row to the top of the next.
pub const GRID_ROW_STRIDE: f32 = GRID_CELL_HEIGHT + GRID_GAP;
/// Columns of the tag grid: never fewer, however narrow the area.
pub const GRID_MIN_COLUMNS: u32 = 2;

// Batch panel (§13.6).
pub const BATCH_HEADER_HEIGHT: f32 = 48.0;
pub const ACTION_LIST_WIDTH: f32 = 232.0;
/// The panel's width on entering batch mode: the whole action list and a page.
pub const BATCH_PANEL_WIDTH: f32 = 600.0;
/// The failed files' table scrolls after this many rows.
pub const FAILED_ROWS_SHOWN: f32 = 8.0;

/// The widest a page of settings or options gets: longer lines are hard to read.
pub const PAGE_MAX_WIDTH: f32 = 640.0;

// The Settings window (§14.1): its size at first and the smallest it gets.
pub const SETTINGS_WINDOW_WIDTH: f32 = 800.0;
pub const SETTINGS_WINDOW_HEIGHT: f32 = 600.0;
pub const SETTINGS_MIN_WIDTH: f32 = 720.0;
pub const SETTINGS_MIN_HEIGHT: f32 = 520.0;
/// Settings navigation column.
pub const NAV_WIDTH: f32 = 188.0;
/// Settings label column.
pub const LABEL_WIDTH: f32 = 160.0;
