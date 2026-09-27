//! What the app says about turning a video, shared by the open clip's notes, the ↺ ↻ tooltips
//! and the "Rotate videos" batch action, so all of them read the same.

use frename_core::{Rotation, RotationError};

/// Why a video was not (or cannot be) turned. The system's own words (a sharing violation,
/// access denied) are in the log.
pub fn why_not_rotated(error: &RotationError) -> String {
    match error {
        RotationError::CannotRotate => fl!("rotate-reason-format"),
        RotationError::Damaged => fl!("rotate-reason-damaged"),
        RotationError::NoVideoTrack => fl!("rotate-reason-no-video"),
        RotationError::UnusualMatrix => fl!("rotate-reason-matrix"),
        RotationError::Missing => fl!("rotate-reason-missing"),
        RotationError::Io(_) => fl!("rotate-reason-in-use"),
    }
}

/// The note after a turn: how the clip is turned now.
pub fn rotated(rotation: Rotation) -> String {
    match rotation.degrees() {
        90 => fl!("rotate-now-right"),
        180 => fl!("rotate-now-half"),
        270 => fl!("rotate-now-left"),
        _ => fl!("rotate-now-none"),
    }
}
