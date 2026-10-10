//! AI descriptions of clips (issue #17, stage 1), frename's side: the AI block of a comment and
//! the API key. Describing a clip itself (frames, request, models, cost) is the
//! `clipscribe` crate.

pub mod block;
pub mod key;

pub use block::has_editor_comment;
use clipscribe::{Model, Provider, MODELS};
pub use clipscribe::{MomentsMode, SummaryLanguage};

/// The models frename offers: Anthropic's. The only key it asks for and keeps is Anthropic's, so
/// a model of another provider (clipscribe also knows OpenAI's) would be sent that key.
pub fn models() -> Vec<Model> {
    MODELS
        .into_iter()
        .filter(|m| m.provider == Provider::Anthropic)
        .collect()
}

/// The offered model with `id`; an unknown id, or one of another provider (saved by an older
/// version), is the default (the cheapest).
pub fn model_from_id(id: &str) -> Model {
    let models = models();
    models
        .iter()
        .find(|m| m.id == id)
        .copied()
        .unwrap_or(models[0])
}

/// `moments`' name as the settings store it: `important` or `full`.
pub fn moments_as_str(moments: MomentsMode) -> &'static str {
    match moments {
        MomentsMode::Important => "important",
        MomentsMode::Full => "full",
    }
}

/// The moments mode stored as `name`; an unknown name is the default (only what stands out).
pub fn moments_from_name(name: &str) -> MomentsMode {
    match name {
        "full" => MomentsMode::Full,
        _ => MomentsMode::Important,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_anthropic_models_are_offered() {
        assert!(!models().is_empty());
        assert!(models().iter().all(|m| m.provider == Provider::Anthropic));
        assert_eq!(model_from_id("claude-haiku-4-5").id, "claude-haiku-4-5");
        assert_eq!(model_from_id("gpt-4.1").id, models()[0].id);
        assert_eq!(model_from_id("nonsense").id, models()[0].id);
    }

    #[test]
    fn a_moments_mode_is_read_back_by_its_name() {
        for moments in [MomentsMode::Important, MomentsMode::Full] {
            assert_eq!(moments_from_name(moments_as_str(moments)), moments);
        }
        assert_eq!(moments_from_name("something else"), MomentsMode::Important);
    }
}
