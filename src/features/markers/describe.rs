//! "Describe with AI" on a marker: one small request (clipscribe's `describe_moment`) with the
//! frames and subtitle lines around the marker's moment, which comes back with a short name and
//! a one-to-two sentence description. It is set up like a request of the batch action "Describe
//! with AI" (`describe_ai::Request`), with the same key, model and language, and its failures
//! read the same. Blocking: runs on a worker thread.

use std::path::Path;
use std::sync::atomic::AtomicBool;

use clipscribe::{self as describe, Model, Stage, SummaryLanguage, MOMENT_WINDOW_S};
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

/// "Describe N unnamed" asks before sending when more markers than this would go out.
pub const CONFIRM_ABOVE: usize = 10;

/// Frames one marker request carries: the moment and a window's length each side of it
/// (`Clip::sample_moment`).
const MOMENT_FRAMES: u64 = 3;

/// Whether "Describe N unnamed" asks first for `count` markers.
pub fn needs_confirmation(count: usize) -> bool {
    count > CONFIRM_ABOVE
}

/// The tokens one marker request is expected to use with `model`: clipscribe's estimate for a
/// clip (its instructions and a typical answer, plus the frames of a clip of this length), with
/// the frames replaced by the [`MOMENT_FRAMES`] a moment request sends. Nearby subtitles are
/// not counted: they add a few lines at most. The answer is clipscribe's typical answer for a
/// whole clip (thinking included), longer than a name and a sentence or two: the price runs
/// high, an upper-ish estimate.
pub fn moment_usage(model: Model) -> clipscribe::AiUsage {
    let base = describe::estimate_usage(model, 0.0, 0);
    let (w, h) = describe::frame_size(1920, 1080);
    let frame = describe::frame_tokens(w, h);
    let base_frames = describe::frame_count(0.0) as u64;
    clipscribe::AiUsage {
        input_tokens: base.input_tokens - base_frames * frame + MOMENT_FRAMES * frame,
        output_tokens: base.output_tokens,
    }
}

/// What one marker request is expected to cost with `model`, in US dollars; `None` when the
/// model has no usable price (nothing is better than a wrong number).
pub fn moment_price_usd(model: Model) -> Option<f64> {
    let usd = model.cost_usd(moment_usage(model));
    (usd.is_finite() && usd > 0.0).then_some(usd)
}

/// What `count` marker requests cost at `per_marker_usd` each.
pub fn run_price_usd(count: usize, per_marker_usd: f64) -> f64 {
    count as f64 * per_marker_usd
}

/// The time of `marker` the request is about, in seconds: a point marker's own, the middle of
/// a range.
pub fn moment_s(marker: &Marker) -> f64 {
    (marker.start_ms + marker.duration_ms / 2) as f64 / 1000.0
}

/// Name and describe the moment at `at_s` of the clip at `path`. Blocking. `on_released` is
/// called once the frames are read and the clip is let go of, before the request is sent (never,
/// when the request ends before that).
pub fn describe_moment(
    path: &Path,
    at_s: f64,
    model: Model,
    language: SummaryLanguage,
    cancel: &AtomicBool,
    mut on_released: impl FnMut(),
) -> MomentOutcome {
    // One moment: clipscribe's `describe_moment` does not use the moments mode.
    let Some(request) =
        describe_ai::Request::for_clip(path, model, language, describe::MomentsMode::default())
    else {
        return MomentOutcome::NoKey;
    };
    match describe::describe_moment_notifying(
        &request.on_disk,
        at_s,
        MOMENT_WINDOW_S,
        &request.subtitles,
        &request.options,
        cancel,
        |stage| {
            // A stage this version does not know is not the clip being let go of.
            if matches!(stage, Stage::Released) {
                on_released();
            }
        },
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
            // `failure_reason` logs an unreadable clip itself.
            if !matches!(e, describe::Error::Unreadable(_)) {
                log::warn!("ai: describing the moment at {at_s:.2}s failed: {e:?}");
            }
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

    #[test]
    fn a_run_asks_first_only_above_ten_markers() {
        assert!(!needs_confirmation(0));
        assert!(!needs_confirmation(10));
        assert!(needs_confirmation(11));
        assert!(needs_confirmation(40));
    }

    #[test]
    fn a_run_costs_the_count_times_one_request() {
        assert_eq!(run_price_usd(0, 0.01), 0.0);
        assert!((run_price_usd(40, 0.01) - 0.4).abs() < 1e-12);
        assert!((run_price_usd(11, 0.02) - 0.22).abs() < 1e-12);
    }

    #[test]
    fn a_marker_request_is_priced_by_three_frames_and_the_models_prices() {
        let (w, h) = describe::frame_size(1920, 1080);
        for model in clipscribe::MODELS {
            let usage = moment_usage(model);
            let frames = 3 * describe::frame_tokens(w, h);
            assert!(
                usage.input_tokens > frames,
                "{}: frames and instructions",
                model.id
            );
            assert_eq!(usage.output_tokens, model.answer_tokens, "{}", model.id);
            // A clip with three frames is estimated with the same input.
            assert_eq!(describe::frame_count(5.0), 3);
            let clip = describe::estimate_usage(model, 5.0, 0);
            assert_eq!(usage.input_tokens, clip.input_tokens, "{}", model.id);
            let price = moment_price_usd(model).expect("every model is priced");
            assert!(price > 0.0 && price < 1.0, "{}: {price}", model.id);
        }
    }

    #[test]
    fn a_haiku_marker_request_costs_about_four_tenths_of_a_cent() {
        let haiku = Model::from_id("claude-haiku-4-5");
        let usage = moment_usage(haiku);
        // 3 frames of 19 x 11 tokens, and 600 tokens of instructions; a typical 600-token answer.
        assert_eq!(
            (usage.input_tokens, usage.output_tokens),
            (3 * 209 + 600, 600)
        );
        let price = moment_price_usd(haiku).expect("priced");
        assert!((price - 0.004227).abs() < 1e-9, "{price}");
    }

    #[test]
    fn a_model_without_a_price_has_no_estimate() {
        let mut model = clipscribe::MODELS[0];
        model.input_usd_per_mtok = 0.0;
        model.output_usd_per_mtok = 0.0;
        assert_eq!(moment_price_usd(model), None);
        model.input_usd_per_mtok = f64::NAN;
        assert_eq!(moment_price_usd(model), None);
    }
}
