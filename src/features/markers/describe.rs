//! "Describe with AI" on a marker: one small request (clipscribe's `describe_moment`) with the
//! frames and subtitle lines around the marker's moment, which comes back with a short name and
//! a one-to-two sentence description. It uses the key, model and language of the batch action
//! "Describe with AI". Blocking: runs on a worker thread.

use std::path::Path;
use std::sync::atomic::AtomicBool;

use clipscribe::{
    self as describe, AiError, FrameSampling, Model, MomentsMode, SummaryLanguage, MOMENT_WINDOW_S,
};
use frename_core::ai::key;
use frename_core::{FileTagger, Marker};

/// What a request for one marker came back with.
#[derive(Debug, Clone, PartialEq)]
pub enum MomentOutcome {
    Described {
        name: String,
        description: String,
    },
    /// No Anthropic key is saved.
    NoKey,
    /// Stopped by the editor, or the clip was left.
    Cancelled,
    /// Why it failed, for the notice over the video.
    Failed(String),
}

/// The time of `marker` the request is about, in seconds: a point marker's own, the middle of
/// a range.
pub fn moment_s(marker: &Marker) -> f64 {
    (marker.start_ms + marker.duration_ms / 2) as f64 / 1000.0
}

/// Name and describe the moment at `at_s` of the clip at `path`. Blocking.
pub fn describe_moment(
    path: &Path,
    at_s: f64,
    model: Model,
    language: SummaryLanguage,
    cancel: &AtomicBool,
) -> MomentOutcome {
    let Some(api_key) = key::read_key(key::ApiKey::Anthropic) else {
        return MomentOutcome::NoKey;
    };
    // A debug build renames in memory only: the clip and its subtitles are read by the name on disk.
    let on_disk = FileTagger::disk_path(path);
    let subtitles = describe::srt::load_for(&on_disk).unwrap_or_else(|e| {
        log::warn!("ai: subtitles of {} not read: {e}", path.display());
        Vec::new()
    });
    let options = describe::Options {
        api_key,
        model,
        language,
        frame_sampling: FrameSampling::KeyFrames,
        moments: MomentsMode::Important,
    };
    match describe::describe_moment(
        &on_disk,
        at_s,
        MOMENT_WINDOW_S,
        &subtitles,
        &options,
        cancel,
    ) {
        Ok(described) => {
            log::info!(
                "ai: described the moment at {at_s:.2}s of {} ({} in / {} out tokens)",
                path.display(),
                described.usage.input_tokens,
                described.usage.output_tokens
            );
            MomentOutcome::Described {
                name: described.moment.name,
                description: described.moment.description,
            }
        }
        Err(describe::Error::Cancelled) => MomentOutcome::Cancelled,
        Err(describe::Error::Unreadable(e)) => {
            log::warn!("ai: cannot read {}: {e}", path.display());
            MomentOutcome::Failed(fl!("batch-ai-fail-unreadable"))
        }
        Err(describe::Error::Ai(AiError::Cancelled)) => MomentOutcome::Cancelled,
        Err(describe::Error::Ai(e)) => {
            log::warn!("ai: request failed: {e:?}");
            MomentOutcome::Failed(e.reason())
        }
        Err(describe::Error::BadAnswer { reason, .. }) => MomentOutcome::Failed(reason),
        Err(describe::Error::TooLong(_)) => MomentOutcome::Failed(fl!("batch-ai-fail-unreadable")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_range_is_described_at_its_middle() {
        let point = Marker::new(41_000);
        assert_eq!(moment_s(&point), 41.0);
        let mut range = Marker::new(10_000);
        range.duration_ms = 6_000;
        assert_eq!(moment_s(&range), 13.0);
    }
}
