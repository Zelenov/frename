//! The style functions behind the `ui` components: every look of the design system, built from
//! the tokens (`docs/design/design-system.md` §8). Split by what they style.

mod button;
mod form;
mod scroll;
mod surface;

use iced::{Shadow, Vector};

pub use button::*;
pub use form::*;
pub use scroll::*;
pub use surface::*;

use super::tokens::*;

/// The one shadow, under popups.
fn popup_shadow() -> Shadow {
    Shadow {
        color: SHADOW,
        offset: Vector::new(0.0, SHADOW_OFFSET_Y),
        blur_radius: SHADOW_BLUR,
    }
}

/// The lift of a dragged chip.
pub fn lift_shadow() -> Shadow {
    Shadow {
        color: SHADOW,
        offset: Vector::new(0.0, LIFT_OFFSET_Y),
        blur_radius: LIFT_BLUR,
    }
}
