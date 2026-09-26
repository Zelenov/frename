//! AI features: summaries of a video written into its comment (the AI block), and the provider
//! layer that talks to the language model. No UI here; the batch action in the app drives it.

mod block;
mod provider;
mod settings;
mod summary;

pub use block::{
    ai_block, editor_comment, format_ai_block, has_ai_block, has_editor_comment, replace_ai_block,
    today, AiSegment, AiSummary,
};
pub use provider::{
    AiError, AiProvider, AiRequest, AiResponse, AiUsage, AnthropicProvider, HttpResponse,
    HttpTransport, RetryPolicy, TransportError, UreqTransport, REQUEST_TIMEOUT,
};
pub use settings::{ai_settings, set_ai_settings, AiModel, AiSettings, Price, SummaryLanguage};
pub use summary::{
    estimated_input_tokens, parse_summary, summarize, summary_prompt, summary_schema,
    SummaryEstimate,
};
