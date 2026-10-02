//! Sizes of the windows and of the regions of the main window (`docs/design/design-system.md`
//! §13, §13.9, §14.1): their minimums, defaults and maximums, and the fixed heights of their bars.

use super::size::*;
use super::space::*;
use super::typography::{
    BODY_CHAR_WIDTH, LINE_BODY, LINE_CAPTION, LINE_TOOLTIP, TEXT_BODY, TEXT_CHIP_MINI,
};

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
pub const VIDEO_WIDTH: f32 = 440.0;
pub const FILE_LIST_WIDTH: f32 = 300.0;
/// The block of the whole-window empty screen ("Open a folder of clips"), left-aligned.
pub const EMPTY_SCREEN_MAX_WIDTH: f32 = 420.0;
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
/// The time readout: a fixed width so it never moves. Fits the longest it shows, a clip of hours
/// paused (`1:02:05.250 / 1:30:00`): 21 mono characters, 151.2 px.
pub const TIME_READOUT_WIDTH: f32 = 152.0;
/// The notices' slot in the controls bar needs this much, otherwise they float over the picture.
pub const NOTICE_SLOT_MIN_WIDTH: f32 = 120.0;
/// A marker row's color dot.
pub const MARKER_DOT: f32 = 14.0;
/// The timeline's and the volume's track.
pub const TRACK_HEIGHT: f32 = 6.0;
/// How far the playhead line reaches beyond the track at both ends, and its knob while the
/// timeline is hovered or dragged.
pub const PLAYHEAD_OVERHANG: f32 = 4.0;
pub const PLAYHEAD_KNOB: f32 = 10.0;
/// A point marker's round head, and how far its needle reaches below the track.
pub const PIN_HEAD: f32 = 7.0;
pub const PIN_OVERHANG: f32 = 3.0;
/// A range's band in its lane (lanes are `LINE` apart).
pub const BAND_HEIGHT: f32 = 4.0;
/// The handles at the ends of the active range.
pub const RANGE_HANDLE_WIDTH: f32 = 6.0;
pub const RANGE_HANDLE_HEIGHT: f32 = 8.0;
/// How far around a band or a handle a press still hits it.
pub const TIMELINE_HIT_SLACK: f32 = 3.0;
/// A Shift seek or a dragged handle snaps to a marker this close to the pointer.
pub const SNAP_REACH: f32 = 8.0;
/// The lane at the top of the timeline that the active marker's label sits in, so it never
/// covers the subtitle strip above: its line of text, padding and edge.
pub const MARKER_LABEL_LANE: f32 = LINE_TOOLTIP + 2.0 * SPACE_XXS + 2.0 * RING;
/// A subtitle row of the side list without its text: the time line and the row's inset. Each
/// line of the cue adds `LINE_BODY`: rows take their natural height.
pub const CUE_ROW_CHROME: f32 = LINE_BODY + 2.0 * SPACE_TIGHT;
/// Room for a cue's text in a side list of the usual width: the list's inset, the gutter and the
/// row's inset go first.
pub const CUE_TEXT_WIDTH: f32 = SIDE_LIST_MAX_WIDTH - 3.0 * SPACE_S - SCROLL_GUTTER;
/// A marker row with a one-line name: its first line of row buttons and the name. A longer
/// name adds `LINE_BODY` per line.
pub const MARKER_ROW_HEIGHT: f32 = ICON_BUTTON_SMALL + SPACE_XXS + LINE_BODY + 2.0 * SPACE_TIGHT;

// File list (§13.4).
/// The search bar over a list: a 28 px field with 6 px around it.
pub const SEARCH_BAR_HEIGHT: f32 = CONTROL_HEIGHT + 2.0 * SPACE_TIGHT;
/// A file row: the name line and the comment line.
pub const FILE_ROW_HEIGHT: f32 = 52.0;
/// The check column in batch mode, and the status column (SRT, lock).
pub const CHECK_COLUMN: f32 = 28.0;
pub const STATUS_COLUMN: f32 = 32.0;
/// A file name keeps at least this much beside its chips: chips give way first.
pub const NAME_MIN_WIDTH: f32 = 80.0;
/// The line over the list that says why it is locked.
pub const LOCK_LINE_HEIGHT: f32 = CONTROL_HEIGHT;

// Tags area (§13.5).
pub const CHIP_HEIGHT: f32 = CONTROL_HEIGHT;
pub const CHIP_MINI_HEIGHT: f32 = 20.0;
/// A grid column is at most this wide; longer names are cut inside the chip.
pub const CHIP_MAX_WIDTH: f32 = 240.0;
/// The order strip between the grid and the file name card.
pub const ORDER_STRIP_HEIGHT: f32 = BAR_HEIGHT;
/// From this width of the order strip on, its buttons show their words; narrower, only icons.
pub const ORDER_STRIP_WORDS_FROM: f32 = 420.0;
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
/// The average width of one character of a mini chip's 12-px label: not generous, since the file
/// list fits chips to the row by it and a generous guess hides chips that would fit.
pub const CHIP_MINI_CHAR_WIDTH: f32 = BODY_CHAR_WIDTH * TEXT_CHIP_MINI / TEXT_BODY;
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
/// A group's caption over its rows ("Not in the folder's tags", "Folder tags").
pub const GRID_CAPTION_HEIGHT: f32 = LINE_CAPTION + SPACE_XS;
/// Between the last row of one group and the next group's caption (§13.5.2: 8 px).
pub const GRID_GROUP_GAP: f32 = SPACE_S;
/// Columns of the tag grid: never fewer, however narrow the area.
pub const GRID_MIN_COLUMNS: u32 = 2;

// Batch panel (§13.6).
pub const BATCH_HEADER_HEIGHT: f32 = 48.0;
pub const ACTION_LIST_WIDTH: f32 = 232.0;
/// The panel's width on entering batch mode: the whole action list and a page.
pub const BATCH_PANEL_WIDTH: f32 = 600.0;
/// Narrower than `BATCH_PANEL_WIDTH` the action list shows only its icons, this wide; narrower
/// than `BATCH_ICON_LIST_FROM` it becomes a dropdown over the page (§13.9).
pub const ACTION_LIST_ICONS_WIDTH: f32 = 48.0;
pub const BATCH_ICON_LIST_FROM: f32 = 440.0;
/// The failed files' table scrolls after this many rows.
pub const FAILED_ROWS_SHOWN: f32 = 8.0;
/// A job's progress bar.
pub const PROGRESS_HEIGHT: f32 = 6.0;
/// A row of the failed files' table: a line of text and 6 px above and below.
pub const TABLE_ROW_HEIGHT: f32 = LINE_BODY + 2.0 * SPACE_TIGHT;

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
