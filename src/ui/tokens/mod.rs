//! Design tokens: every color, space, size, radius, text size, font and duration of the design
//! system (`docs/design/design-system.md` §3–7, §13.8–13.9). Views take values from here, never
//! literals. Consts only, grouped by kind; `use crate::ui::tokens::*` brings them all.

mod color;
mod content;
mod region;
mod size;
mod space;
mod typography;

use std::time::Duration;

pub use color::*;
pub use content::*;
pub use region::*;
pub use size::*;
pub use space::*;
pub use typography::*;

/// Hover time before a tooltip shows.
pub const TOOLTIP_DELAY: Duration = Duration::from_millis(500);
/// One step of the spinner clock: `ui::icons::spinner` turns once in twelve.
pub const SPINNER_TICK: Duration = Duration::from_millis(150);
