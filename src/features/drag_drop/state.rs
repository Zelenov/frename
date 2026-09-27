//! State for drag and drop feature

use iced::{event, window, Subscription};
use std::path::PathBuf;
use std::time::{Duration, Instant};

use super::Message;

/// How long after the last dropped item the drop counts as complete. The window gets one event
/// per dropped item, back to back, with no event marking the end of the drop.
const DROP_QUIET: Duration = Duration::from_millis(50);

/// Collects the items of one drop, so a drop of several items opens only one of them.
#[derive(Default)]
pub struct DragDropState {
    /// Items of the drop in progress, in the order they arrived.
    pending: Vec<PathBuf>,
    /// When the last item arrived.
    last_drop: Option<Instant>,
}

impl DragDropState {
    /// Remember a dropped item until the drop is complete.
    pub fn handle_file_dropped(&mut self, path: PathBuf, now: Instant) {
        log::info!("File dropped: {}", path.display());
        self.pending.push(path);
        self.last_drop = Some(now);
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
    fn a_drop_of_missing_items_opens_nothing() {
        let dir = temp_dir("missing");
        let start = Instant::now();
        let mut state = DragDropState::default();
        state.handle_file_dropped(dir.join("gone.mp4"), start);
        assert_eq!(state.handle_tick(start + DROP_QUIET), None);
        assert_eq!(state.handle_tick(start + DROP_QUIET * 2), None);
    }
}
