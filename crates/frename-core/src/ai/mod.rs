//! AI descriptions of clips (issue #17, stage 1), frename's side: the AI block of a comment and
//! the API key. Describing a clip itself (frames, request, models, cost) is the
//! `clipscribe` crate.

pub mod block;
pub mod key;

pub use block::has_editor_comment;
use clipscribe::{Model, MODELS};
pub use clipscribe::{MomentsMode, SummaryLanguage};

/// The models frename offers: all of clipscribe's, Anthropic's first, then OpenAI's. A model is
/// paid with the key of its own provider ([`key::ApiKey::of_provider`]).
pub fn models() -> Vec<Model> {
    MODELS.into_iter().collect()
}

/// The offered model with `id`; an unknown id is the default (the cheapest).
pub fn model_from_id(id: &str) -> Model {
    let models = models();
    models
        .iter()
        .find(|m| m.id == id)
        .copied()
        .unwrap_or(models[0])
}

/// The key `model` is paid with.
pub fn key_of(model: Model) -> key::ApiKey {
    key::ApiKey::of_provider(model.provider)
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
    use clipscribe::Provider;

    #[test]
    fn both_providers_models_are_offered_each_with_its_own_key() {
        let models = models();
        assert_eq!(
            models[0].provider,
            Provider::Anthropic,
            "the default is Claude's"
        );
        assert!(models.iter().any(|m| m.provider == Provider::OpenAi));
        assert_eq!(model_from_id("claude-haiku-4-5").id, "claude-haiku-4-5");
        assert_eq!(model_from_id("gpt-4.1").id, "gpt-4.1");
        assert_eq!(model_from_id("nonsense").id, models[0].id);
        assert_eq!(key_of(model_from_id("gpt-4.1")), key::ApiKey::OpenAi);
        assert_eq!(
            key_of(model_from_id("claude-haiku-4-5")),
            key::ApiKey::Anthropic
        );
    }

    #[test]
    fn a_moments_mode_is_read_back_by_its_name() {
        for moments in [MomentsMode::Important, MomentsMode::Full] {
            assert_eq!(moments_from_name(moments_as_str(moments)), moments);
        }
        assert_eq!(moments_from_name("something else"), MomentsMode::Important);
    }
}
