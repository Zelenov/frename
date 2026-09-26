//! The provider layer: one request to a language model and its answer, with timeouts, retries
//! and cancel. Shaped around the request (text in, JSON out), not around clips, so other
//! features and other providers fit it.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::{json, Value};

use super::settings::AiModel;

/// A request for a JSON answer that follows `schema`.
#[derive(Debug, Clone, PartialEq)]
pub struct AiRequest {
    pub model: AiModel,
    pub prompt: String,
    /// JSON schema of the answer; every object has `required` fields and no additional ones.
    pub schema: Value,
}

/// Tokens a request used, as the provider counted them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AiUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl AiUsage {
    /// What the tokens cost with `model`, in US dollars.
    pub fn cost(self, model: AiModel) -> f64 {
        let price = model.price();
        (self.input_tokens as f64 * price.input + self.output_tokens as f64 * price.output)
            / 1_000_000.0
    }
}

/// The answer to a request.
#[derive(Debug, Clone, PartialEq)]
pub struct AiResponse {
    pub json: Value,
    pub usage: AiUsage,
}

/// Why a request failed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiError {
    /// The key was rejected (401/403). Every later request would fail the same way.
    KeyRejected,
    /// The account has no credit left. Every later request would fail the same way.
    NoCredit,
    /// The request was refused as invalid, with the provider's own message.
    BadRequest(String),
    /// No answer within the timeout. Not retried: the provider may have billed it already.
    Timeout,
    /// The provider could not be reached, or kept failing, after the retries.
    Unavailable(String),
    /// The model declined to answer.
    Refused,
    /// The answer was cut off at the output limit.
    Truncated,
    /// The answer could not be read.
    InvalidAnswer(String),
    /// Cancelled while waiting to retry.
    Cancelled,
}

impl AiError {
    /// Whether every later request would fail the same way, so a batch job should stop.
    pub fn stops_job(&self) -> bool {
        matches!(self, Self::KeyRejected | Self::NoCredit)
    }
}

impl std::fmt::Display for AiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyRejected => write!(f, "The API key was rejected"),
            Self::NoCredit => write!(f, "The API account has no credit left"),
            Self::BadRequest(message) => write!(f, "The request was refused: {message}"),
            Self::Timeout => write!(f, "No answer within {} s", REQUEST_TIMEOUT.as_secs()),
            Self::Unavailable(why) => write!(f, "The AI service is not reachable: {why}"),
            Self::Refused => write!(f, "The model declined to summarize this video"),
            Self::Truncated => write!(f, "The answer was cut off"),
            Self::InvalidAnswer(why) => write!(f, "Unreadable answer: {why}"),
            Self::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::error::Error for AiError {}

/// A language model provider.
pub trait AiProvider: Send + Sync {
    /// Send `request` and return the answer. Blocking. `cancel` is checked while waiting to
    /// retry; a request already sent runs to its answer or timeout.
    fn complete(&self, request: &AiRequest, cancel: &AtomicBool) -> Result<AiResponse, AiError>;
}

/// An HTTP response, as much of it as the providers read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpResponse {
    pub status: u16,
    /// The `retry-after` header in seconds, if any.
    pub retry_after: Option<Duration>,
    pub body: String,
}

/// Why an HTTP request got no response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportError {
    Timeout,
    Network(String),
}

/// Sends HTTP requests; replaced by a fake in tests, so no test needs the network.
pub trait HttpTransport: Send + Sync {
    fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &str,
        timeout: Duration,
    ) -> Result<HttpResponse, TransportError>;
}

/// The real transport: rustls with the operating system's certificate store (corporate proxies
/// that inspect TLS install their certificate there), and the system's proxy settings.
pub struct UreqTransport {
    agent: ureq::Agent,
}

impl Default for UreqTransport {
    fn default() -> Self {
        let tls = ureq::tls::TlsConfig::builder()
            .root_certs(ureq::tls::RootCerts::PlatformVerifier)
            .build();
        let agent = ureq::Agent::config_builder()
            .tls_config(tls)
            .http_status_as_error(false)
            .build()
            .new_agent();
        Self { agent }
    }
}

impl HttpTransport for UreqTransport {
    fn post_json(
        &self,
        url: &str,
        headers: &[(&str, &str)],
        body: &str,
        timeout: Duration,
    ) -> Result<HttpResponse, TransportError> {
        let mut request = self
            .agent
            .post(url)
            .config()
            .timeout_global(Some(timeout))
            .build()
            .header("content-type", "application/json");
        for (name, value) in headers {
            request = request.header(*name, *value);
        }
        let mut response = request.send(body).map_err(|e| match e {
            ureq::Error::Timeout(_) => TransportError::Timeout,
            e => TransportError::Network(e.to_string()),
        })?;
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map(Duration::from_secs);
        let status = response.status().as_u16();
        let body = response.body_mut().read_to_string().map_err(|e| match e {
            ureq::Error::Timeout(_) => TransportError::Timeout,
            e => TransportError::Network(e.to_string()),
        })?;
        Ok(HttpResponse {
            status,
            retry_after,
            body,
        })
    }
}

/// How long one request may take. Generous: a long transcript with a thinking model takes a
/// while, and a timeout is not retried.
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(180);

/// When to try a failed request again.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryPolicy {
    /// Waits before each retry after a server error or a lost connection.
    pub backoff: Vec<Duration>,
    /// Wait after a 429 without a `retry-after` header.
    pub rate_limit_wait: Duration,
    /// 429s tolerated per request. They do not use up `backoff`: a long job on a new key
    /// reaches the per-minute limit easily and should slow down, not fail.
    pub max_rate_limit_waits: usize,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            backoff: vec![
                Duration::from_secs(2),
                Duration::from_secs(8),
                Duration::from_secs(30),
            ],
            rate_limit_wait: Duration::from_secs(30),
            max_rate_limit_waits: 10,
        }
    }
}

/// Claude through the Anthropic Messages API (raw HTTP: there is no official Rust SDK).
pub struct AnthropicProvider<T: HttpTransport = UreqTransport> {
    api_key: String,
    transport: T,
    retry: RetryPolicy,
}

impl AnthropicProvider {
    pub fn new(api_key: &str) -> Self {
        Self::with_transport(api_key, UreqTransport::default(), RetryPolicy::default())
    }
}

const MESSAGES_URL: &str = "https://api.anthropic.com/v1/messages";
/// Re-runs a request a safety classifier declined on Anthropic's recommended fallback model.
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

impl<T: HttpTransport> AnthropicProvider<T> {
    pub fn with_transport(api_key: &str, transport: T, retry: RetryPolicy) -> Self {
        Self {
            api_key: api_key.trim().to_string(),
            transport,
            retry,
        }
    }

    /// The request body for `request`.
    pub fn body(request: &AiRequest) -> Value {
        let mut output_config = json!({
            "format": {"type": "json_schema", "schema": request.schema},
        });
        let mut body = json!({
            "model": request.model.id(),
            "max_tokens": 16000,
            "messages": [{"role": "user", "content": request.prompt}],
        });
        match request.model {
            // A summary is a simple task: low effort keeps the thinking short and cheap.
            AiModel::ClaudeOpus5 => {
                output_config["effort"] = json!("low");
                body["fallbacks"] = json!("default");
            }
            AiModel::ClaudeSonnet5 => output_config["effort"] = json!("low"),
            // Haiku 4.5 does not take `effort` and does not think unless asked.
            AiModel::ClaudeHaiku45 => {}
        }
        body["output_config"] = output_config;
        body
    }

    fn send(&self, request: &AiRequest, body: &str) -> Result<HttpResponse, TransportError> {
        let mut headers = vec![
            ("x-api-key", self.api_key.as_str()),
            ("anthropic-version", "2023-06-01"),
        ];
        if request.model == AiModel::ClaudeOpus5 {
            headers.push(("anthropic-beta", FALLBACK_BETA));
        }
        self.transport
            .post_json(MESSAGES_URL, &headers, body, REQUEST_TIMEOUT)
    }
}

impl<T: HttpTransport> AiProvider for AnthropicProvider<T> {
    fn complete(&self, request: &AiRequest, cancel: &AtomicBool) -> Result<AiResponse, AiError> {
        let body = Self::body(request).to_string();
        let mut retries = self.retry.backoff.iter();
        let mut rate_limit_waits = 0;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(AiError::Cancelled);
            }
            let wait = match self.send(request, &body) {
                Err(TransportError::Timeout) => return Err(AiError::Timeout),
                Err(TransportError::Network(why)) => {
                    log::warn!("ai: request failed: {why}");
                    retries.next().copied().ok_or(AiError::Unavailable(why))?
                }
                Ok(response) => match classify(&response) {
                    Outcome::Answer => return parse_answer(&response.body),
                    Outcome::Fail(error) => return Err(error),
                    Outcome::RateLimited => {
                        rate_limit_waits += 1;
                        if rate_limit_waits > self.retry.max_rate_limit_waits {
                            return Err(AiError::Unavailable("rate limit".to_string()));
                        }
                        log::info!("ai: rate limited, waiting");
                        response.retry_after.unwrap_or(self.retry.rate_limit_wait)
                    }
                    Outcome::Retry => {
                        log::warn!("ai: server error {}", response.status);
                        retries.next().copied().ok_or_else(|| {
                            AiError::Unavailable(format!("server error {}", response.status))
                        })?
                    }
                },
            };
            sleep_unless_cancelled(wait, cancel)?;
        }
    }
}

enum Outcome {
    Answer,
    Retry,
    RateLimited,
    Fail(AiError),
}

fn classify(response: &HttpResponse) -> Outcome {
    let message = || error_message(&response.body);
    match response.status {
        200 => Outcome::Answer,
        401 | 403 => Outcome::Fail(AiError::KeyRejected),
        402 => Outcome::Fail(AiError::NoCredit),
        429 => Outcome::RateLimited,
        // 529 is Anthropic's "overloaded".
        500 | 502 | 503 | 504 | 529 => Outcome::Retry,
        400 if message().to_lowercase().contains("credit balance") => {
            Outcome::Fail(AiError::NoCredit)
        }
        _ => Outcome::Fail(AiError::BadRequest(message())),
    }
}

/// The `error.message` of an error response, or the status line's body as is.
fn error_message(body: &str) -> String {
    serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(str::to_string))
        .unwrap_or_else(|| body.chars().take(200).collect())
}

/// Read a successful Messages API response: the JSON in its text block and the usage.
fn parse_answer(body: &str) -> Result<AiResponse, AiError> {
    let value: Value =
        serde_json::from_str(body).map_err(|e| AiError::InvalidAnswer(e.to_string()))?;
    match value["stop_reason"].as_str() {
        Some("end_turn") => {}
        Some("refusal") => return Err(AiError::Refused),
        Some("max_tokens") => return Err(AiError::Truncated),
        other => {
            return Err(AiError::InvalidAnswer(format!(
                "stop reason {}",
                other.unwrap_or("missing")
            )))
        }
    }
    let text = value["content"]
        .as_array()
        .and_then(|blocks| {
            blocks
                .iter()
                .rev()
                .find(|b| b["type"] == "text")
                .and_then(|b| b["text"].as_str())
        })
        .ok_or_else(|| AiError::InvalidAnswer("no text in the answer".to_string()))?;
    let json = serde_json::from_str(text).map_err(|e| AiError::InvalidAnswer(e.to_string()))?;
    let usage = AiUsage {
        input_tokens: value["usage"]["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: value["usage"]["output_tokens"].as_u64().unwrap_or(0),
    };
    Ok(AiResponse { json, usage })
}

/// Sleep `wait` in short steps, returning early with `Cancelled` once `cancel` is set.
fn sleep_unless_cancelled(wait: Duration, cancel: &AtomicBool) -> Result<(), AiError> {
    const STEP: Duration = Duration::from_millis(250);
    let mut left = wait;
    while !left.is_zero() {
        if cancel.load(Ordering::Relaxed) {
            return Err(AiError::Cancelled);
        }
        let step = left.min(STEP);
        std::thread::sleep(step);
        left -= step;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Headers and body of one request.
    type Sent = (Vec<(String, String)>, String);

    /// Answers requests from a script and records what was sent.
    struct FakeTransport {
        replies: Mutex<Vec<Result<HttpResponse, TransportError>>>,
        sent: Mutex<Vec<Sent>>,
    }

    impl FakeTransport {
        fn new(mut replies: Vec<Result<HttpResponse, TransportError>>) -> Self {
            replies.reverse();
            Self {
                replies: Mutex::new(replies),
                sent: Mutex::new(Vec::new()),
            }
        }

        fn requests(&self) -> usize {
            self.sent.lock().map_or(0, |s| s.len())
        }
    }

    impl HttpTransport for FakeTransport {
        fn post_json(
            &self,
            _url: &str,
            headers: &[(&str, &str)],
            body: &str,
            _timeout: Duration,
        ) -> Result<HttpResponse, TransportError> {
            if let Ok(mut sent) = self.sent.lock() {
                let headers = headers
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                sent.push((headers, body.to_string()));
            }
            self.replies
                .lock()
                .ok()
                .and_then(|mut r| r.pop())
                .unwrap_or(Err(TransportError::Network("no reply scripted".into())))
        }
    }

    fn reply(status: u16, body: &str) -> Result<HttpResponse, TransportError> {
        Ok(HttpResponse {
            status,
            retry_after: None,
            body: body.to_string(),
        })
    }

    fn answer(stop_reason: &str) -> Result<HttpResponse, TransportError> {
        let body = json!({
            "stop_reason": stop_reason,
            "content": [{"type": "text", "text": "{\"summary\":\"Hi\",\"segments\":[]}"}],
            "usage": {"input_tokens": 1200, "output_tokens": 80},
        });
        reply(200, &body.to_string())
    }

    fn no_waits() -> RetryPolicy {
        RetryPolicy {
            backoff: vec![Duration::ZERO; 3],
            rate_limit_wait: Duration::ZERO,
            max_rate_limit_waits: 2,
        }
    }

    fn request(model: AiModel) -> AiRequest {
        AiRequest {
            model,
            prompt: "Summarize".to_string(),
            schema: json!({"type": "object"}),
        }
    }

    fn run(
        replies: Vec<Result<HttpResponse, TransportError>>,
    ) -> (Result<AiResponse, AiError>, usize) {
        let provider =
            AnthropicProvider::with_transport(" key ", FakeTransport::new(replies), no_waits());
        let result = provider.complete(&request(AiModel::ClaudeHaiku45), &AtomicBool::new(false));
        (result, provider.transport.requests())
    }

    #[test]
    fn a_good_answer_is_parsed_with_its_usage() {
        let (result, requests) = run(vec![answer("end_turn")]);
        let response = result.expect("answer");
        assert_eq!(response.json["summary"], "Hi");
        assert_eq!(
            response.usage,
            AiUsage {
                input_tokens: 1200,
                output_tokens: 80
            }
        );
        assert_eq!(requests, 1);
    }

    #[test]
    fn server_errors_and_lost_connections_are_retried_three_times() {
        let (result, requests) = run(vec![
            reply(529, "{}"),
            Err(TransportError::Network("reset".into())),
            reply(500, "{}"),
            answer("end_turn"),
        ]);
        assert!(result.is_ok());
        assert_eq!(requests, 4);

        let (result, requests) = run(vec![reply(503, "{}"); 5]);
        assert!(matches!(result, Err(AiError::Unavailable(_))));
        assert_eq!(requests, 4, "one try and three retries");
    }

    #[test]
    fn rate_limits_wait_without_using_up_the_retries() {
        let (result, requests) = run(vec![
            reply(429, "{}"),
            reply(529, "{}"),
            reply(429, "{}"),
            reply(529, "{}"),
            reply(529, "{}"),
            answer("end_turn"),
        ]);
        assert!(result.is_ok());
        assert_eq!(requests, 6);

        let (result, _) = run(vec![reply(429, "{}"); 5]);
        assert!(matches!(result, Err(AiError::Unavailable(_))));
    }

    #[test]
    fn key_and_credit_errors_stop_the_job_other_400s_fail_the_file() {
        let (result, requests) = run(vec![reply(401, "{}")]);
        assert_eq!(result, Err(AiError::KeyRejected));
        assert_eq!(requests, 1, "not retried");
        assert!(AiError::KeyRejected.stops_job());

        let credit = json!({"type": "error", "error": {"type": "invalid_request_error",
            "message": "Your credit balance is too low to access the Anthropic API."}});
        let (result, _) = run(vec![reply(400, &credit.to_string())]);
        assert_eq!(result, Err(AiError::NoCredit));
        assert!(AiError::NoCredit.stops_job());

        let other = json!({"error": {"message": "prompt is too long"}});
        let (result, _) = run(vec![reply(400, &other.to_string())]);
        assert_eq!(
            result,
            Err(AiError::BadRequest("prompt is too long".to_string()))
        );
        assert!(!AiError::BadRequest(String::new()).stops_job());
    }

    #[test]
    fn a_timeout_is_not_retried() {
        let (result, requests) = run(vec![Err(TransportError::Timeout), answer("end_turn")]);
        assert_eq!(result, Err(AiError::Timeout));
        assert_eq!(requests, 1);
    }

    #[test]
    fn refusals_and_cut_off_answers_fail() {
        assert_eq!(run(vec![answer("refusal")]).0, Err(AiError::Refused));
        assert_eq!(run(vec![answer("max_tokens")]).0, Err(AiError::Truncated));
    }

    #[test]
    fn a_cancel_stops_before_the_next_try() {
        let provider = AnthropicProvider::with_transport(
            "key",
            FakeTransport::new(vec![reply(529, "{}"), answer("end_turn")]),
            RetryPolicy {
                backoff: vec![Duration::from_secs(60)],
                ..no_waits()
            },
        );
        let cancel = AtomicBool::new(true);
        let result = provider.complete(&request(AiModel::ClaudeHaiku45), &cancel);
        assert_eq!(result, Err(AiError::Cancelled));
        assert_eq!(provider.transport.requests(), 0);
    }

    #[test]
    fn the_request_matches_what_each_model_takes() {
        let provider = AnthropicProvider::with_transport(
            " sk-ant \n",
            FakeTransport::new(vec![answer("end_turn")]),
            no_waits(),
        );
        let _ = provider.complete(&request(AiModel::ClaudeOpus5), &AtomicBool::new(false));
        let sent = provider.transport.sent.lock().expect("sent");
        let (headers, body) = &sent[0];
        assert!(headers.contains(&("x-api-key".to_string(), "sk-ant".to_string())));
        assert!(headers.contains(&("anthropic-beta".to_string(), FALLBACK_BETA.to_string())));
        let body: Value = serde_json::from_str(body).expect("json");
        assert_eq!(body["model"], "claude-opus-5");
        assert_eq!(body["fallbacks"], "default");
        assert_eq!(body["output_config"]["effort"], "low");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");

        let haiku = AnthropicProvider::<UreqTransport>::body(&request(AiModel::ClaudeHaiku45));
        assert!(haiku["output_config"].get("effort").is_none());
        assert!(haiku.get("fallbacks").is_none());
    }

    #[test]
    fn usage_costs_follow_the_price_table() {
        let usage = AiUsage {
            input_tokens: 1_000_000,
            output_tokens: 100_000,
        };
        assert!((usage.cost(AiModel::ClaudeHaiku45) - 1.5).abs() < 1e-9);
        assert!((usage.cost(AiModel::ClaudeOpus5) - 7.5).abs() < 1e-9);
    }
}
