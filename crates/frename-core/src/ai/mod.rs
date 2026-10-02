//! AI descriptions of clips (issue #17, stage 1), frename's side: the AI block of a comment and
//! the API key. Describing a clip itself (frames, request, models, cost) is the
//! `clipscribe` crate.

pub mod block;
pub mod key;

pub use block::has_editor_comment;
pub use clipscribe::{MomentsMode, SummaryLanguage};

/// `moments`' name as the settings store it: `important` or `full`.
pub fn moments_name(moments: MomentsMode) -> &'static str {
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
    fn a_moments_mode_is_read_back_by_its_name() {
        for moments in [MomentsMode::Important, MomentsMode::Full] {
            assert_eq!(moments_from_name(moments_name(moments)), moments);
        }
        assert_eq!(moments_from_name("something else"), MomentsMode::Important);
    }
}
