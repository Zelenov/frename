//! "Describe with AI" on a marker: one small request (clipscribe's `describe_moment`) with the
//! frames and subtitle lines around the marker's moment, which comes back with a short name and
//! a one-to-two sentence description. It is set up like a request of the batch action "Describe
//! with AI" (`describe_ai::Request`), with the same key, model and language, and its failures
//! read the same. Blocking: runs on a worker thread.

use std::path::Path;
use std::sync::atomic::AtomicBool;

use clipscribe::{self as describe, Model, SummaryLanguage, MOMENT_WINDOW_S};
use frename_core::Marker;

use crate::features::batch::describe_ai;

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
    let Some(request) = describe_ai::Request::for_clip(path, model, language) else {
        return MomentOutcome::NoKey;
    };
    match describe::describe_moment(
        &request.on_disk,
        at_s,
        MOMENT_WINDOW_S,
        &request.subtitles,
        &request.options,
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
        Err(describe::Error::Cancelled | describe::Error::Ai(clipscribe::AiError::Cancelled)) => {
            MomentOutcome::Cancelled
        }
        Err(e) => {
            log::warn!("ai: describing the moment at {at_s:.2}s failed: {e:?}");
            MomentOutcome::Failed(describe_ai::failure_reason(&e, path))
        }
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
