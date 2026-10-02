//! "Describe with AI" on a marker of the open clip: a row's ✨ or `Ctrl+F2` sends one request per
//! marker in the background (`markers::describe`), several at once if asked. Its answer names an
//! unnamed marker and adds the description to its comment, as one undo step, and is saved and
//! journaled like an edit made by hand. Leaving the clip, deleting the marker or starting a
//! batch job stops the requests still on their way.
//!
//! A request reads the clip's frames through a GStreamer pipeline of its own, which holds the
//! file open, and clipscribe does not say when it is done reading: until a clip's last request
//! has come back, stopped or not, a save of that clip that would rename or write it waits, and so
//! does a batch job.

use frename_core::{marker_text_with_moment, FileId, FileSnapshot, SetMarkerTextCommand};
use iced::Task;

use super::FolderWorkspace;
use crate::features::batch;
use crate::features::folder_workspace::Message;
use crate::features::markers::{self, describe, MomentOutcome};

impl FolderWorkspace {
    /// Send the marker `guid` of the open clip to the AI, unless it is on its way already. With
    /// no key saved, nothing is sent: Settings opens where the key is set.
    pub(super) fn describe_marker(&mut self, guid: &str) -> Task<Message> {
        // A batch job is about to close the clip, or has.
        if self.batch.is_running() || self.batch.is_waiting_for_markers() {
            return Task::none();
        }
        let Some(file) = self.file_workspace.file() else {
            return Task::none();
        };
        let (id, path) = (file.id(), file.file_path().to_path_buf());
        let Some(marker) = self
            .file_workspace
            .tag_list()
            .marker(guid)
            .filter(|m| m.is_editable())
            .cloned()
        else {
            return Task::none();
        };
        if self.batch.actions().ai_key_missing() {
            return Self::no_key();
        }
        let Some((request, cancel)) = self.markers.start_describing(guid) else {
            return Task::none();
        };
        *self.marker_requests.entry(id).or_default() += 1;
        let (model, language) = self.batch.actions().ai_model_and_language();
        let at_s = describe::moment_s(&marker);
        let guid = guid.to_string();
        Task::future(async move {
            let outcome = tokio::task::spawn_blocking(move || {
                describe::describe_moment(&path, at_s, model, language, &cancel)
            })
            .await
            .unwrap_or_else(|e| {
                log::error!("ai: describing a marker failed: {e}");
                MomentOutcome::Failed(fl!("markers-ai-failed-unknown"))
            });
            Message::MarkerDescribed {
                file: id,
                guid,
                request,
                outcome,
            }
        })
    }

    /// `Ctrl+F2`: describe the marker the playhead is on (the one the list lights up and the
    /// progress bar labels), saying so when there is none. The marker list opens, so the request
    /// shows its progress and can be stopped there.
    pub(super) fn describe_marker_at(&mut self, position_ms: u64) -> Task<Message> {
        let lit = self.file_workspace.markers().and_then(|markers| {
            markers::view::lit_index(markers, position_ms).map(|i| markers[i].guid.clone())
        });
        match lit {
            Some(Some(guid)) => {
                let describe = self.describe_marker(&guid);
                if !self.markers.is_describing(&guid) {
                    return describe;
                }
                Task::batch([self.show_marker_list(), describe])
            }
            Some(None) => Self::notice(&fl!("markers-read-only-notice")),
            None => Self::notice(&fl!("markers-no-marker-notice")),
        }
    }

    /// A batch job is about to close the clip: stop its marker requests. Whether one still runs
    /// (it may be reading the clip's frames); the job then waits for it.
    pub(super) fn stop_marker_requests(&mut self) -> bool {
        self.markers.stop_all_describing();
        !self.marker_requests.is_empty()
    }

    /// Save the clip `id`, left with the edits in `snapshot`: at once, unless a marker request
    /// still reads it, then once the last one is back (see the module's doc). Closing frename
    /// does not wait: a save that fails keeps its edits in the recovery journal.
    pub(super) fn save_left_clip(&mut self, id: FileId, snapshot: FileSnapshot) -> Task<Message> {
        let still_open = self.file_workspace.file().is_some_and(|f| f.id() == id);
        if self.marker_requests.contains_key(&id) && !still_open && !self.closing {
            self.held_for_markers.push((id, snapshot));
            return Task::none();
        }
        self.apply_file_updated(id, snapshot)
    }

    /// Apply every save held for marker requests now (another folder is opened).
    pub(super) fn apply_held_for_markers(&mut self) -> Task<Message> {
        let held = std::mem::take(&mut self.held_for_markers);
        Task::batch(
            held.into_iter()
                .map(|(id, snapshot)| self.apply_file_updated(id, snapshot)),
        )
    }

    /// A request about the clip `file` came back: when it was its last, the saves held for it
    /// go ahead, and a batch job waiting for it starts. A clip opened again meanwhile keeps its
    /// held edits for its next save, like a save waiting for the video to unload.
    fn marker_request_done(&mut self, file: FileId) -> Task<Message> {
        match self.marker_requests.get_mut(&file) {
            Some(left) if *left > 1 => {
                *left -= 1;
                return Task::none();
            }
            Some(_) => {
                self.marker_requests.remove(&file);
            }
            None => {}
        }
        let (mine, others) = std::mem::take(&mut self.held_for_markers)
            .into_iter()
            .partition::<Vec<_>, _>(|(id, _)| *id == file);
        self.held_for_markers = others;
        let saved = if self.file_workspace.file().is_some_and(|f| f.id() == file) {
            self.pending_file_updates.extend(mine);
            Task::none()
        } else {
            Task::batch(
                mine.into_iter()
                    .map(|(id, snapshot)| self.apply_file_updated(id, snapshot)),
            )
        };
        // The run waited for this; it starts unless batch mode was left meanwhile.
        let batch = if self.marker_requests.is_empty() && self.batch.is_waiting_for_markers() {
            self.batch.set_waiting_for_markers(false);
            if self.batch.is_active() {
                self.start_batch()
            } else {
                Task::none()
            }
        } else {
            Task::none()
        };
        Task::batch([saved, batch])
    }

    /// A marker's request came back: fill the marker in, or say why not. The answer of a
    /// request that was stopped (or of an earlier one for the same marker), or of a clip that
    /// was left, is dropped.
    pub(super) fn marker_described(
        &mut self,
        file: FileId,
        guid: String,
        request: u64,
        outcome: MomentOutcome,
    ) -> Task<Message> {
        let done = self.marker_request_done(file);
        if !self.markers.finish_describing(&guid, request)
            || self.file_workspace.file().map(|f| f.id()) != Some(file)
        {
            return done;
        }
        let answered = match outcome {
            MomentOutcome::Described { name, description } => {
                self.apply_moment(guid, &name, &description)
            }
            // Said once, even when several requests find it out; the settings, which read the
            // key, read it again and pass it on, so later clicks send nothing.
            MomentOutcome::NoKey => {
                if self.markers.no_key_said() {
                    Task::none()
                } else {
                    Task::batch([
                        Self::no_key(),
                        Task::done(Message::Batch(batch::Message::Action(
                            batch::ActionMessage::ReadKeyState,
                        ))),
                    ])
                }
            }
            MomentOutcome::Cancelled => Task::none(),
            MomentOutcome::Failed(reason) => {
                Self::notice(&fl!("markers-ai-failed", reason = reason))
            }
        };
        Task::batch([done, answered])
    }

    /// Put an answer into the marker `guid`: one undo step, saved like an edit by hand.
    fn apply_moment(&mut self, guid: String, name: &str, description: &str) -> Task<Message> {
        // A name being typed in its row is committed first, as a step of its own.
        if self.markers.edit().is_some_and(|edit| edit.guid == guid) {
            self.close_marker_row();
        }
        // Removed meanwhile (an undo of adding it): the answer has nowhere to go.
        let Some(marker) = self.file_workspace.tag_list().marker(&guid).cloned() else {
            return Self::notice(&fl!("markers-ai-gone"));
        };
        let new = marker_text_with_moment(&marker, name, description);
        let old = (marker.name, marker.comment);
        if new == old {
            return Self::notice(&fl!("markers-ai-nothing-new"));
        }
        self.file_workspace
            .tag_list_mut()
            .update_marker(&guid, |m| (m.name, m.comment) = new.clone());
        self.history
            .push(Box::new(SetMarkerTextCommand { guid, old, new }));
        Task::batch([
            self.write_markers_to_comment_now(),
            Self::notice(&fl!("markers-ai-done")),
        ])
    }

    /// No key is saved: say so over the video and open Settings where it is set.
    fn no_key() -> Task<Message> {
        Task::batch([
            Self::notice(&fl!("markers-ai-no-key")),
            Task::done(Message::Batch(batch::Message::Action(
                batch::ActionMessage::OpenAiSettings,
            ))),
        ])
    }
}
