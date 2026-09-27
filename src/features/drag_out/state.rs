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
        /// Where the save of the open file for this drag is.
        save: Save,
    },
    /// The operating system's drag loop runs.
    Dragging,
}

/// The save of the open file that a drag asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Save {
    /// None asked for.
    NotAsked,
    /// Asked for this file; it has not run yet (its message or its video unload is pending).
    Asked(FileId),
    /// It ran: edits still not on disk after it mean it failed.
    Ran,
}

/// A press on a file row that may become a drag out of the window.
#[derive(Debug, Default)]
pub struct DragOutState {
    phase: Phase,
}

impl DragOutState {
    /// A file row was pressed. Also after a drag whose end never came back: no press reaches
    /// the window while the system's drag loop runs, so a press means that drag is over.
    pub fn press(&mut self, file: FileId) {
        self.phase = Phase::Pressed {
            file,
            origin: None,
            crossed: false,
            save: Save::NotAsked,
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

    /// Whether this drag asked for a save that has not run yet.
    pub fn save_pending(&self) -> bool {
        matches!(
            self.phase,
            Phase::Pressed {
                save: Save::Asked(_),
                ..
            }
        )
    }

    /// Whether the save this drag asked for has run.
    pub fn save_ran(&self) -> bool {
        matches!(
            self.phase,
            Phase::Pressed {
                save: Save::Ran,
                ..
            }
        )
    }

    /// This drag asks for a save of `file` (the open file).
    pub fn ask_save(&mut self, file: FileId) {
        if let Phase::Pressed { save, .. } = &mut self.phase {
            *save = Save::Asked(file);
        }
    }

    /// `file` was saved: if this drag waited for that, the save has run.
    pub fn saved(&mut self, file: FileId) {
        if let Phase::Pressed { save, .. } = &mut self.phase {
            if *save == Save::Asked(file) {
                *save = Save::Ran;
            }
        }
    }

    /// Whether a file row is pressed (armed, not yet dragging).
    #[cfg(test)]
    pub fn is_pressed(&self) -> bool {
        matches!(self.phase, Phase::Pressed { .. })
    }

    /// Whether the system's drag loop was started.
    #[cfg(test)]
    pub fn is_dragging(&self) -> bool {
        matches!(self.phase, Phase::Dragging)
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
/// `listed` is only walked in that case.
pub fn files_to_drag(
    pressed: FileId,
    listed: impl IntoIterator<Item = FileId>,
    batch_mode: bool,
    is_checked: impl Fn(FileId) -> bool,
) -> Vec<FileId> {
    if batch_mode && is_checked(pressed) {
        return listed.into_iter().filter(|id| is_checked(*id)).collect();
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

/// Whether a drag may start. `save_in_flight`: a save of the open file is asked for and has not
/// run yet (its message is queued, or it waits for the video to unload). `unsaved`: the open file
/// is among the dragged ones and its edits are not all on disk. `save_ran`: the save this drag
/// asked for has run.
pub fn readiness(save_in_flight: bool, unsaved: bool, save_ran: bool) -> Readiness {
    match (save_in_flight, unsaved, save_ran) {
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
    fn the_save_counts_once_it_ran_for_the_file_asked_and_belongs_to_one_press() {
        let (open, other) = (id(), id());
        let mut state = DragOutState::default();
        state.press(open);
        assert!(!state.save_pending() && !state.save_ran());
        state.ask_save(open);
        assert!(state.save_pending() && !state.save_ran());
        state.saved(other);
        assert!(state.save_pending(), "another file's save does not count");
        state.saved(open);
        assert!(!state.save_pending() && state.save_ran());
        state.press(open);
        assert!(!state.save_pending() && !state.save_ran());
    }

    #[test]
    fn a_press_after_a_drag_whose_end_never_came_arms_again() {
        let file = id();
        let mut state = DragOutState::default();
        state.press(file);
        state.start();
        assert!(state.is_dragging());
        state.press(file);
        assert!(state.is_pressed());
    }

    #[test]
    fn a_checked_row_in_batch_mode_drags_every_checked_file_in_list_order() {
        let (a, b, c, d) = (id(), id(), id(), id());
        let listed = [a, b, c, d];
        let checked = |f: FileId| f == d || f == b;
        assert_eq!(files_to_drag(d, listed, true, checked), vec![b, d]);
        // An unchecked row drags only itself.
        assert_eq!(files_to_drag(a, listed, true, checked), vec![a]);
        // Outside batch mode checks do not count.
        assert_eq!(files_to_drag(b, listed, false, checked), vec![b]);
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
