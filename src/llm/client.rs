//! Thin HTTP client for OpenAI-compatible chat completion endpoints.
//!
//! Uses `ureq` (already a dep) so we don't add new dependencies.
//! Includes retry-with-backoff on HTTP 429 (rate limit) responses.

use std::thread;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::llm::prompt::ChatMessage;
use crate::llm::LlmConfig;
use crate::log_debug;

/// Timeout for a single LLM HTTP request (seconds).
const TIMEOUT_SECS: u64 = 30;

/// Maximum retries on HTTP 429 before giving up.
const MAX_RETRIES: u32 = 3;

/// Base backoff duration (doubles each retry: 2s → 4s → 8s).
const BASE_BACKOFF: Duration = Duration::from_secs(2);

/// Send a chat completion request and return the assistant's text content.
///
/// Automatically retries on HTTP 429 (rate limited) with exponential backoff.
/// Respects the `Retry-After` header when present.
pub fn chat_completion(config: &LlmConfig, messages: &[ChatMessage]) -> Result<String> {
    let body = RequestBody {
        model: &config.model,
        messages,
        temperature: 0.2,
    };

    let json_body = serde_json::to_string(&body)?;

    for attempt in 0..=MAX_RETRIES {
        let response = ureq::post(&config.endpoint)
            .set("Authorization", &format!("Bearer {}", config.api_key))
            .set("Content-Type", "application/json")
            .timeout(Duration::from_secs(TIMEOUT_SECS))
            .send_string(&json_body);

        match response {
            Ok(r) => return parse_response(r),
            Err(ureq::Error::Status(429, resp)) => {
                if attempt == MAX_RETRIES {
                    let body = resp.into_string().unwrap_or_default();
                    bail!("LLM API rate limited after {MAX_RETRIES} retries: {body}");
                }

                let wait = retry_after_duration(&resp).unwrap_or_else(|| backoff(attempt));
                log_debug!(
                    "LLM API rate limited (429), retry {}/{MAX_RETRIES} in {}s",
                    attempt + 1,
                    wait.as_secs(),
                );
                thread::sleep(wait);
            }
            Err(ureq::Error::Status(status, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                bail!("LLM API returned {status}: {body}");
            }
            Err(e) => bail!("LLM API request failed: {e}"),
        }
    }

    bail!("LLM API: exhausted retries (unreachable)")
}

/// Parse the `Retry-After` header value (seconds) from an HTTP response.
fn retry_after_duration(resp: &ureq::Response) -> Option<Duration> {
    resp.header("Retry-After")
        .and_then(|v| v.parse::<u64>().ok())
        .map(Duration::from_secs)
}

/// Exponential backoff: 2s, 4s, 8s, …
fn backoff(attempt: u32) -> Duration {
    BASE_BACKOFF * 2u32.saturating_pow(attempt)
}

/// Extract the assistant text from a successful HTTP response.
fn parse_response(r: ureq::Response) -> Result<String> {
    let text = r
        .into_string()
        .context("failed to read LLM API response body")?;

    let resp_body: ResponseBody =
        serde_json::from_str(&text).context("failed to parse LLM API response JSON")?;

    resp_body
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .context("LLM API returned no choices")
}

// ── Request / response types (OpenAI-compatible) ─────────────────────

#[derive(Serialize)]
struct RequestBody<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    temperature: f32,
}

#[derive(Deserialize)]
struct ResponseBody {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: AssistantMessage,
}

#[derive(Deserialize)]
struct AssistantMessage {
    content: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_increases_exponentially() {
        assert_eq!(backoff(0), Duration::from_secs(2));
        assert_eq!(backoff(1), Duration::from_secs(4));
        assert_eq!(backoff(2), Duration::from_secs(8));
    }
}
