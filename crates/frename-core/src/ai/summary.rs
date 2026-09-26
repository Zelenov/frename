//! Summaries from subtitles: the prompt, the answer's schema and its validation, and the cost
//! estimate shown before a run.

use std::sync::atomic::AtomicBool;
use std::time::Duration;

use serde_json::{json, Value};

use super::block::{format_time, AiSegment, AiSummary};
use super::provider::{AiError, AiProvider, AiRequest, AiUsage};
use super::settings::{AiModel, SummaryLanguage};
use crate::Subtitles;

/// Most segments asked for, so the comment stays short.
const MAX_SEGMENTS: u64 = 8;

/// The prompt for a clip with `subtitles`.
pub fn summary_prompt(subtitles: &Subtitles, language: SummaryLanguage) -> String {
    let language = match language {
        SummaryLanguage::SameAsSubtitles => {
            "the language the subtitles are in (if they mix languages, the main one)".to_string()
        }
        other => other.label().to_string(),
    };
    let max_segments = max_segments(subtitles);
    let mut transcript = String::new();
    for cue in subtitles.cues() {
        let text = cue.text.split_whitespace().collect::<Vec<_>>().join(" ");
        transcript.push_str(&format!(
            "[{}–{}] {text}\n",
            format_time(cue.start),
            format_time(cue.end)
        ));
    }
    format!(
        "You help a video editor who has not watched a clip decide what it contains. Below are the \
         clip's subtitles: a speech transcript with timestamps, often made by automatic \
         transcription, so it can contain errors.\n\
         \n\
         Write in {language}.\n\
         - summary: one or two short sentences on what happens or what is talked about in the \
         clip. Be factual and specific; do not guess who the speakers are unless the transcript \
         says it.\n\
         - segments: when the clip has clearly different parts, up to {max_segments} time ranges \
         in seconds from the start of the clip (start_s, end_s) with a few words each on what \
         happens there, in order and not overlapping. For a clip about a single thing, an empty \
         list.\n\
         \n\
         <subtitles>\n{transcript}</subtitles>"
    )
}

/// Segments asked for: one per half minute of speech, between 2 and [`MAX_SEGMENTS`].
fn max_segments(subtitles: &Subtitles) -> u64 {
    (spoken_until(subtitles).as_secs() / 30).clamp(2, MAX_SEGMENTS)
}

/// End of the last cue.
fn spoken_until(subtitles: &Subtitles) -> Duration {
    subtitles
        .cues()
        .iter()
        .map(|c| c.end)
        .max()
        .unwrap_or_default()
}

/// The JSON schema of the answer.
pub fn summary_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "summary": {"type": "string"},
            "segments": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "start_s": {"type": "number"},
                        "end_s": {"type": "number"},
                        "description": {"type": "string"}
                    },
                    "required": ["start_s", "end_s", "description"],
                    "additionalProperties": false
                }
            }
        },
        "required": ["summary", "segments"],
        "additionalProperties": false
    })
}

/// Read an answer. Segments that are empty, backwards, overlap an earlier one, or start after
/// the last subtitle are dropped; an empty summary fails.
pub fn parse_summary(answer: &Value, subtitles: &Subtitles) -> Result<AiSummary, AiError> {
    let summary = answer["summary"].as_str().unwrap_or_default().trim();
    if summary.is_empty() {
        return Err(AiError::InvalidAnswer("empty summary".to_string()));
    }
    // A segment may run a little past the last cue (the scene goes on), not start after it.
    let last = spoken_until(subtitles).as_secs_f64();
    let mut segments: Vec<AiSegment> = Vec::new();
    for item in answer["segments"].as_array().into_iter().flatten() {
        let (Some(start), Some(end)) = (item["start_s"].as_f64(), item["end_s"].as_f64()) else {
            continue;
        };
        let description = item["description"].as_str().unwrap_or_default().trim();
        let after_previous = segments
            .last()
            .is_none_or(|p| start >= p.end.as_secs_f64() - 0.5);
        let valid = start >= 0.0 && end > start && start < last && after_previous;
        if !valid || description.is_empty() || segments.len() as u64 >= MAX_SEGMENTS {
            continue;
        }
        // A minute past the last cue at most: also keeps a huge number from overflowing.
        let end = end.min(last + 60.0);
        segments.push(AiSegment {
            start: Duration::from_secs_f64(start),
            end: Duration::from_secs_f64(end),
            description: description.to_string(),
        });
    }
    Ok(AiSummary {
        summary: summary.to_string(),
        segments,
    })
}

/// Summarize a clip from its subtitles.
pub fn summarize(
    provider: &dyn AiProvider,
    model: AiModel,
    language: SummaryLanguage,
    subtitles: &Subtitles,
    cancel: &AtomicBool,
) -> Result<(AiSummary, AiUsage), AiError> {
    let request = AiRequest {
        model,
        prompt: summary_prompt(subtitles, language),
        schema: summary_schema(),
    };
    let response = provider.complete(&request, cancel)?;
    Ok((parse_summary(&response.json, subtitles)?, response.usage))
}

/// Overhead per request beyond the prompt's text (the schema), in input tokens.
const PROMPT_TOKENS: u64 = 300;
/// Answer per clip in output tokens, thinking included.
const ANSWER_TOKENS: u64 = 700;

/// Input tokens a clip's request is expected to use: about 3 characters per token (subtitles
/// are often not English, which takes more tokens per character).
pub fn estimated_input_tokens(prompt_chars: usize) -> u64 {
    prompt_chars as u64 / 3 + PROMPT_TOKENS
}

/// The cost estimate shown before a run.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SummaryEstimate {
    /// Videos that will be sent.
    pub videos: usize,
    /// Checked files without subtitles, skipped.
    pub without_subtitles: usize,
    /// Videos that already have an AI block and are not redone, skipped.
    pub already_summarized: usize,
    /// Expected tokens over all the videos sent.
    pub usage: AiUsage,
}

impl SummaryEstimate {
    /// Count one video that will be sent, with the character length of its prompt.
    pub fn add_video(&mut self, prompt_chars: usize) {
        self.videos += 1;
        self.usage.input_tokens += estimated_input_tokens(prompt_chars);
        self.usage.output_tokens += ANSWER_TOKENS;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRT: &str = "1\n00:00:00,000 --> 00:00:05,000\nПривет всем,\nсегодня борщ.\n\n\
                       2\n00:01:00,000 --> 00:01:30,000\nРежем свёклу.\n";

    fn subtitles() -> Subtitles {
        Subtitles::parse(SRT)
    }

    #[test]
    fn the_prompt_holds_the_transcript_and_the_language() {
        let prompt = summary_prompt(&subtitles(), SummaryLanguage::SameAsSubtitles);
        assert!(prompt.contains("[0:00–0:05] Привет всем, сегодня борщ.\n"));
        assert!(prompt.contains("[1:00–1:30] Режем свёклу.\n"));
        assert!(prompt.contains("the language the subtitles are in"));
        assert!(prompt.contains("up to 3 time ranges"));
        let english = summary_prompt(&subtitles(), SummaryLanguage::English);
        assert!(english.contains("Write in English."));
    }

    #[test]
    fn the_schema_is_strict() {
        let schema = summary_schema();
        assert_eq!(schema["additionalProperties"], false);
        assert_eq!(schema["required"], json!(["summary", "segments"]));
        let item = &schema["properties"]["segments"]["items"];
        assert_eq!(item["additionalProperties"], false);
        assert_eq!(item["required"], json!(["start_s", "end_s", "description"]));
    }

    #[test]
    fn invalid_segments_are_dropped() {
        let answer = json!({
            "summary": " A cooking lesson. ",
            "segments": [
                {"start_s": 0, "end_s": 30, "description": "Greeting"},
                {"start_s": 20, "end_s": 40, "description": "Overlaps"},
                {"start_s": 60, "end_s": 50, "description": "Backwards"},
                {"start_s": 60, "end_s": 1e300, "description": "Beets"},
                {"start_s": 95, "end_s": 99, "description": " "},
                {"start_s": 200, "end_s": 300, "description": "After the end"}
            ]
        });
        let summary = parse_summary(&answer, &subtitles()).expect("summary");
        assert_eq!(summary.summary, "A cooking lesson.");
        let kept: Vec<&str> = summary
            .segments
            .iter()
            .map(|s| s.description.as_str())
            .collect();
        assert_eq!(kept, ["Greeting", "Beets"]);
        assert_eq!(summary.segments[1].end, Duration::from_secs(150), "clamped");
    }

    #[test]
    fn an_empty_summary_fails() {
        let answer = json!({"summary": "  ", "segments": []});
        assert!(matches!(
            parse_summary(&answer, &subtitles()),
            Err(AiError::InvalidAnswer(_))
        ));
    }

    #[test]
    fn the_estimate_adds_up_per_video() {
        let mut estimate = SummaryEstimate::default();
        estimate.add_video(3_000);
        estimate.add_video(0);
        assert_eq!(estimate.videos, 2);
        assert_eq!(estimate.usage.input_tokens, 1_000 + 2 * PROMPT_TOKENS);
        assert_eq!(estimate.usage.output_tokens, 2 * ANSWER_TOKENS);
    }

    /// Calls the real API, only when `FRENAME_LIVE_ANTHROPIC_KEY` holds a key (a CI secret or a
    /// developer's key); without it the test passes without touching the network.
    #[test]
    fn live_summary_from_subtitles() {
        let Some(key) = std::env::var("FRENAME_LIVE_ANTHROPIC_KEY")
            .ok()
            .filter(|k| !k.trim().is_empty())
        else {
            eprintln!("skipped: FRENAME_LIVE_ANTHROPIC_KEY is not set");
            return;
        };
        let provider = crate::ai::AnthropicProvider::new(&key);
        let (summary, usage) = summarize(
            &provider,
            AiModel::ClaudeHaiku45,
            SummaryLanguage::English,
            &subtitles(),
            &AtomicBool::new(false),
        )
        .expect("a summary");
        assert!(!summary.summary.is_empty());
        for segment in &summary.segments {
            assert!(segment.start < segment.end);
        }
        eprintln!("live: {summary:?}, {usage:?}");
    }
}
