//! State for drag and drop feature

use iced::{event, window, Subscription};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::Message;

/// How long after the last dropped item the drop counts as complete. The window gets one event
/// per dropped item, back to back, with no event marking the end of the drop.
const DROP_QUIET: Duration = Duration::from_millis(50);

/// How long after frename's own drag out of the window ends that drops of its files are still
/// ignored: winit holds back the drop events of the drag loop and delivers them after it, in no
/// fixed order with the drag's result.
const OWN_DRAG_GRACE: Duration = Duration::from_secs(1);

/// Collects the items of one drop, so a drop of several items opens only one of them.
#[derive(Default)]
pub struct DragDropState {
    /// Items of the drop in progress, in the order they arrived.
    pending: Vec<PathBuf>,
    /// When the last item arrived.
    last_drop: Option<Instant>,
    /// Files frename itself drags out of the window: dropped back on it, they are ignored.
    own_drag: Vec<PathBuf>,
    /// While frename's own drag runs this is `None`; after it, until when its drops are ignored.
    own_drag_until: Option<Instant>,
}

impl DragDropState {
    /// Remember a dropped item until the drop is complete.
    pub fn handle_file_dropped(&mut self, path: PathBuf, now: Instant) {
        if self.is_own_drop(&path, now) {
            log::info!(
                "Own drag dropped back on the window, ignored: {}",
                path.display()
            );
            return;
        }
        log::info!("File dropped: {}", path.display());
        self.pending.push(path);
        self.last_drop = Some(now);
    }

    /// frename starts dragging `paths` out of the window.
    pub fn begin_own_drag(&mut self, paths: Vec<PathBuf>) {
        self.own_drag = paths;
        self.own_drag_until = None;
    }

    /// frename's own drag ended at `now`; its drops still arriving are ignored for a moment.
    pub fn end_own_drag(&mut self, now: Instant) {
        self.own_drag_until = Some(now + OWN_DRAG_GRACE);
    }

    /// Whether `path` is one of the files of frename's own drag, dropped while it runs or just
    /// after.
    fn is_own_drop(&self, path: &std::path::Path, now: Instant) -> bool {
        let recent = self.own_drag_until.map_or(true, |until| now < until);
        // Case-insensitive: the path comes back from the shell, maybe spelled differently.
        recent
            && self.own_drag.iter().any(|own| {
                own.to_string_lossy()
                    .eq_ignore_ascii_case(&path.to_string_lossy())
            })
    }

    /// On a timer tick: when the drop is complete, the path to open (the first folder, or else
    /// the first file), if any item of the drop still exists.
    pub fn handle_tick(&mut self, now: Instant) -> Option<PathBuf> {
        let last = self.last_drop?;
        if now.saturating_duration_since(last) < DROP_QUIET {
            return None;
        }
        self.last_drop = None;
        let dropped = std::mem::take(&mut self.pending);
        let chosen = frename_core::choose_dropped_path(&dropped).map(PathBuf::from);
        if chosen.is_none() {
            log::warn!("Nothing to open among the dropped items: {dropped:?}");
        }
        chosen
    }

    /// Listen for file drop window events, and tick while a drop waits to complete.
    pub fn subscription(&self) -> Subscription<Message> {
        let drops = event::listen_with(|event, _status, _id| match event {
            iced::Event::Window(window::Event::FileDropped(path)) => {
                Some(Message::FileDropped(path))
            }
            _ => None,
        });
        if self.pending.is_empty() {
            return drops;
        }
        Subscription::batch([drops, iced::time::every(DROP_QUIET).map(Message::Tick)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("frename-drop-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_drop_opens_nothing_until_it_is_complete() {
        let dir = temp_dir("wait");
        let start = Instant::now();
        let mut state = DragDropState::default();
        state.handle_file_dropped(dir.clone(), start);
        assert_eq!(state.handle_tick(start + DROP_QUIET / 2), None);
        assert_eq!(state.handle_tick(start + DROP_QUIET), Some(dir));
        // The drop is done: later ticks open nothing again.
        assert_eq!(state.handle_tick(start + DROP_QUIET * 3), None);
    }

    #[test]
    fn several_items_of_one_drop_open_the_folder_among_them() {
        let dir = temp_dir("several");
        let file = dir.join("a.mp4");
        std::fs::write(&file, b"a").unwrap();
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        let start = Instant::now();
        let mut state = DragDropState::default();
        state.handle_file_dropped(file, start);
        // The next item arrives while the drop waits, so it belongs to the same drop.
        state.handle_file_dropped(sub.clone(), start + DROP_QUIET / 2);
        assert_eq!(state.handle_tick(start + DROP_QUIET), None);
        assert_eq!(state.handle_tick(start + DROP_QUIET * 2), Some(sub));
    }

    #[test]
    fn own_dragged_files_dropped_back_are_ignored_during_the_drag_and_just_after() {
        let dir = temp_dir("own");
        let own = dir.join("Goat.clip.mp4");
        std::fs::write(&own, b"a").unwrap();
        let start = Instant::now();
        let mut state = DragDropState::default();
        state.begin_own_drag(vec![own.clone()]);
        state.handle_file_dropped(own.clone(), start);
        assert_eq!(
            state.handle_tick(start + DROP_QUIET),
            None,
            "during the drag"
        );
        state.end_own_drag(start);
        state.handle_file_dropped(own.clone(), start + OWN_DRAG_GRACE / 2);
        assert_eq!(
            state.handle_tick(start + OWN_DRAG_GRACE),
            None,
            "just after"
        );
        // Another path still opens, and the same file dropped later opens too.
        state.handle_file_dropped(dir.clone(), start + OWN_DRAG_GRACE / 2);
        assert_eq!(state.handle_tick(start + OWN_DRAG_GRACE), Some(dir.clone()));
        let later = start + OWN_DRAG_GRACE * 2;
        state.handle_file_dropped(own.clone(), later);
        assert_eq!(state.handle_tick(later + DROP_QUIET), Some(own));
    }

    #[test]
    fn a_drop_of_missing_items_opens_nothing() {
        let dir = temp_dir("missing");
        let start = Instant::now();
        let mut state = DragDropState::default();
        state.handle_file_dropped(dir.join("gone.mp4"), start);
        assert_eq!(state.handle_tick(start + DROP_QUIET), None);
        assert_eq!(state.handle_tick(start + DROP_QUIET * 2), None);
    }
}
