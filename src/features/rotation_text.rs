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

/// The note for a turn that did not happen: `Not rotated: <reason>`.
pub fn not_rotated(error: &RotationError) -> String {
    fl!("rotate-failed", reason = why_not_rotated(error))
}

/// The file's rotation flag, as the notes name it: `90° right`, `180°`, `none`. It names the
/// flag, not the picture: a phone's portrait clip plays upright with a 90° flag.
fn flag(rotation: Rotation) -> String {
    match rotation.degrees() {
        90 => fl!("rotate-flag-right"),
        180 => fl!("rotate-flag-half"),
        270 => fl!("rotate-flag-left"),
        _ => fl!("rotate-flag-none"),
    }
}

/// The note after an undo or redo: `Rotation: 90° right`.
pub fn rotated(rotation: Rotation) -> String {
    fl!("rotate-now", flag = flag(rotation))
}

/// The note after a turn by `quarter_turns` that left the flag at `now`: `Rotation: 90° right`
/// when the clip had no rotation before, else what was pressed and the result (`Turned 90°
/// right · rotation now 180°`), so a phone's clip does not read as turned by 180° at one press.
pub fn turned(quarter_turns: i32, now: Rotation) -> String {
    if now.turned(-quarter_turns).degrees() == 0 {
        return rotated(now);
    }
    let turn = Rotation::UPRIGHT.turned(quarter_turns);
    fl!("rotate-turned", turn = flag(turn), flag = flag(now))
}
