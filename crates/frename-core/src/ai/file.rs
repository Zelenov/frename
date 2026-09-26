//! Summaries of files on disk: read a video's subtitles and comment, ask the provider, and write
//! the AI block into the comment through the normal save path.

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use super::block::{ai_block, format_ai_block, has_ai_block, replace_ai_block, today};
use super::provider::{AiError, AiProvider, AiUsage};
use super::settings::{AiModel, SummaryLanguage};
use super::summary::{summarize, summary_prompt, SummaryEstimate};
use crate::{load_subtitles, FileSnapshot, FileTagger, FolderInfo, SaveAndReparse};

/// What summarizing one file did.
#[derive(Debug, Clone)]
pub enum SummarizeOutcome {
    /// The AI block was written; the file's path and snapshot afterwards, and the tokens used.
    Written {
        path: PathBuf,
        snapshot: Box<FileSnapshot>,
        usage: AiUsage,
    },
    /// The video has no subtitles.
    NoSubtitles,
    /// The video already has an AI block and `redo` was off.
    AlreadySummarized,
}

/// Why summarizing one file failed.
#[derive(Debug, Clone)]
pub enum SummarizeError {
    Ai(AiError),
    /// The comment could not be read; writing would lose it.
    CommentUnreadable,
    /// The comment did not come back with the new block after the save.
    NotSaved {
        path: PathBuf,
        snapshot: Box<FileSnapshot>,
    },
}

impl std::fmt::Display for SummarizeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ai(error) => error.fmt(f),
            Self::CommentUnreadable => write!(f, "The comment could not be read"),
            Self::NotSaved { .. } => write!(f, "The comment could not be saved"),
        }
    }
}

/// The snapshot of the file at `path` with its comment read, even when the folder scan deferred
/// it. `None` when the comment still could not be read.
fn read_with_comment(path: &Path) -> Option<FileSnapshot> {
    let snapshot = FileTagger::parse(path, &FolderInfo::default());
    let snapshot = if snapshot.comment_loading() {
        FileTagger::load_comment(path, &snapshot)
    } else {
        snapshot
    };
    (!snapshot.comment_loading()).then_some(snapshot)
}

/// Summarize the video at `path` from its subtitles and write the summary into its comment as
/// the AI block, keeping the editor's text. Blocking.
pub fn summarize_file(
    provider: &dyn AiProvider,
    path: &Path,
    model: AiModel,
    language: SummaryLanguage,
    redo: bool,
    cancel: &AtomicBool,
) -> Result<SummarizeOutcome, SummarizeError> {
    let Some(subtitles) = load_subtitles(path) else {
        return Ok(SummarizeOutcome::NoSubtitles);
    };
    let mut snapshot = read_with_comment(path).ok_or(SummarizeError::CommentUnreadable)?;
    if !redo && has_ai_block(snapshot.comment()) {
        return Ok(SummarizeOutcome::AlreadySummarized);
    }
    let (summary, usage) =
        summarize(provider, model, language, &subtitles, cancel).map_err(SummarizeError::Ai)?;
    let block = format_ai_block(&summary, model.label(), &today());
    snapshot.set_comment(replace_ai_block(snapshot.comment(), &block));
    let (path, snapshot) = snapshot.save_and_reparse(path);
    if ai_block(snapshot.comment()) != Some(block.as_str()) {
        return Err(SummarizeError::NotSaved {
            path,
            snapshot: Box::new(snapshot),
        });
    }
    Ok(SummarizeOutcome::Written {
        path,
        snapshot: Box::new(snapshot),
        usage,
    })
}

/// The cost estimate for the files at `paths`, counted as [`summarize_file`] would treat them.
/// Blocking: reads their subtitles and, when summarized videos are not redone, their comments.
pub fn estimate_files(paths: &[PathBuf], language: SummaryLanguage, redo: bool) -> SummaryEstimate {
    let mut estimate = SummaryEstimate::default();
    for path in paths {
        let Some(subtitles) = load_subtitles(path) else {
            estimate.without_subtitles += 1;
            continue;
        };
        let summarized = || read_with_comment(path).is_some_and(|s| has_ai_block(s.comment()));
        if !redo && summarized() {
            estimate.already_summarized += 1;
            continue;
        }
        estimate.add_video(summary_prompt(&subtitles, language).chars().count());
    }
    estimate
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::provider::{AiRequest, AiResponse};
    use crate::subtitle_path;
    use serde_json::json;
    use std::sync::Mutex;

    /// Answers every request with the same summary, or fails; counts the requests.
    struct MockProvider {
        answer: Result<&'static str, AiError>,
        requests: Mutex<usize>,
    }

    impl MockProvider {
        fn new(answer: Result<&'static str, AiError>) -> Self {
            Self {
                answer,
                requests: Mutex::new(0),
            }
        }

        fn requests(&self) -> usize {
            self.requests.lock().map_or(0, |r| *r)
        }
    }

    impl AiProvider for MockProvider {
        fn complete(
            &self,
            _request: &AiRequest,
            _cancel: &AtomicBool,
        ) -> Result<AiResponse, AiError> {
            if let Ok(mut requests) = self.requests.lock() {
                *requests += 1;
            }
            let summary = self.answer.clone()?;
            Ok(AiResponse {
                json: json!({"summary": summary, "segments": []}),
                usage: AiUsage {
                    input_tokens: 100,
                    output_tokens: 10,
                },
            })
        }
    }

    /// A video with subtitles and `comment`, saved through the (in-memory) tagger.
    fn video(name: &str, comment: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("frename-ai-file-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let path = dir.join(name);
        std::fs::write(
            subtitle_path(&path),
            "1\n00:00:00,000 --> 00:00:04,000\nHello there\n",
        )
        .expect("srt");
        let mut snapshot = FileSnapshot::parse(name);
        snapshot.set_comment(comment.to_string());
        FileTagger::save(&snapshot, &path)
    }

    fn run(
        provider: &MockProvider,
        path: &Path,
        redo: bool,
    ) -> Result<SummarizeOutcome, SummarizeError> {
        summarize_file(
            provider,
            path,
            AiModel::ClaudeHaiku45,
            SummaryLanguage::English,
            redo,
            &AtomicBool::new(false),
        )
    }

    fn comment(path: &Path) -> String {
        FileTagger::parse(path, &FolderInfo::default())
            .comment()
            .to_string()
    }

    #[test]
    fn a_summary_is_added_after_the_editors_text() {
        let path = video("add.mp4", "Mine — Café 🎥");
        let outcome = run(&MockProvider::new(Ok("First.")), &path, false).expect("written");
        assert!(matches!(outcome, SummarizeOutcome::Written { .. }));
        let comment = comment(&path);
        assert!(comment.starts_with("Mine — Café 🎥\n\nAI: First.\n— Claude Haiku 4.5, "));
    }

    #[test]
    fn a_redo_replaces_only_the_block_and_without_redo_nothing_is_sent() {
        let path = video("redo.mp4", "Mine");
        run(&MockProvider::new(Ok("First.")), &path, false).expect("first");

        let provider = MockProvider::new(Ok("Second."));
        assert!(matches!(
            run(&provider, &path, false),
            Ok(SummarizeOutcome::AlreadySummarized)
        ));
        assert_eq!(provider.requests(), 0);

        run(&provider, &path, true).expect("redo");
        let comment = comment(&path);
        assert!(comment.starts_with("Mine\n\nAI: Second.\n"), "{comment}");
        assert!(!comment.contains("First."));
    }

    #[test]
    fn a_failed_request_leaves_the_comment_as_it_was() {
        let path = video("fail.mp4", "Mine");
        let result = run(&MockProvider::new(Err(AiError::KeyRejected)), &path, false);
        assert!(matches!(
            result,
            Err(SummarizeError::Ai(AiError::KeyRejected))
        ));
        assert_eq!(comment(&path), "Mine");
    }

    #[test]
    fn a_video_without_subtitles_is_not_sent() {
        let dir = std::env::temp_dir().join(format!("frename-ai-file-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let provider = MockProvider::new(Ok("Never."));
        let path = dir.join("silent.mp4");
        assert!(matches!(
            run(&provider, &path, true),
            Ok(SummarizeOutcome::NoSubtitles)
        ));
        assert_eq!(provider.requests(), 0);
    }

    #[test]
    fn the_estimate_counts_what_a_run_would_send() {
        let summarized = video("est-done.mp4", "Mine");
        run(&MockProvider::new(Ok("First.")), &summarized, false).expect("first");
        let fresh = video("est-fresh.mp4", "");
        let silent = fresh.with_file_name("est-silent.mp4");
        let paths = [summarized, fresh, silent];

        let estimate = estimate_files(&paths, SummaryLanguage::English, false);
        assert_eq!(
            (
                estimate.videos,
                estimate.without_subtitles,
                estimate.already_summarized
            ),
            (1, 1, 1)
        );
        assert!(estimate.usage.input_tokens > 0);
        assert_eq!(
            estimate_files(&paths, SummaryLanguage::English, true).videos,
            2
        );
    }
}
