//! "Describe with AI" on a marker of the open clip: a row's ✨ or `Ctrl+F2` sends one request per
//! marker in the background (`markers::describe`), several at once if asked; "Describe N
//! unnamed" over the list asks for every marker without a name and sends them a few at a time
//! (`MAX_DESCRIBING_AT_ONCE`), the others waiting their turn in a queue. Its answer names an
//! unnamed marker and adds the description to its comment, as one undo step, and is saved and
//! journaled like an edit made by hand. Leaving the clip, deleting the marker or starting a
//! batch job stops the requests still on their way.
//!
//! A request reads the clip's frames through a GStreamer pipeline of its own, which holds the
//! file open, and clipscribe says when it is done reading (`Stage::Released`, before the HTTP
//! request): a batch job waits until every request has let go of the clip or come back, stopped
//! or not. Leaving the clip does not wait: a save that meets a request still reading the file
//! fails like any save of a file in use (see `apply_file_updated`); once it has released the
//! clip, it does not.

use frename_core::{marker_text_with_moment, FileId, SetMarkerTextCommand};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use iced::futures::SinkExt;
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
        let editable = self
            .file_workspace
            .tag_list()
            .marker(guid)
            .is_some_and(|m| m.is_editable());
        if !editable || self.file_workspace.file().is_none() {
            return Task::none();
        }
        if self.batch.actions().ai_key_missing() {
            return Self::no_key();
        }
        let Some((request, cancel)) = self.markers.start_describing(guid) else {
            return Task::none();
        };
        self.send_request(guid.to_string(), request, cancel)
    }

    /// Send the request just started for the marker `guid` (see `MarkersState::start_describing`)
    /// on a worker thread. `marker_requests` counts it until its answer is back, stopped or not.
    fn send_request(
        &mut self,
        guid: String,
        request: u64,
        cancel: Arc<AtomicBool>,
    ) -> Task<Message> {
        let marker = self.file_workspace.tag_list().marker(&guid).cloned();
        let (Some(file), Some(marker)) = (self.file_workspace.file(), marker) else {
            self.markers.finish_describing(&guid, request);
            return Task::none();
        };
        let (id, path) = (file.id(), file.file_path().to_path_buf());
        self.marker_requests += 1;
        self.marker_readers.insert((guid.clone(), request));
        let (model, language) = self.batch.actions().ai_model_and_language();
        let at_s = describe::moment_s(&marker);
        let stream = iced::stream::channel(4, async move |mut output| {
            let mut released = output.clone();
            let released_guid = guid.clone();
            let outcome = tokio::task::spawn_blocking(move || {
                describe::describe_moment(&path, at_s, model, language, &cancel, || {
                    // Awaited by nobody: the answer follows, and marks the clip released too.
                    let _ = released.try_send(Message::MarkerRequestReleased {
                        guid: released_guid.clone(),
                        request,
                    });
                })
            })
            .await
            .unwrap_or_else(|e| {
                log::error!("ai: describing a marker failed: {e}");
                MomentOutcome::Failed(fl!("markers-ai-failed-unknown"))
            });
            // Awaited, unlike the release: a full buffer must not lose the answer.
            let _ = output
                .send(Message::MarkerDescribed {
                    file: id,
                    guid,
                    request,
                    outcome,
                })
                .await;
        });
        Task::run(stream, |message| message)
    }

    /// "Describe N unnamed": queue every marker of the open clip that has no name and is not on
    /// its way, and send the first few. A name typed meanwhile is kept: that marker is left
    /// out when its turn comes. With no key saved, nothing is queued: Settings opens where the
    /// key is set. More than `describe::CONFIRM_ABOVE` markers are not sent by the button
    /// alone: it shows the price and asks first, and the answer comes as `confirmed`.
    pub(super) fn describe_unnamed_markers(&mut self, confirmed: bool) -> Task<Message> {
        self.markers.set_confirming_describe_all(false);
        if self.batch.is_running() || self.batch.is_waiting_for_markers() {
            return Task::none();
        }
        let guids = self.unnamed_marker_guids();
        if guids.is_empty() {
            return Task::none();
        }
        if self.batch.actions().ai_key_missing() {
            return Self::no_key();
        }
        if !confirmed && markers::describe::needs_confirmation(guids.len()) {
            self.markers.set_confirming_describe_all(true);
            return Task::none();
        }
        self.markers.queue_describing(guids);
        self.send_waiting_markers()
    }

    /// What one marker request is expected to cost with the model set for Describe with AI, in
    /// US dollars; `None` when that model has no price.
    pub fn marker_price_usd(&self) -> Option<f64> {
        markers::describe::moment_price_usd(self.batch.actions().ai_model_and_language().0)
    }

    /// The markers "Describe N unnamed" would send now (see `MarkersState::unnamed_guids`).
    pub fn unnamed_marker_guids(&self) -> Vec<String> {
        self.markers
            .unnamed_guids(self.file_workspace.markers().unwrap_or_default())
    }

    /// Send the markers waiting for their turn while fewer than `MAX_DESCRIBING_AT_ONCE`
    /// requests are really alive (`marker_requests`: a stopped request counts until its answer
    /// is back, as it still reads the clip). Called when the run starts and whenever an answer
    /// comes back, which is when a slot frees. One that got a name (or went) while it waited
    /// is skipped.
    pub(super) fn send_waiting_markers(&mut self) -> Task<Message> {
        if self.markers.waiting_count() == 0 {
            return Task::none();
        }
        // A batch job is about to close the clip, or has.
        if self.batch.is_running() || self.batch.is_waiting_for_markers() {
            self.markers.clear_waiting();
            return Task::none();
        }
        // The key was removed meanwhile: the rest would find out the same.
        if self.batch.actions().ai_key_missing() {
            self.markers.clear_waiting();
            return Self::no_key();
        }
        let mut tasks = Vec::new();
        loop {
            let tag_list = self.file_workspace.tag_list();
            let next = self
                .markers
                .start_next_waiting(self.marker_requests, |guid| {
                    tag_list.marker(guid).is_some_and(|m| m.is_unnamed())
                });
            let Some((guid, request, cancel)) = next else {
                break;
            };
            tasks.push(self.send_request(guid, request, cancel));
        }
        Task::batch(tasks)
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
        !self.marker_readers.is_empty()
    }

    /// Whether a marker request may still be reading the clip's frames (it holds the file open
    /// until `Stage::Released` or its end).
    pub(super) fn marker_reading_clip(&self) -> bool {
        !self.marker_readers.is_empty()
    }

    /// A request has let go of the clip (before its answer): when it was the last, a batch job
    /// waiting for them starts. Said again by the answer, or after it, it changes nothing.
    pub(super) fn marker_request_released(&mut self, guid: String, request: u64) -> Task<Message> {
        self.marker_readers.remove(&(guid, request));
        self.start_batch_if_markers_let_go()
    }

    /// A request came back, stopped or not: it holds the clip no more, if it still did.
    fn marker_request_done(&mut self, guid: &str, request: u64) -> Task<Message> {
        self.marker_requests = self.marker_requests.saturating_sub(1);
        self.marker_readers.remove(&(guid.to_string(), request));
        self.start_batch_if_markers_let_go()
    }

    /// A batch job waiting for the marker requests to let go of the clip starts once none does.
    fn start_batch_if_markers_let_go(&mut self) -> Task<Message> {
        if !self.marker_readers.is_empty() || !self.batch.is_waiting_for_markers() {
            return Task::none();
        }
        // Leaving batch mode ends the wait, so the job is still wanted.
        self.batch.update(batch::Message::StopWaiting);
        self.start_batch()
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
        let done = self.marker_request_done(&guid, request);
        if !self.markers.finish_describing(&guid, request)
            || self.file_workspace.file().map(|f| f.id()) != Some(file)
        {
            // A slot is free, or the clip was left and nothing is queued any more.
            return Task::batch([done, self.send_waiting_markers()]);
        }
        // A request that came back without a description (no key, an error) means the ones
        // still waiting would not get one either: they stay unnamed, and the notice is said once.
        if matches!(outcome, MomentOutcome::NoKey | MomentOutcome::Failed(_)) {
            self.markers.clear_waiting();
        }
        let answered = match outcome {
            MomentOutcome::Described { name, description } => {
                self.apply_moment(guid, &name, &description)
            }
            // Said once, even when several requests find it out; the settings, which read the
            // key, read it again and pass it on, so later clicks send nothing.
            MomentOutcome::NoKey => {
                if self.markers.take_first_no_key() {
                    Task::batch([
                        Self::no_key(),
                        Task::done(Message::Batch(batch::Message::Action(
                            batch::ActionMessage::ReadKeyState,
                        ))),
                    ])
                } else {
                    Task::none()
                }
            }
            MomentOutcome::Cancelled => Task::none(),
            // Said once, even when several requests of a run fail.
            MomentOutcome::Failed(reason) => {
                if self.markers.failed_said() {
                    Task::none()
                } else {
                    Self::notice(&fl!("markers-ai-failed", reason = reason))
                }
            }
        };
        Task::batch([done, answered, self.send_waiting_markers()])
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
