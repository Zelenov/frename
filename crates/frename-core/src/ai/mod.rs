//! AI descriptions of clips (issue #17, stage 1): the AI block of a comment, the provider
//! layer with its Anthropic implementation, the clip request and its cost, and the API key.
//! No iced, no GStreamer: frames are sampled by the app and handed in.

pub mod anthropic;
pub mod block;
pub mod describe;
pub mod key;
pub mod provider;

use std::sync::RwLock;

pub use block::{editor_comment, has_editor_comment};
pub use describe::SummaryLanguage;

static SUMMARY_LANGUAGE: RwLock<SummaryLanguage> = RwLock::new(SummaryLanguage::SameAsSubtitles);

/// Set the language descriptions are written in (from the settings).
pub fn set_summary_language(language: SummaryLanguage) {
    if let Ok(mut current) = SUMMARY_LANGUAGE.write() {
        *current = language;
    }
}

/// The language descriptions are written in.
pub fn summary_language() -> SummaryLanguage {
    SUMMARY_LANGUAGE.read().map(|l| *l).unwrap_or_default()
}
