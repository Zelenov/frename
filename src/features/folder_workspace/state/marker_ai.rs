//! "Describe with AI" on a marker of the open clip: a row's ✨ or `Ctrl+F2` sends one request per
//! marker in the background (`markers::describe`), several at once if asked. Its answer names an
//! unnamed marker and adds the description to its comment, as one undo step, and is saved and
//! journaled like an edit made by hand. Leaving the clip, deleting the marker or starting a
//! batch job stops the requests still on their way; a batch job waits until the stopped ones
//! have let go of the clip.

use frename_core::ai::key::KeyState;
use frename_core::{marker_text_with_moment, FileId, SetMarkerTextCommand};
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
        if self.batch.is_running() || self.batch_waits_for_markers {
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
        self.marker_requests += 1;
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
            None => Self::notice(&fl!("markers-ai-no-marker")),
        }
    }

    /// A batch job is about to close the clip: stop its marker requests. Whether one still runs
    /// (it may be reading the clip's frames, which a rename or write would fail on); the job
    /// then waits for it (see [`Self::marker_described`]).
    pub(super) fn stop_marker_requests(&mut self) -> bool {
        self.markers.stop_all_describing();
        self.marker_requests > 0
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
        self.marker_requests = self.marker_requests.saturating_sub(1);
        if self.marker_requests == 0 && std::mem::take(&mut self.batch_waits_for_markers) {
            return self.start_batch();
        }
        if !self.markers.finish_describing(&guid, request)
            || self.file_workspace.file().map(|f| f.id()) != Some(file)
        {
            return Task::none();
        }
        match outcome {
            MomentOutcome::Described { name, description } => {
                // A name being typed in its row is a step of its own, first.
                if self.markers.edit().is_some_and(|edit| edit.guid == guid) {
                    self.close_marker_row();
                }
                // Removed meanwhile (an undo of adding it): the answer has nowhere to go.
                let Some(marker) = self.file_workspace.tag_list().marker(&guid).cloned() else {
                    return Self::notice(&fl!("markers-ai-gone"));
                };
                let new = marker_text_with_moment(&marker, &name, &description);
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
            // Said once: later answers of requests sent before it was known find it known.
            MomentOutcome::NoKey => {
                if self.batch.actions().ai_key_missing() {
                    return Task::none();
                }
                self.batch
                    .update(batch::Message::Action(batch::ActionMessage::DescribeAi(
                        batch::describe_ai::Message::KeyState(KeyState::Missing),
                    )));
                Self::no_key()
            }
            MomentOutcome::Cancelled => Task::none(),
            MomentOutcome::Failed(reason) => {
                Self::notice(&fl!("markers-ai-failed", reason = reason))
            }
        }
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
