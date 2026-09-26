//! The AI settings: which model writes the summaries, in which language, and the API key.
//! Edited in the settings window and set here for the batch action (see [`set_ai_settings`]).

use std::sync::RwLock;

/// A Claude model the summaries can be written with.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AiModel {
    #[default]
    ClaudeOpus5,
    ClaudeSonnet5,
    ClaudeHaiku45,
}

/// Price of a model in US dollars per million tokens.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Price {
    pub input: f64,
    pub output: f64,
}

impl AiModel {
    pub const ALL: [AiModel; 3] = [Self::ClaudeOpus5, Self::ClaudeSonnet5, Self::ClaudeHaiku45];

    /// The model's API id.
    pub fn id(self) -> &'static str {
        match self {
            Self::ClaudeOpus5 => "claude-opus-5",
            Self::ClaudeSonnet5 => "claude-sonnet-5",
            Self::ClaudeHaiku45 => "claude-haiku-4-5",
        }
    }

    /// The name shown to the user and written in the AI block's end line.
    pub fn label(self) -> &'static str {
        match self {
            Self::ClaudeOpus5 => "Claude Opus 5",
            Self::ClaudeSonnet5 => "Claude Sonnet 5",
            Self::ClaudeHaiku45 => "Claude Haiku 4.5",
        }
    }

    /// Anthropic's list prices, checked 2026-09-26.
    pub fn price(self) -> Price {
        match self {
            Self::ClaudeOpus5 => Price {
                input: 5.0,
                output: 25.0,
            },
            Self::ClaudeSonnet5 => Price {
                input: 2.0,
                output: 10.0,
            },
            Self::ClaudeHaiku45 => Price {
                input: 1.0,
                output: 5.0,
            },
        }
    }

    /// Name stored in the app database.
    pub fn as_str(self) -> &'static str {
        self.id()
    }

    /// Parse a stored name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|m| m.id() == name)
            .unwrap_or_default()
    }
}

impl std::fmt::Display for AiModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// The language summaries are written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SummaryLanguage {
    /// The subtitles' own language: a Russian interview gets a Russian summary.
    #[default]
    SameAsSubtitles,
    English,
    Russian,
    Ukrainian,
    German,
    Spanish,
    French,
}

impl SummaryLanguage {
    pub const ALL: [SummaryLanguage; 7] = [
        Self::SameAsSubtitles,
        Self::English,
        Self::Russian,
        Self::Ukrainian,
        Self::German,
        Self::Spanish,
        Self::French,
    ];

    /// The name shown in the settings.
    pub fn label(self) -> &'static str {
        match self {
            Self::SameAsSubtitles => "Same as the subtitles",
            Self::English => "English",
            Self::Russian => "Russian",
            Self::Ukrainian => "Ukrainian",
            Self::German => "German",
            Self::Spanish => "Spanish",
            Self::French => "French",
        }
    }

    /// Name stored in the app database.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SameAsSubtitles => "subtitles",
            Self::English => "en",
            Self::Russian => "ru",
            Self::Ukrainian => "uk",
            Self::German => "de",
            Self::Spanish => "es",
            Self::French => "fr",
        }
    }

    /// Parse a stored name; unknown names fall back to the default.
    pub fn from_name(name: &str) -> Self {
        Self::ALL
            .into_iter()
            .find(|l| l.as_str() == name)
            .unwrap_or_default()
    }
}

impl std::fmt::Display for SummaryLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// The AI settings as the batch action uses them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AiSettings {
    pub model: AiModel,
    pub language: SummaryLanguage,
    /// The Anthropic API key; empty when not set.
    pub anthropic_api_key: String,
}

impl AiSettings {
    /// The key with surrounding whitespace (from pasting) removed; `None` when not set.
    pub fn api_key(&self) -> Option<&str> {
        Some(self.anthropic_api_key.trim()).filter(|k| !k.is_empty())
    }
}

static AI_SETTINGS: RwLock<Option<AiSettings>> = RwLock::new(None);

/// Set the AI settings the batch action runs with. Called at start-up and whenever they change
/// in the settings window.
pub fn set_ai_settings(settings: AiSettings) {
    if let Ok(mut current) = AI_SETTINGS.write() {
        *current = Some(settings);
    }
}

/// The AI settings set last (defaults, without a key, before any).
pub fn ai_settings() -> AiSettings {
    AI_SETTINGS
        .read()
        .ok()
        .and_then(|s| s.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_names_round_trip_and_unknown_ones_fall_back() {
        for model in AiModel::ALL {
            assert_eq!(AiModel::from_name(model.as_str()), model);
        }
        for language in SummaryLanguage::ALL {
            assert_eq!(SummaryLanguage::from_name(language.as_str()), language);
        }
        assert_eq!(AiModel::from_name("gpt"), AiModel::ClaudeOpus5);
        assert_eq!(
            SummaryLanguage::from_name(""),
            SummaryLanguage::SameAsSubtitles
        );
    }

    #[test]
    fn a_blank_key_is_no_key() {
        let mut settings = AiSettings::default();
        assert_eq!(settings.api_key(), None);
        settings.anthropic_api_key = "  sk-ant-x \n".to_string();
        assert_eq!(settings.api_key(), Some("sk-ant-x"));
    }
}
