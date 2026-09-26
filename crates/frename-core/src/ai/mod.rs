//! AI descriptions of clips (issue #17, stage 1), frename's side: the AI block of a comment and
//! the API key. Describing a clip itself (frames, request, models, cost) is the
//! `video-describe` crate.

pub mod block;
pub mod key;

pub use block::has_editor_comment;
pub use video_describe::SummaryLanguage;
