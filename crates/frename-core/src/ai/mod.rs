//! AI descriptions of clips (issue #17, stage 1): the AI block of a comment, the provider
//! layer with its Anthropic implementation, the clip request and its cost, and the API key.
//! No iced, no GStreamer: frames are sampled by the app and handed in.

pub mod anthropic;
pub mod block;
pub mod describe;
pub mod key;
pub mod provider;

pub use block::{editor_comment, has_editor_comment};
pub use describe::SummaryLanguage;
