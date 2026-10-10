//! State of a drag out of the window, and the pure decisions around it.

use std::path::{Path, PathBuf};

use frename_core::{FileId, FileSnapshot};
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
        /// Saves of files since the press (the pressed row's own save, the file it left, the
        /// save the drag asked for), by file.
        saves: Vec<(FileId, Save)>,
    },
    /// The operating system's drag loop runs.
    Dragging,
}

/// A save of one file while a row is pressed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Save {
    /// Asked for by the drag; it has not run yet (its message or its video unload is pending).
    Pending,
    /// It ran and left the file as edited.
    Worked,
    /// It ran and did not (the rename or the markers failed; see [`save_failed`]).
    Failed,
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
            saves: Vec::new(),
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

    /// The last save of `file` since the press, if any.
    pub fn save_of(&self, file: FileId) -> Option<Save> {
        let Phase::Pressed { saves, .. } = &self.phase else {
            return None;
        };
        saves
            .iter()
            .find(|(id, _)| *id == file)
            .map(|(_, save)| *save)
    }

    fn set_save(&mut self, file: FileId, save: Save) {
        if let Phase::Pressed { saves, .. } = &mut self.phase {
            match saves.iter_mut().find(|(id, _)| *id == file) {
                Some(entry) => entry.1 = save,
                None => saves.push((file, save)),
            }
        }
    }

    /// This drag asks for a save of `file` (the open file).
    pub fn ask_save(&mut self, file: FileId) {
        self.set_save(file, Save::Pending);
    }

    /// `file` was saved, `failed` or not, while a row is pressed: whatever the file, a drag
    /// that carries it must know (the press itself saves the file it leaves).
    pub fn saved(&mut self, file: FileId, failed: bool) {
        let save = if failed { Save::Failed } else { Save::Worked };
        self.set_save(file, save);
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

    /// The drag loop returned (dropped, cancelled or not started). Only a running drag ends:
    /// a late end must not clear a newer press.
    pub fn finish(&mut self) {
        if matches!(self.phase, Phase::Dragging) {
            self.phase = Phase::Idle;
        }
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

/// Whether a save of `wanted` failed, judged by the file it left (`saved`: reparsed after the
/// save). Only the tags, the name and the extension are compared: they are always in the file
/// name, so a failed rename shows there. In/out are not: where they are kept depends on the
/// settings, and reading them back normalises them (an in point at 0 s reads as none).
/// `wanted` is compared as the name it makes reads back, not as it is held: a tag with a dot
/// (`v1.2`) reads back as two tags (`v1`, `2`), and that save worked.
pub fn save_failed(wanted: &FileSnapshot, saved: &FileSnapshot) -> bool {
    let expected = FileSnapshot::parse(&wanted.file_name());
    expected.tags() != saved.tags()
        || expected.name_without_extension() != saved.name_without_extension()
        || !expected.extension().eq_ignore_ascii_case(saved.extension())
}

/// `path` made absolute against the current directory (without touching the disk), as the
/// Windows shell needs it for a drag; unchanged when that is not possible.
pub fn absolute_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(path),
        Err(_) => path.to_path_buf(),
    }
}

/// What the drag decision looks at, over all the dragged files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DragCheck {
    /// A save waits for the video to unload, or a save the drag asked for has not run yet.
    pub waiting: bool,
    /// A dragged file's save failed since the press, or its markers are not written.
    pub failed: bool,
    /// The open file is dragged and its name on disk is not the one its edits make (maybe only
    /// because reading back normalises; a save settles it).
    pub open_unsaved: bool,
    /// The open file was saved since the press and that worked.
    pub open_saved: bool,
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
    /// A dragged file's save failed, so the drag does not start.
    Refuse,
}

/// Whether a drag may start. A save counts once it ran, not when it was asked for (its message
/// may be queued, or its video unloading); its own outcome decides, not a comparison of names.
pub fn readiness(check: DragCheck) -> Readiness {
    if check.waiting {
        Readiness::Wait
    } else if check.failed {
        Readiness::Refuse
    } else if check.open_unsaved && !check.open_saved {
        Readiness::SaveFirst
    } else {
        Readiness::Start
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
    fn saves_are_kept_per_file_and_belong_to_one_press() {
        let (open, left) = (id(), id());
        let mut state = DragOutState::default();
        state.saved(left, true);
        assert_eq!(state.save_of(left), None, "nothing pressed");
        state.press(open);
        // The press saves the file it leaves: kept, whether the drag asked or not.
        state.saved(left, true);
        assert_eq!(state.save_of(left), Some(Save::Failed));
        state.ask_save(open);
        assert_eq!(state.save_of(open), Some(Save::Pending));
        state.saved(open, false);
        assert_eq!(state.save_of(open), Some(Save::Worked));
        state.press(open);
        assert_eq!((state.save_of(open), state.save_of(left)), (None, None));
    }

    #[test]
    fn a_late_end_does_not_clear_a_newer_press() {
        let file = id();
        let mut state = DragOutState::default();
        state.press(file);
        state.finish();
        assert!(state.is_pressed());
        state.start();
        state.finish();
        assert!(!state.is_pressed() && !state.is_dragging());
    }

    /// A save that worked is not taken for a failed one because the file reads back a little
    /// differently: an in point at 0 s kept in XMP reads back as none, and in/out points are
    /// never in the name. A rename that did not happen is a failure.
    #[test]
    fn a_save_failed_only_when_the_tags_or_the_name_did_not_make_it() {
        let mut wanted = FileSnapshot::parse("pick.clip.mp4");
        wanted.set_segment_start(Some(0.0));
        wanted.set_segment_end(Some(10.0));
        assert_eq!(wanted.tags(), ["pick"]);
        let mut read_back = FileSnapshot::parse("pick.clip.mp4");
        read_back.set_segment_end(Some(10.0));
        assert!(!save_failed(&wanted, &read_back), "in at 0 s reads as none");
        read_back.set_segment_end(Some(9.9996));
        wanted.set_segment_end(Some(10.0));
        assert!(!save_failed(&wanted, &read_back), "rounded");
        let in_video = FileSnapshot::parse("pick.clip.MP4");
        assert!(!save_failed(&wanted, &in_video), "in/out not in the name");
        let old_name = FileSnapshot::parse("clip.mp4");
        assert!(save_failed(&wanted, &old_name), "the rename did not happen");
    }

    /// A tag with a dot reads back as two tags, so the saved file never equals the held tags.
    #[test]
    fn a_save_with_a_dotted_tag_that_worked_is_not_a_failure() {
        let mut wanted = FileSnapshot::parse("clip.mp4");
        wanted.set_tags(["v1.2"]);
        let read_back = FileSnapshot::parse(&wanted.file_name());
        assert_eq!(read_back.tags(), ["v1", "2"]);
        assert!(!save_failed(&wanted, &read_back), "the rename happened");
        let old_name = FileSnapshot::parse("clip.mp4");
        assert!(save_failed(&wanted, &old_name), "the rename did not happen");
    }

    #[test]
    fn a_relative_path_is_made_absolute_and_an_absolute_one_is_kept() {
        let relative = Path::new("clips").join("clip.mp4");
        let made = absolute_path(&relative);
        assert!(made.is_absolute());
        assert!(made.ends_with(&relative));
        let cwd = std::env::current_dir().expect("a current directory");
        assert_eq!(made, cwd.join(&relative));
        assert_eq!(absolute_path(&made), made);
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
    fn a_drag_waits_for_saves_saves_once_and_refuses_only_when_a_save_failed() {
        let check = |waiting, failed, open_unsaved, open_saved| {
            readiness(DragCheck {
                waiting,
                failed,
                open_unsaved,
                open_saved,
            })
        };
        assert_eq!(check(true, true, true, false), Readiness::Wait);
        assert_eq!(check(false, false, false, false), Readiness::Start);
        assert_eq!(check(false, false, true, false), Readiness::SaveFirst);
        // Names still differ after a save that worked (reading back normalised): start.
        assert_eq!(check(false, false, true, true), Readiness::Start);
        // Any dragged file's failed save refuses, the open one's or another's.
        assert_eq!(check(false, true, false, true), Readiness::Refuse);
    }
}
