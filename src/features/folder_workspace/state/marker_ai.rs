//! "Describe with AI" on a marker of the open clip: a row's ✨ or `Ctrl+F2` sends one request per
//! marker in the background (`markers::describe`), several at once if asked. Its answer names an
//! unnamed marker and adds the description to its comment, as one undo step, and is saved and
//! journaled like an edit made by hand. Leaving the clip stops the requests still on their way.

use frename_core::{marker_text_with_moment, FileId, SetMarkerTextCommand};
use iced::Task;

use super::FolderWorkspace;
use crate::features::batch;
use crate::features::folder_workspace::Message;
use crate::features::markers::{self, describe, MomentOutcome};

impl FolderWorkspace {
    /// Send the marker `guid` of the open clip to the AI, unless it is on its way already.
    pub(super) fn describe_marker(&mut self, guid: &str) -> Task<Message> {
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
        let Some(cancel) = self.markers.start_describing(guid) else {
            return Task::none();
        };
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
                outcome,
            }
        })
    }

    /// `Ctrl+F2`: describe the marker the playhead is on (the one the list lights up and the
    /// progress bar labels), saying so when there is none.
    pub(super) fn describe_marker_at(&mut self, position_ms: u64) -> Task<Message> {
        let lit = self.file_workspace.markers().and_then(|markers| {
            markers::view::lit_index(markers, position_ms).map(|i| markers[i].guid.clone())
        });
        match lit {
            Some(Some(guid)) => self.describe_marker(&guid),
            Some(None) => Self::notice("That marker is read-only"),
            None => Self::notice(&fl!("markers-ai-no-marker")),
        }
    }

    /// A marker's request came back: fill the marker in, or say why not. The answer of a
    /// request that was stopped, or of a clip that was left, is dropped.
    pub(super) fn marker_described(
        &mut self,
        file: FileId,
        guid: String,
        outcome: MomentOutcome,
    ) -> Task<Message> {
        if !self.markers.finish_describing(&guid)
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
                let Some(marker) = self.file_workspace.tag_list().marker(&guid).cloned() else {
                    return Task::none();
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
            // The settings page where the key is set, with the reason over the video.
            MomentOutcome::NoKey => Task::batch([
                Self::notice(&fl!("markers-ai-no-key")),
                Task::done(Message::Batch(batch::Message::Action(
                    batch::ActionMessage::OpenAiSettings,
                ))),
            ]),
            MomentOutcome::Cancelled => Task::none(),
            MomentOutcome::Failed(reason) => {
                Self::notice(&fl!("markers-ai-failed", reason = reason))
            }
        }
    }
}
