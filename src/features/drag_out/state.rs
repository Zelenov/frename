//! State of a drag out of the window, and the pure decisions around it.

use frename_core::FileId;
use iced::{event, mouse, Point, Subscription};

use super::Message;

/// How far (logical pixels) the pointer moves with the button held on a file row before the
/// drag starts. Windows' own threshold (`SM_CXDRAG`) is 4 physical pixels; a little more keeps a
/// slightly shaky click a click.
pub const DRAG_THRESHOLD: f32 = 6.0;

/// Where a drag out of the window is.
#[derive(Debug, Default)]
enum Phase {
    /// No file row is pressed.
    #[default]
    Idle,
    /// A file row is pressed and the button is still down.
    Pressed {
        /// The file whose row was pressed.
        file: FileId,
        /// The first cursor position seen after the press; the threshold counts from it.
        origin: Option<Point>,
        /// The pointer went past the threshold: the drag is asked for on every move until it
        /// starts (it may wait for the file to be saved).
        crossed: bool,
        /// The open file was saved once for this drag; unsaved after that means the save failed.
        save_attempted: bool,
    },
    /// The operating system's drag loop runs.
    Dragging,
}

/// A press on a file row that may become a drag out of the window.
#[derive(Debug, Default)]
pub struct DragOutState {
    phase: Phase,
}

impl DragOutState {
    /// A file row was pressed.
    pub fn press(&mut self, file: FileId) {
        if matches!(self.phase, Phase::Dragging) {
            return;
        }
        self.phase = Phase::Pressed {
            file,
            origin: None,
            crossed: false,
            save_attempted: false,
        };
    }

    /// The button went up, or the drag was refused: back to idle.
    pub fn release(&mut self) {
        if matches!(self.phase, Phase::Pressed { .. }) {
            self.phase = Phase::Idle;
        }
    }

    /// The cursor moved. Returns the pressed file when the drag should be asked for: the
    /// pointer is (or already was) past [`DRAG_THRESHOLD`] with the button held.
    pub fn moved(&mut self, position: Point) -> Option<FileId> {
        let Phase::Pressed {
            file,
            origin,
            crossed,
            ..
        } = &mut self.phase
        else {
            return None;
        };
        if *crossed {
            return Some(*file);
        }
        let Some(start) = *origin else {
            *origin = Some(position);
            return None;
        };
        if start.distance(position) < DRAG_THRESHOLD {
            return None;
        }
        *crossed = true;
        Some(*file)
    }

    /// Whether the open file was already saved once for this drag.
    pub fn save_attempted(&self) -> bool {
        matches!(
            self.phase,
            Phase::Pressed {
                save_attempted: true,
                ..
            }
        )
    }

    /// Remember that the open file is being saved for this drag.
    pub fn mark_save_attempted(&mut self) {
        if let Phase::Pressed { save_attempted, .. } = &mut self.phase {
            *save_attempted = true;
        }
    }

    /// The operating system's drag starts.
    pub fn start(&mut self) {
        self.phase = Phase::Dragging;
    }

    /// The drag loop returned (dropped, cancelled or not started).
    pub fn finish(&mut self) {
        self.phase = Phase::Idle;
    }

    /// Cursor moves and the button release, only while a file row is pressed.
    pub fn subscription(&self) -> Subscription<Message> {
        if !matches!(self.phase, Phase::Pressed { .. }) {
            return Subscription::none();
        }
        event::listen_with(|event, _status, _window| match event {
            iced::Event::Mouse(mouse::Event::CursorMoved { position }) => {
                Some(Message::Moved(position))
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                Some(Message::Released)
            }
            _ => None,
        })
    }
}

/// The files a drag from the row of `pressed` carries. In batch mode a checked row carries every
/// checked file of the list, in list order; any other row carries only its own file.
pub fn files_to_drag(
    pressed: FileId,
    listed: &[FileId],
    batch_mode: bool,
    is_checked: impl Fn(FileId) -> bool,
) -> Vec<FileId> {
    if batch_mode && is_checked(pressed) {
        return listed
            .iter()
            .copied()
            .filter(|id| is_checked(*id))
            .collect();
    }
    vec![pressed]
}

/// What to do when the drag is asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Readiness {
    /// Start the drag now.
    Start,
    /// A save of the open file is still under way (its video unloads first): ask again on the
    /// next move.
    Wait,
    /// The open file has edits not on disk: save it, then ask again.
    SaveFirst,
    /// The open file was saved for this drag and still is not on disk as edited: the save
    /// failed, so the drag does not start.
    Refuse,
}

/// Whether a drag may start. `save_in_flight`: a save of the open file waits for its video to
/// unload. `unsaved`: the open file is among the dragged ones and its edits are not all on disk.
/// `save_attempted`: the open file was already saved once for this drag.
pub fn readiness(save_in_flight: bool, unsaved: bool, save_attempted: bool) -> Readiness {
    match (save_in_flight, unsaved, save_attempted) {
        (true, _, _) => Readiness::Wait,
        (false, false, _) => Readiness::Start,
        (false, true, false) => Readiness::SaveFirst,
        (false, true, true) => Readiness::Refuse,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn id() -> FileId {
        FileId::new()
    }

    #[test]
    fn a_small_move_is_still_a_click_and_the_threshold_starts_the_drag_once_asked() {
        let file = id();
        let mut state = DragOutState::default();
        assert_eq!(state.moved(Point::new(10.0, 10.0)), None, "nothing pressed");
        state.press(file);
        assert_eq!(state.moved(Point::new(10.0, 10.0)), None, "origin");
        assert_eq!(state.moved(Point::new(13.0, 13.0)), None, "4.2 px");
        assert_eq!(state.moved(Point::new(16.0, 10.0)), Some(file), "6 px");
        // Still waiting to start (e.g. for a save): every move asks again.
        assert_eq!(state.moved(Point::new(16.0, 11.0)), Some(file));
        state.start();
        assert_eq!(state.moved(Point::new(40.0, 40.0)), None, "the drag runs");
        state.finish();
        assert_eq!(state.moved(Point::new(80.0, 80.0)), None, "idle again");
    }

    #[test]
    fn a_release_before_the_threshold_disarms() {
        let file = id();
        let mut state = DragOutState::default();
        state.press(file);
        assert_eq!(state.moved(Point::new(0.0, 0.0)), None);
        state.release();
        assert_eq!(state.moved(Point::new(50.0, 0.0)), None);
        // A new press starts over, with a new origin.
        state.press(file);
        assert_eq!(state.moved(Point::new(50.0, 0.0)), None);
        assert_eq!(state.moved(Point::new(50.0, 6.0)), Some(file));
    }

    #[test]
    fn the_save_attempt_belongs_to_one_press() {
        let mut state = DragOutState::default();
        state.press(id());
        assert!(!state.save_attempted());
        state.mark_save_attempted();
        assert!(state.save_attempted());
        state.press(id());
        assert!(!state.save_attempted());
    }

    #[test]
    fn a_checked_row_in_batch_mode_drags_every_checked_file_in_list_order() {
        let (a, b, c, d) = (id(), id(), id(), id());
        let listed = [a, b, c, d];
        let checked = |f: FileId| f == d || f == b;
        assert_eq!(files_to_drag(d, &listed, true, checked), vec![b, d]);
        // An unchecked row drags only itself.
        assert_eq!(files_to_drag(a, &listed, true, checked), vec![a]);
        // Outside batch mode checks do not count.
        assert_eq!(files_to_drag(b, &listed, false, checked), vec![b]);
    }

    #[test]
    fn a_drag_waits_for_a_save_saves_once_and_refuses_when_the_save_failed() {
        assert_eq!(readiness(true, true, false), Readiness::Wait);
        assert_eq!(readiness(true, false, true), Readiness::Wait);
        assert_eq!(readiness(false, false, false), Readiness::Start);
        assert_eq!(readiness(false, false, true), Readiness::Start);
        assert_eq!(readiness(false, true, false), Readiness::SaveFirst);
        assert_eq!(readiness(false, true, true), Readiness::Refuse);
    }
}
