//! The search over the side list of the open clip (#170): it opens and closes the bar, narrows the
//! list to what matches, and steps the playhead from match to match. The text and the bar's
//! state are [`SideSearch`]; this module is what the workspace does with them, since the markers
//! it goes through live here.

use frename_core::matching_cues;
use iced::widget::operation;
use iced::Task;

use super::FolderWorkspace;
use crate::features::folder_workspace::Message;
use crate::features::markers::view::{
    filtered_row_offset, shown_markers, MARKER_LIST_SCROLLABLE_ID,
};
use crate::features::media_viewer;
use crate::features::media_viewer::video::{
    self, step, step_cue, Overlay, SearchMessage, SIDE_SEARCH_INPUT_ID,
};

impl FolderWorkspace {
    /// Apply a message of the search bar or its keys.
    pub(super) fn handle_side_search(&mut self, msg: SearchMessage) -> Task<Message> {
        let was_filtering = self.side_search.is_filtering();
        let task = match msg {
            SearchMessage::Open => {
                // Only a clip with a video has a side list to search.
                if self.media_viewer.video_position_ms().is_none() {
                    return Task::none();
                }
                self.side_search.open();
                let show = self.media_viewer.shown_overlay() == Overlay::Closed;
                let show = show.then(|| self.show_a_list());
                Task::batch([
                    show.unwrap_or_else(Task::none),
                    operation::focus(iced::widget::Id::new(SIDE_SEARCH_INPUT_ID)),
                ])
            }
            SearchMessage::Input(text) => {
                self.side_search.set_query(text);
                Task::none()
            }
            SearchMessage::Clear => {
                self.side_search.set_query(String::new());
                operation::focus(iced::widget::Id::new(SIDE_SEARCH_INPUT_ID))
            }
            SearchMessage::Submit => return self.step_search(!self.modifiers.shift()),
            SearchMessage::Step(forward) => return self.step_search(forward),
            SearchMessage::Close => {
                self.side_search.close();
                Task::none()
            }
        };
        Task::batch([task, self.sync_list_filter(was_filtering)])
    }

    /// Whether the bar is on screen: open, over a side list of the open clip's video.
    fn side_search_visible(&self) -> bool {
        self.side_search.is_open()
            && self.media_viewer.video_position_ms().is_some()
            && self.media_viewer.shown_overlay() != Overlay::Closed
    }

    /// `Esc` in the search: the text first, then the bar. `None` when the bar is not on screen,
    /// so the Esc goes on to what it would cancel without it (a rename, fullscreen, the batch).
    pub(super) fn escape_side_search(&mut self) -> Option<Task<Message>> {
        if !self.side_search_visible() {
            return None;
        }
        let msg = if self.side_search.query().is_empty() {
            SearchMessage::Close
        } else {
            SearchMessage::Clear
        };
        Some(self.handle_side_search(msg))
    }

    /// Another clip was opened: its list starts unfiltered. The bar stays open or closed as it was.
    pub(super) fn reset_side_search(&mut self) -> Task<Message> {
        let was_filtering = self.side_search.is_filtering();
        self.side_search.set_query(String::new());
        self.sync_list_filter(was_filtering)
    }

    /// Show the subtitle list if the clip has subtitles, the marker list otherwise.
    fn show_a_list(&mut self) -> Task<Message> {
        if self.media_viewer.has_subtitles() {
            self.media_viewer
                .update(media_viewer::Message::Video(video::Message::ShowOverlay(
                    Overlay::Subtitles,
                )))
                .map(Message::MediaViewer)
        } else {
            self.show_marker_list()
        }
    }

    /// Tell the player whether its list is narrowed, when that changed.
    fn sync_list_filter(&mut self, was_filtering: bool) -> Task<Message> {
        let filtering = self.side_search.is_filtering();
        if filtering == was_filtering {
            return Task::none();
        }
        let player = self
            .media_viewer
            .update(media_viewer::Message::Video(
                video::Message::SetListFiltered(filtering),
            ))
            .map(Message::MediaViewer);
        // The list is whole again: back to following the playhead.
        let follow = if filtering {
            Task::none()
        } else {
            self.follow_marker_list(true)
        };
        Task::batch([player, follow])
    }

    /// The match to go to from the playhead: in the shown list, the next (or previous) one.
    pub(super) fn search_match(&self, forward: bool) -> Option<SearchMatch> {
        if !self.side_search.is_filtering() {
            return None;
        }
        let position_ms = self.media_viewer.video_position_ms()?;
        let words = self.side_search.words();
        let current = self.side_search.current();
        let overlay = match self.media_viewer.shown_overlay() {
            // The tab of a list the clip does not have shows the markers.
            Overlay::Subtitles if self.media_viewer.subtitles().is_none() => Overlay::Markers,
            overlay => overlay,
        };
        match overlay {
            Overlay::Subtitles => {
                let subtitles = self.media_viewer.subtitles()?;
                let shown = matching_cues(subtitles, words);
                let index = step_cue(subtitles.cues(), &shown, position_ms, forward, current)?;
                let start = subtitles.cues()[index].start;
                Some(SearchMatch {
                    overlay: Overlay::Subtitles,
                    index,
                    start_ms: u64::try_from(start.as_millis()).unwrap_or(u64::MAX),
                    shown,
                })
            }
            Overlay::Markers => {
                let markers = self.file_workspace.markers()?;
                let shown = shown_markers(markers, &self.markers, words);
                let starts: Vec<(usize, u64)> =
                    shown.iter().map(|&i| (i, markers[i].start_ms)).collect();
                let index = step(&starts, position_ms, forward, current)?;
                Some(SearchMatch {
                    overlay: Overlay::Markers,
                    index,
                    start_ms: markers[index].start_ms,
                    shown,
                })
            }
            Overlay::Closed => None,
        }
    }

    /// Go to the next (or previous) match of the shown list from the playhead, and bring its row
    /// into view.
    fn step_search(&mut self, forward: bool) -> Task<Message> {
        let Some(found) = self.search_match(forward) else {
            return Task::none();
        };
        self.side_search.set_current(Some(found.index));
        let cue_y = (found.overlay == Overlay::Subtitles).then(|| {
            self.media_viewer.subtitles().map_or(0.0, |subtitles| {
                video::view::filtered_cue_offset(subtitles, &found.shown, found.index)
            })
        });
        let mut video = |message| {
            self.media_viewer
                .update(media_viewer::Message::Video(message))
                .map(Message::MediaViewer)
        };
        if let Some(y) = cue_y {
            return Task::batch([
                video(video::Message::SeekToCue(found.index)),
                video(video::Message::ScrollCueListTo(y)),
            ]);
        }
        let y = filtered_row_offset(self.markers.row_heights(), &found.shown, found.index);
        Task::batch([
            video(video::Message::SeekExact(found.start_ms)),
            operation::scroll_to::<()>(
                iced::widget::Id::new(MARKER_LIST_SCROLLABLE_ID),
                iced::widget::scrollable::AbsoluteOffset {
                    x: None,
                    y: Some(y),
                },
            )
            .discard(),
        ])
    }
}

/// Where a step of the search goes.
pub(super) struct SearchMatch {
    pub(super) overlay: Overlay,
    /// Its index in the whole list.
    pub(super) index: usize,
    pub(super) start_ms: u64,
    /// Every match of the list, in order: what the list shows.
    pub(super) shown: Vec<usize>,
}
