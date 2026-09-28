//! Spacing (`docs/design/design-system.md` §5.1, §5.3): a 4-px base, each step at least 25 %
//! bigger than the one before. Space inside a group is smaller than space between groups.

/// Between rows of a list; between a label and its description.
pub const SPACE_XXS: f32 = 2.0;
/// Between buttons of a toolbar group; between chips; icon to text in a badge.
pub const SPACE_XS: f32 = 4.0;
/// Between an icon and its words in a button; the inset of tooltips and of the search bar.
pub const SPACE_TIGHT: f32 = 6.0;
/// Inside a group: between controls of one row, options of one question, buttons of a bar.
pub const SPACE_S: f32 = 8.0;
/// Between radio options with descriptions; between groups of a toolbar.
pub const SPACE_M: f32 = 12.0;
/// Padding of button bars; between the label column and the control column.
pub const SPACE_L: f32 = 16.0;
/// Between rows of a Settings page; page side padding.
pub const SPACE_XL: f32 = 24.0;
/// Between sections of a long page.
pub const SPACE_XXL: f32 = 32.0;

/// Page padding: top and bottom.
pub const PAGE_PADDING_Y: f32 = 20.0;
/// Inside a modal dialog.
pub const DIALOG_PADDING: f32 = 20.0;
