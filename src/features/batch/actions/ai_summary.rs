//! "Summarize subtitles with AI": sends each checked video's subtitles (`.srt`) to Claude and
//! writes a short summary of what happens into the video's comment, in its AI block (see
//! [`frename_core::ai`]). The editor's own text is never changed; a re-run replaces only the
//! block. The cost is estimated before anything is sent, and the run needs that estimate.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use frename_core::ai::{
    self, AiError, AiModel, AnthropicProvider, SummarizeError, SummarizeOutcome, SummaryEstimate,
    SummaryLanguage,
};
use iced::widget::{button, checkbox, column, text};
use iced::Element;

use super::super::{ItemResult, ItemStatus};
use super::ActionMessage;
use crate::theme;

pub const LABEL: &str = "Summarize subtitles with AI";

/// What a run does, fixed when it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Job {
    pub model: AiModel,
    pub language: SummaryLanguage,
    /// Summarize videos that already have an AI summary again.
    pub redo: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    /// Redo videos that already have an AI summary.
    SetRedo(bool),
    /// Estimate the cost for the checked files. Started by the workspace, which knows their
    /// paths (see [`Options::pending_estimate`] and [`estimate`]).
    Estimate,
    /// The estimate numbered `request` is ready.
    Estimated(u64, SummaryEstimate),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EstimateState {
    None,
    /// Being worked out; the number tells a stale answer from the current one.
    Running(u64),
    Ready(SummaryEstimate),
}

#[derive(Debug, Clone)]
pub struct Options {
    redo: bool,
    estimate: EstimateState,
    /// Number of the last estimate asked for.
    requests: u64,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            redo: false,
            estimate: EstimateState::None,
            requests: 0,
        }
    }
}

impl Options {
    pub fn update(&mut self, message: Message) {
        match message {
            Message::SetRedo(redo) => {
                self.redo = redo;
                self.estimate = EstimateState::None;
            }
            Message::Estimate => {
                self.requests += 1;
                self.estimate = EstimateState::Running(self.requests);
            }
            Message::Estimated(request, estimate) => {
                if self.estimate == EstimateState::Running(request) {
                    self.estimate = EstimateState::Ready(estimate);
                }
            }
        }
    }

    /// The estimate counted other files: it has to be worked out again.
    pub fn checks_changed(&mut self) {
        self.estimate = EstimateState::None;
    }

    /// The estimate being worked out, as its number and the run it is for.
    pub fn pending_estimate(&self) -> Option<(u64, Job)> {
        match self.estimate {
            EstimateState::Running(request) => Some((request, self.job())),
            _ => None,
        }
    }

    fn job(&self) -> Job {
        let settings = ai::ai_settings();
        Job {
            model: settings.model,
            language: settings.language,
            redo: self.redo,
        }
    }

    /// Runs with an API key, once the estimate is shown and has videos to send.
    pub fn operation(&self) -> Option<super::Operation> {
        let ready = matches!(self.estimate, EstimateState::Ready(e) if e.videos > 0);
        (ready && ai::ai_settings().api_key().is_some())
            .then(|| super::Operation::AiSummary(self.job()))
    }

    pub fn view(&self) -> Element<'_, ActionMessage> {
        let settings = ai::ai_settings();
        let muted = |line: String| text(line).size(12).color(theme::TEXT_MUTED);

        // Stacked, not side by side: the options column is narrow.
        let mut options = column![
            text(format!(
                "{} · summaries in: {}",
                settings.model.label(),
                settings.language.label()
            ))
            .size(13),
            button(text("AI settings…").size(12))
                .on_press(ActionMessage::OpenSettings)
                .padding([3, 10])
                .style(theme::icon_button_style(true)),
        ]
        .spacing(10);
        if settings.api_key().is_none() {
            options = options.push(
                text("Set an Anthropic API key in the settings to run this.")
                    .size(12)
                    .color(theme::ERROR),
            );
        }
        options = options.push(
            checkbox(self.redo)
                .label("Redo videos that already have an AI summary")
                .text_size(13)
                .on_toggle(|redo| ActionMessage::AiSummary(Message::SetRedo(redo))),
        );

        let estimate: Element<'_, ActionMessage> = match self.estimate {
            EstimateState::None => column![
                button(text("Estimate cost").size(13))
                    .on_press(ActionMessage::AiSummary(Message::Estimate))
                    .padding([4, 12]),
                muted("The price is shown before anything is sent.".to_string()),
            ]
            .spacing(4)
            .into(),
            EstimateState::Running(_) => muted("Estimating…".to_string()).into(),
            EstimateState::Ready(estimate) => {
                let mut lines =
                    column![text(estimate_line(&estimate, settings.model)).size(13)].spacing(4);
                if let Some(skipped) = skipped_line(&estimate) {
                    lines = lines.push(muted(skipped));
                }
                if estimate.videos > 0 {
                    lines = lines.push(muted("Their subtitles are sent to Anthropic.".to_string()));
                }
                lines.into()
            }
        };
        options = options.push(estimate);

        super::panel(
            LABEL,
            "Sends each checked video's subtitles (.srt) to Claude and writes a short summary, \
             with time ranges, into its comment after your own text, which stays unchanged. \
             Running again replaces only the summary."
                .to_string(),
            options.into(),
        )
    }
}

/// `12 videos · about $0.35 with Claude Opus 5`.
fn estimate_line(estimate: &SummaryEstimate, model: AiModel) -> String {
    if estimate.videos == 0 {
        return "Nothing to send.".to_string();
    }
    let videos = if estimate.videos == 1 {
        "1 video".to_string()
    } else {
        format!("{} videos", estimate.videos)
    };
    format!(
        "{videos} · about {} with {}",
        dollars(estimate.usage.cost(model)),
        model.label()
    )
}

/// `$0.35`, or `under $0.01`.
fn dollars(amount: f64) -> String {
    if amount < 0.01 {
        "under $0.01".to_string()
    } else {
        format!("${amount:.2}")
    }
}

/// `3 without subtitles and 2 already summarized are skipped.`
fn skipped_line(estimate: &SummaryEstimate) -> Option<String> {
    let parts: Vec<String> = [
        (estimate.without_subtitles, "without subtitles"),
        (estimate.already_summarized, "already summarized"),
    ]
    .into_iter()
    .filter(|(n, _)| *n > 0)
    .map(|(n, what)| format!("{n} {what}"))
    .collect();
    (!parts.is_empty()).then(|| format!("{} are skipped.", parts.join(" and ")))
}

/// Work out the estimate for the files at `paths`. Blocking.
pub fn estimate(paths: &[PathBuf], job: Job) -> SummaryEstimate {
    ai::estimate_files(paths, job.language, job.redo)
}

/// Summarize the video at `path` and write the summary into its comment.
pub fn run(job: Job, path: &Path, cancel: &AtomicBool) -> ItemResult {
    let settings = ai::ai_settings();
    let Some(key) = settings.api_key() else {
        return ItemResult {
            stop_job: true,
            ..ItemResult::failed("No Anthropic API key")
        };
    };
    let provider = AnthropicProvider::new(key);
    match ai::summarize_file(&provider, path, job.model, job.language, job.redo, cancel) {
        Ok(SummarizeOutcome::Written {
            path: new_path,
            snapshot,
            usage,
        }) => {
            log::info!(
                "ai summary: {} with {}: {} input + {} output tokens, ${:.4}",
                path.display(),
                job.model.label(),
                usage.input_tokens,
                usage.output_tokens,
                usage.cost(job.model)
            );
            ItemResult::new(ItemStatus::Done, Some((new_path, *snapshot)))
        }
        Ok(SummarizeOutcome::NoSubtitles | SummarizeOutcome::AlreadySummarized)
        | Err(SummarizeError::Ai(AiError::Cancelled)) => ItemResult::new(ItemStatus::Skipped, None),
        Err(error) => {
            log::warn!("ai summary: {}: {error}", path.display());
            let stop_job = matches!(&error, SummarizeError::Ai(e) if e.stops_job());
            let update = match &error {
                SummarizeError::NotSaved { path, snapshot } => {
                    Some((path.clone(), (**snapshot).clone()))
                }
                _ => None,
            };
            ItemResult {
                stop_job,
                update,
                ..ItemResult::failed(error.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frename_core::ai::AiUsage;

    #[test]
    fn a_new_estimate_replaces_the_old_and_a_stale_answer_is_dropped() {
        let mut options = Options::default();
        options.update(Message::Estimate);
        let (first, _) = options.pending_estimate().expect("running");
        options.checks_changed();
        options.update(Message::Estimate);
        let (second, _) = options.pending_estimate().expect("running");
        let ready = SummaryEstimate {
            videos: 2,
            ..SummaryEstimate::default()
        };
        options.update(Message::Estimated(first, ready));
        assert!(options.pending_estimate().is_some(), "stale answer dropped");
        options.update(Message::Estimated(second, ready));
        assert_eq!(options.estimate, EstimateState::Ready(ready));

        options.update(Message::SetRedo(true));
        assert_eq!(
            options.estimate,
            EstimateState::None,
            "redo counts other videos"
        );
    }

    #[test]
    fn it_does_not_run_without_an_estimate() {
        let options = Options::default();
        assert_eq!(options.operation(), None);
    }

    #[test]
    fn the_estimate_reads_like_a_price() {
        let estimate = SummaryEstimate {
            videos: 12,
            without_subtitles: 3,
            already_summarized: 2,
            usage: AiUsage {
                input_tokens: 40_000,
                output_tokens: 8_400,
            },
        };
        assert_eq!(
            estimate_line(&estimate, AiModel::ClaudeOpus5),
            "12 videos · about $0.41 with Claude Opus 5"
        );
        assert_eq!(
            skipped_line(&estimate).as_deref(),
            Some("3 without subtitles and 2 already summarized are skipped.")
        );
        let none = SummaryEstimate::default();
        assert_eq!(
            estimate_line(&none, AiModel::ClaudeHaiku45),
            "Nothing to send."
        );
        assert_eq!(skipped_line(&none), None);
        assert_eq!(dollars(0.004), "under $0.01");
    }
}
