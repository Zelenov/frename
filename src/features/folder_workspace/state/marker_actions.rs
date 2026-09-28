//! What the marker keys, the marker list and the progress bar do to the open file's markers:
//! add (a held `F2` draws a range), name, color, resize, delete and jump. The markers live in the file workspace's tag list, like in/out,
//! so saving, undo and renames carry them; this module also writes them into the file when it
//! is saved.

use std::path::Path;

use frename_core::{
    AddMarkerCommand, DeleteMarkerCommand, FileId, FileTagger, Marker, MarkersError,
    SetMarkerColorCommand, SetMarkerSpanCommand, MARKER_SNAP_MS,
};
use iced::widget::operation;
use iced::Task;

use super::FolderWorkspace;
use crate::features::folder_workspace::Message;
use crate::features::markers::{self, view::MARKER_LIST_SCROLLABLE_ID};
use crate::features::media_viewer::{self, video};

/// `Shift+F1` skips a marker the playhead passed less than this ago, so pressing it again
/// while playing goes on to the one before instead of back to the same one.
const PREVIOUS_SLACK_MS: u64 = 750;

impl FolderWorkspace {
    /// Apply a marker key or marker list message; `position_ms` is the playhead.
    pub(super) fn handle_marker(
        &mut self,
        msg: markers::Message,
        position_ms: u64,
    ) -> Task<Message> {
        use markers::Message as M;
        if self.file_workspace.file().is_none() {
            return Task::none();
        }
        if !matches!(msg, M::Add | M::KeyDown | M::KeyUp) {
            self.markers.forget_added();
        }
        match msg {
            M::Add => self.add_marker_key(position_ms, false),
            M::KeyDown => {
                // A press whose release never came (the 📍 button let go outside it) ends
                // there first.
                self.end_held_marker(position_ms);
                self.add_marker_key(position_ms, true)
            }
            M::KeyUp => {
                self.end_held_marker(position_ms);
                Task::none()
            }
            M::SetSpan(guid, start_ms, end_ms) => {
                self.set_marker_span(&guid, start_ms, end_ms);
                Task::none()
            }
            M::AddRange(start_ms, end_ms) => self.add_range(start_ms, end_ms),
            M::DeleteAtPlayhead => self.delete_marker_at(position_ms),
            M::Previous => self.jump_to_marker(position_ms, false),
            M::Next => self.jump_to_marker(position_ms, true),
            M::JumpTo(ms) => seek_exact(ms),
            M::Open(guid) => self.open_marker_row(guid),
            M::Close => {
                self.markers.close();
                Task::none()
            }
            M::NameAction(action) => {
                // A name is one line: the field wraps it, `Enter` closes the row.
                if let Some((guid, name)) = self.markers.edit_name(action) {
                    self.file_workspace
                        .tag_list_mut()
                        .update_marker(&guid, |m| m.name = name);
                }
                Task::none()
            }
            M::ToggleColorPicker(guid) => {
                self.markers.toggle_color_picker(guid);
                Task::none()
            }
            M::SetColor(guid, color) => {
                self.markers.close_color_picker();
                let Some(old) = self
                    .file_workspace
                    .tag_list()
                    .marker(&guid)
                    .map(|m| m.color)
                else {
                    return Task::none();
                };
                if old != color {
                    self.file_workspace
                        .tag_list_mut()
                        .update_marker(&guid, |m| m.color = color);
                    self.history.push(Box::new(SetMarkerColorCommand {
                        guid,
                        old,
                        new: color,
                    }));
                }
                Task::none()
            }
            M::Delete(guid) => {
                self.delete_marker(&guid);
                Task::none()
            }
        }
    }

    /// `F2` / `📍`: add a marker at the playhead; or open the row of the marker it just added
    /// (`F2 F2`) or of the one under the playhead, to name it. With a row open, close it and
    /// add without opening, so the next moment can be caught while typing. With `held` (the
    /// key, not the button), a marker it adds grows into a range until `F2` comes up.
    fn add_marker_key(&mut self, position_ms: u64, held: bool) -> Task<Message> {
        let Some(markers) = self.file_workspace.markers() else {
            // The list says the file cannot hold markers: a key press never does nothing.
            return self.show_marker_list();
        };
        let near = nearest(markers, position_ms).map(|m| m.guid.clone());
        if self.markers.is_editing() {
            self.markers.close();
            return match near {
                Some(_) => Task::none(),
                None => self.add_marker(position_ms, false, held),
            };
        }
        if let Some(guid) = self.markers.take_recently_added() {
            if self.file_workspace.tag_list().marker(&guid).is_some() {
                return self.open_marker_row(guid);
            }
        }
        match near {
            Some(Some(guid)) => self.open_marker_row(guid),
            Some(None) => Task::batch([
                self.show_marker_list(),
                Self::notice("That marker is read-only"),
            ]),
            None => self.add_marker(position_ms, true, held),
        }
    }

    /// Add a point marker at `position_ms`. One undo step, which takes the marker as it is
    /// when undone, so a range a held `F2` drew goes back and forth whole.
    fn add_marker(&mut self, position_ms: u64, remember: bool, held: bool) -> Task<Message> {
        let marker = Marker::new(position_ms);
        let guid = marker.guid.clone().unwrap_or_default();
        self.file_workspace
            .tag_list_mut()
            .add_marker(marker.clone());
        self.history.push(Box::new(AddMarkerCommand { marker }));
        if held {
            self.markers.start_recording(guid.clone(), position_ms);
        }
        if remember {
            self.markers.added(guid);
        }
        Task::none()
    }

    /// While `F2` is held, the marker it added reaches to the playhead (called on every
    /// playback tick).
    pub(super) fn grow_held_marker(&mut self) {
        let Some(position_ms) = self.media_viewer.video_position_ms() else {
            return;
        };
        let Some(recording) = self.markers.recording() else {
            return;
        };
        let (guid, duration_ms) = (recording.guid.clone(), recording.duration_at(position_ms));
        self.file_workspace
            .tag_list_mut()
            .update_marker(&guid, |m| m.duration_ms = duration_ms);
    }

    /// `F2` came up: the held marker ends at the playhead, or stays a point after a short
    /// press or when the playhead did not move past its start. `F2` right after still names
    /// it.
    fn end_held_marker(&mut self, position_ms: u64) {
        let Some(recording) = self.markers.stop_recording() else {
            return;
        };
        let duration_ms = at_least_a_range(recording.duration_at(position_ms));
        let found = self
            .file_workspace
            .tag_list_mut()
            .update_marker(&recording.guid, |m| m.duration_ms = duration_ms);
        if found {
            self.markers.added(recording.guid);
        }
    }

    /// A range's ends were dragged on the bar, or `Alt`+click made it a point: one undo step.
    /// Ends closer than [`markers::MIN_RANGE_MS`] make a point at the start.
    fn set_marker_span(&mut self, guid: &str, start_ms: u64, end_ms: u64) {
        let (start_ms, end_ms) = (start_ms.min(end_ms), start_ms.max(end_ms));
        let Some(old) = self
            .file_workspace
            .tag_list()
            .marker(guid)
            .filter(|m| m.is_editable())
            .map(|m| (m.start_ms, m.duration_ms))
        else {
            return;
        };
        let new = (start_ms, at_least_a_range(end_ms - start_ms));
        if old == new {
            return;
        }
        self.file_workspace
            .tag_list_mut()
            .update_marker(guid, |m| (m.start_ms, m.duration_ms) = new);
        self.history.push(Box::new(SetMarkerSpanCommand {
            guid: guid.to_string(),
            old,
            new,
        }));
    }

    /// `Alt`+drag on the bar drew a range: add it as one undo step. A drag too short to be a
    /// range adds a point.
    fn add_range(&mut self, start_ms: u64, end_ms: u64) -> Task<Message> {
        if self.file_workspace.markers().is_none() {
            return self.show_marker_list();
        }
        let (start_ms, end_ms) = (start_ms.min(end_ms), start_ms.max(end_ms));
        let mut marker = Marker::new(start_ms);
        marker.duration_ms = at_least_a_range(end_ms - start_ms);
        self.file_workspace
            .tag_list_mut()
            .add_marker(marker.clone());
        self.history.push(Box::new(AddMarkerCommand { marker }));
        Task::none()
    }

    /// `Shift+F2`: delete the marker under the playhead, saying what happened.
    fn delete_marker_at(&mut self, position_ms: u64) -> Task<Message> {
        if self.markers.is_editing() {
            return Task::none();
        }
        let guid = self
            .file_workspace
            .markers()
            .and_then(|markers| nearest(markers, position_ms))
            .and_then(|m| m.guid.clone());
        match guid {
            Some(guid) => {
                self.delete_marker(&guid);
                Self::notice("Marker deleted")
            }
            None => Self::notice("No marker here"),
        }
    }

    fn delete_marker(&mut self, guid: &str) {
        if self.markers.edit().is_some_and(|e| e.guid == guid) {
            self.markers.close();
        }
        if let Some(marker) = self.file_workspace.tag_list_mut().remove_marker(guid) {
            self.history.push(Box::new(DeleteMarkerCommand { marker }));
        }
    }

    /// `Shift+F1` / `Shift+F3`: jump to the previous / next marker, exactly onto it.
    fn jump_to_marker(&mut self, position_ms: u64, forward: bool) -> Task<Message> {
        let starts = self
            .file_workspace
            .markers()
            .unwrap_or_default()
            .iter()
            .map(|m| m.start_ms);
        let target = if forward {
            starts.filter(|&start| start > position_ms).min()
        } else {
            starts
                .filter(|&start| start + PREVIOUS_SLACK_MS <= position_ms)
                .max()
        };
        target.map_or_else(Task::none, seek_exact)
    }

    /// Show the marker list, in this update: a focus or scroll task started with it must find
    /// the list already there, which a `Task::done` would only show after them.
    fn show_marker_list(&mut self) -> Task<Message> {
        self.media_viewer
            .update(media_viewer::Message::Video(video::Message::ShowMarkerList))
            .map(Message::MediaViewer)
    }

    /// Open the row of the marker for editing, with its name focused, in the marker list.
    fn open_marker_row(&mut self, guid: String) -> Task<Message> {
        let Some(markers) = self.file_workspace.markers() else {
            return Task::none();
        };
        let Some(index) = markers.iter().position(|m| m.has_guid(&guid)) else {
            return Task::none();
        };
        let current = markers[index].name.clone();
        self.markers.open(guid, &current);
        let name = iced::widget::Id::new(markers::view::MARKER_NAME_INPUT_ID);
        let offset = markers::view::row_offset(markers, index.saturating_sub(1));
        Task::batch([
            self.show_marker_list(),
            scroll_marker_list_to(offset),
            operation::focus(name),
        ])
    }

    /// Keep the lit row (the last marker at or before the playhead) in view while the marker
    /// list is shown, except while a row is open for editing. Scrolls only when the lit row
    /// changed, so a list scrolled by hand is not fought on every tick.
    pub(super) fn follow_marker_list(&mut self, force: bool) -> Task<Message> {
        if !self.media_viewer.marker_list_shown() {
            self.markers.follow(None);
            return Task::none();
        }
        if self.markers.is_editing() {
            return Task::none();
        }
        let (Some(markers), Some(position_ms)) = (
            self.file_workspace.markers(),
            self.media_viewer.video_position_ms(),
        ) else {
            return Task::none();
        };
        let lit = markers::view::lit_index(markers, position_ms);
        if !self.markers.follow(lit) && !force {
            return Task::none();
        }
        scroll_marker_list_to(markers::view::row_offset(
            markers,
            lit.unwrap_or(0).saturating_sub(1),
        ))
    }

    /// The open file's markers were just read from it (another file was opened): remember their
    /// GUIDs, and show the markers whose write failed last time instead, if any.
    pub(super) fn markers_loaded(&mut self, id: FileId) {
        let read = self.file_workspace.markers().unwrap_or_default();
        self.known_marker_guids
            .extend(read.iter().filter_map(|m| m.guid.clone()));
        if self.file_workspace.markers().is_none() {
            return;
        }
        if let Some(unsaved) = self.unsaved_markers.get(&id).cloned() {
            self.file_workspace
                .tag_list_mut()
                .set_markers(Some(unsaved));
        }
    }

    /// Write the markers of the file `id` at `path` into it. A failed write keeps them for the
    /// next save and marks the file in the list.
    pub(super) fn save_markers(
        &mut self,
        id: FileId,
        path: &Path,
        markers: &[Marker],
    ) -> Task<Message> {
        match FileTagger::save_markers(path, markers, &self.known_marker_guids) {
            Ok(()) => {
                self.known_marker_guids
                    .extend(markers.iter().filter_map(|m| m.guid.clone()));
                self.unsaved_markers.remove(&id);
                Task::none()
            }
            Err(MarkersError::CannotHoldMarkers | MarkersError::Damaged) => {
                self.unsaved_markers.remove(&id);
                Task::none()
            }
            Err(MarkersError::WriteFailed(reason)) => {
                log::warn!("markers of {path:?} not saved, kept for the next save: {reason}");
                self.unsaved_markers.insert(id, markers.to_vec());
                Self::notice("Markers not saved: the file is read-only or in use")
            }
        }
    }

    /// Show a short note over the video's picture.
    pub(super) fn notice(text: &str) -> Task<Message> {
        Task::done(Message::MediaViewer(media_viewer::Message::Video(
            video::Message::ShowNotice(text.to_string()),
        )))
    }
}

/// `duration_ms`, or 0 (a point) when it is too short to be a range.
fn at_least_a_range(duration_ms: u64) -> u64 {
    if duration_ms < markers::MIN_RANGE_MS {
        0
    } else {
        duration_ms
    }
}

/// The marker nearest the playhead, if one is within [`MARKER_SNAP_MS`] of it.
fn nearest(markers: &[Marker], position_ms: u64) -> Option<&Marker> {
    markers
        .iter()
        .filter(|m| m.start_ms.abs_diff(position_ms) <= MARKER_SNAP_MS)
        .min_by_key(|m| m.start_ms.abs_diff(position_ms))
}

fn seek_exact(ms: u64) -> Task<Message> {
    Task::done(Message::MediaViewer(media_viewer::Message::Video(
        video::Message::SeekExact(ms),
    )))
}

/// Scroll the marker list to `y`: the top of the row before the one to show, so it keeps a
/// row of context above it (see [`markers::view::row_offset`]).
fn scroll_marker_list_to(y: f32) -> Task<Message> {
    let offset = iced::widget::scrollable::AbsoluteOffset {
        x: None,
        y: Some(y),
    };
    operation::scroll_to::<()>(iced::widget::Id::new(MARKER_LIST_SCROLLABLE_ID), offset).discard()
}
