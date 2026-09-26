//! Claude through Anthropic's Messages API, over plain HTTP (there is no official Rust SDK).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use base64::Engine;
use serde_json::{json, Value};

use super::provider::{AiContent, AiError, AiProvider, AiRequest, AiResponse, AiUsage};

const API_URL: &str = "https://api.anthropic.com";
const API_VERSION: &str = "2023-06-01";
/// A request that has not answered by then fails the file; see [`AiError::Timeout`].
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);

/// How failed requests are retried.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Waits before each retry of a server error or a lost connection; one retry per entry.
    pub delays: Vec<Duration>,
    /// Wait after a 429 without a `retry-after` header. A 429 uses up no retry: a long job
    /// reaching a new key's per-minute limit should slow down, not fail.
    pub rate_limit_wait: Duration,
    /// Waits are slept in steps this long, checking the cancel flag between them.
    pub step: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            delays: vec![
                Duration::from_secs(2),
                Duration::from_secs(8),
                Duration::from_secs(30),
            ],
            rate_limit_wait: Duration::from_secs(30),
            step: Duration::from_millis(250),
        }
    }
}

/// The Anthropic provider, for one key.
pub struct Anthropic {
    key: String,
    base_url: String,
    retry: RetryPolicy,
    client: reqwest::blocking::Client,
}

impl Anthropic {
    pub fn new(key: String) -> Result<Self, AiError> {
        Self::with_endpoint(key, API_URL.to_string(), RetryPolicy::default())
    }

    /// A provider talking to `base_url` (tests use a local server).
    pub fn with_endpoint(
        key: String,
        base_url: String,
        retry: RetryPolicy,
    ) -> Result<Self, AiError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .map_err(|e| AiError::Network(e.to_string()))?;
        Ok(Self {
            key,
            base_url,
            retry,
            client,
        })
    }

    fn body(request: &AiRequest) -> Value {
        let content: Vec<Value> = request
            .content
            .iter()
            .map(|block| match block {
                AiContent::Text(text) => json!({"type": "text", "text": text}),
                AiContent::Jpeg(bytes) => json!({
                    "type": "image",
                    "source": {
                        "type": "base64",
                        "media_type": "image/jpeg",
                        "data": base64::engine::general_purpose::STANDARD.encode(bytes),
                    }
                }),
            })
            .collect();
        json!({
            "model": request.model,
            "max_tokens": request.max_tokens,
            "messages": [{"role": "user", "content": content}],
            "output_config": {"format": {"type": "json_schema", "schema": request.schema}},
        })
    }

    /// Sleep `wait` in steps, returning `Cancelled` as soon as the flag is set.
    fn wait(&self, wait: Duration, cancel: &AtomicBool) -> Result<(), AiError> {
        let mut left = wait;
        while !left.is_zero() {
            if cancel.load(Ordering::Relaxed) {
                return Err(AiError::Cancelled);
            }
            let step = left.min(self.retry.step);
            std::thread::sleep(step);
            left -= step;
        }
        if cancel.load(Ordering::Relaxed) {
            return Err(AiError::Cancelled);
        }
        Ok(())
    }
}

/// What to do after one attempt.
enum Attempt {
    Done(Result<AiResponse, AiError>),
    /// Retry after the policy's next delay, if any is left.
    Retry(String),
    /// Wait this long, then try again without using up a retry.
    RateLimited(Duration),
}

impl AiProvider for Anthropic {
    fn complete(&self, request: &AiRequest, cancel: &AtomicBool) -> Result<AiResponse, AiError> {
        let body = Self::body(request);
        let mut retries = self.retry.delays.iter();
        loop {
            match self.attempt(&body) {
                Attempt::Done(result) => return result,
                Attempt::RateLimited(wait) => {
                    log::info!("ai: rate limited, waiting {} s", wait.as_secs());
                    self.wait(wait, cancel)?;
                }
                Attempt::Retry(why) => {
                    let Some(delay) = retries.next() else {
                        return Err(AiError::Network(why));
                    };
                    log::warn!("ai: {why}; retrying in {} s", delay.as_secs());
                    self.wait(*delay, cancel)?;
                }
            }
        }
    }
}

impl Anthropic {
    fn attempt(&self, body: &Value) -> Attempt {
        let sent = self
            .client
            .post(format!("{}/v1/messages", self.base_url))
            .header("x-api-key", &self.key)
            .header("anthropic-version", API_VERSION)
            .json(body)
            .send();
        let response = match sent {
            Ok(response) => response,
            Err(e) if e.is_timeout() => return Attempt::Done(Err(AiError::Timeout)),
            Err(e) => return Attempt::Retry(format!("connection failed: {e}")),
        };
        let status = response.status().as_u16();
        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.trim().parse::<u64>().ok())
            .map(Duration::from_secs);
        let text = match response.text() {
            Ok(text) => text,
            Err(e) if e.is_timeout() => return Attempt::Done(Err(AiError::Timeout)),
            Err(e) => return Attempt::Retry(format!("reading the answer failed: {e}")),
        };
        classify(status, retry_after, &text, self.retry.rate_limit_wait)
    }
}

/// Turn one HTTP answer into the next step.
fn classify(
    status: u16,
    retry_after: Option<Duration>,
    text: &str,
    rate_limit_wait: Duration,
) -> Attempt {
    let json: Value = serde_json::from_str(text).unwrap_or(Value::Null);
    let message = json["error"]["message"]
        .as_str()
        .map(str::to_string)
        .unwrap_or_else(|| format!("HTTP {status}"));
    match status {
        200 => Attempt::Done(parse_message(&json)),
        401 | 403 => Attempt::Done(Err(AiError::KeyRejected)),
        402 => Attempt::Done(Err(AiError::OutOfCredit)),
        400 if message.to_lowercase().contains("credit balance") => {
            Attempt::Done(Err(AiError::OutOfCredit))
        }
        429 => Attempt::RateLimited(retry_after.unwrap_or(rate_limit_wait)),
        500 | 502 | 503 | 529 => Attempt::Retry(format!("HTTP {status}: {message}")),
        _ => Attempt::Done(Err(AiError::Rejected(message))),
    }
}

/// Read a Messages API answer: the JSON in its text block, why it stopped, and its usage.
fn parse_message(json: &Value) -> Result<AiResponse, AiError> {
    let usage = AiUsage {
        input_tokens: json["usage"]["input_tokens"].as_u64().unwrap_or(0),
        output_tokens: json["usage"]["output_tokens"].as_u64().unwrap_or(0),
    };
    let stop_reason = json["stop_reason"].as_str().unwrap_or_default().to_string();
    let text: String = json["content"]
        .as_array()
        .map(|blocks| {
            blocks
                .iter()
                .filter(|b| b["type"] == "text")
                .filter_map(|b| b["text"].as_str())
                .collect()
        })
        .unwrap_or_default();
    // A model that stopped early (max_tokens, refusal) may have written no valid JSON; the
    // caller fails the file with the stop reason and still counts the usage.
    let answer = serde_json::from_str(&text).unwrap_or(Value::Null);
    if answer.is_null() && stop_reason == "end_turn" {
        return Err(AiError::BadAnswer(format!("not JSON: {text}")));
    }
    Ok(AiResponse {
        json: answer,
        stop_reason,
        usage,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Read, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    /// A local HTTP server answering each request with the next canned response, recording
    /// how many it got.
    fn server(responses: Vec<String>) -> (String, Arc<Mutex<usize>>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let url = format!("http://{}", listener.local_addr().expect("addr"));
        let count = Arc::new(Mutex::new(0));
        let seen = count.clone();
        std::thread::spawn(move || {
            for response in responses {
                let Ok((stream, _)) = listener.accept() else {
                    return;
                };
                let mut reader = BufReader::new(stream);
                let mut length = 0;
                loop {
                    let mut line = String::new();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 || line == "\r\n" {
                        break;
                    }
                    if let Some(v) = line.to_lowercase().strip_prefix("content-length:") {
                        length = v.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0; length];
                let _ = reader.read_exact(&mut body);
                *seen.lock().expect("lock") += 1;
                let _ = reader.get_mut().write_all(response.as_bytes());
            }
        });
        (url, count)
    }

    fn http(status: &str, headers: &str, body: &str) -> String {
        format!(
            "HTTP/1.1 {status}\r\n{headers}content-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
            body.len()
        )
    }

    fn ok() -> String {
        http(
            "200 OK",
            "",
            r#"{"content":[{"type":"text","text":"{\"summary\":\"S\",\"segments\":[]}"}],"stop_reason":"end_turn","usage":{"input_tokens":100,"output_tokens":20}}"#,
        )
    }

    fn fast_retries() -> RetryPolicy {
        RetryPolicy {
            delays: vec![Duration::from_millis(1); 3],
            rate_limit_wait: Duration::from_millis(1),
            step: Duration::from_millis(1),
        }
    }

    fn request() -> AiRequest {
        AiRequest {
            model: "claude-haiku-4-5".to_string(),
            content: vec![
                AiContent::Text("hi".to_string()),
                AiContent::Jpeg(vec![1, 2]),
            ],
            schema: json!({"type": "object"}),
            max_tokens: 10,
        }
    }

    fn complete(responses: Vec<String>) -> (Result<AiResponse, AiError>, usize) {
        let (url, count) = server(responses);
        let provider = Anthropic::with_endpoint("k".into(), url, fast_retries()).expect("client");
        let result = provider.complete(&request(), &AtomicBool::new(false));
        let n = *count.lock().expect("lock");
        (result, n)
    }

    #[test]
    fn the_body_carries_images_and_the_schema() {
        let body = Anthropic::body(&request());
        assert_eq!(body["messages"][0]["content"][1]["source"]["data"], "AQI=");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert!(body.get("thinking").is_none());
    }

    #[test]
    fn an_answer_gives_its_json_and_usage() {
        let (result, n) = complete(vec![ok()]);
        let response = result.expect("answer");
        assert_eq!(response.json["summary"], "S");
        assert_eq!(response.usage.input_tokens, 100);
        assert_eq!(n, 1);
    }

    #[test]
    fn a_429_waits_without_using_up_a_retry() {
        let limited = || http("429 Too Many Requests", "retry-after: 0\r\n", "{}");
        let (result, n) = complete(vec![
            limited(),
            limited(),
            limited(),
            limited(),
            limited(),
            ok(),
        ]);
        assert!(result.is_ok());
        assert_eq!(n, 6);
    }

    #[test]
    fn overloaded_is_retried_three_times_then_fails() {
        let overloaded = || {
            http(
                "529 Overloaded",
                "",
                r#"{"error":{"message":"Overloaded"}}"#,
            )
        };
        let (result, n) = complete(vec![overloaded(), overloaded(), ok()]);
        assert!(result.is_ok());
        assert_eq!(n, 3);
        let (result, n) = complete(vec![overloaded(); 4]);
        assert!(matches!(result, Err(AiError::Network(_))));
        assert_eq!(n, 4);
    }

    #[test]
    fn a_rejected_key_or_no_credit_stops_the_job() {
        let (result, _) = complete(vec![http("401 Unauthorized", "", "{}")]);
        assert_eq!(result, Err(AiError::KeyRejected));
        assert!(AiError::KeyRejected.stops_job().is_some());
        let credit = r#"{"error":{"message":"Your credit balance is too low to access the Anthropic API."}}"#;
        let (result, _) = complete(vec![http("400 Bad Request", "", credit)]);
        assert_eq!(result, Err(AiError::OutOfCredit));
    }

    #[test]
    fn another_400_fails_the_file_with_the_apis_message() {
        let body = r#"{"error":{"message":"image too large"}}"#;
        let (result, n) = complete(vec![http("400 Bad Request", "", body)]);
        assert_eq!(
            result,
            Err(AiError::Rejected("image too large".to_string()))
        );
        assert!(AiError::Rejected(String::new()).stops_job().is_none());
        assert_eq!(n, 1);
    }

    #[test]
    fn a_cancel_during_a_wait_ends_the_request() {
        let (url, _) = server(vec![http(
            "429 Too Many Requests",
            "retry-after: 30\r\n",
            "{}",
        )]);
        let provider = Anthropic::with_endpoint("k".into(), url, fast_retries()).expect("client");
        let cancel = Arc::new(AtomicBool::new(false));
        let flag = cancel.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            flag.store(true, Ordering::Relaxed);
        });
        let started = std::time::Instant::now();
        assert_eq!(
            provider.complete(&request(), &cancel),
            Err(AiError::Cancelled)
        );
        assert!(started.elapsed() < Duration::from_secs(5));
    }

    #[test]
    fn a_stop_before_the_answer_keeps_the_usage() {
        let body = r#"{"content":[{"type":"text","text":"{\"summ"}],"stop_reason":"max_tokens","usage":{"input_tokens":5,"output_tokens":4000}}"#;
        let (result, _) = complete(vec![http("200 OK", "", body)]);
        let response = result.expect("response");
        assert_eq!(response.stop_reason, "max_tokens");
        assert!(response.json.is_null());
        assert_eq!(response.usage.output_tokens, 4000);
    }
}
