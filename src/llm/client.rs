//! Thin HTTP client for OpenAI-compatible chat completion endpoints.
//!
//! Uses `ureq` (already a dep) so we don't add new dependencies.
//! Note: we use `into_string()` + `serde_json::from_str()` instead of
//! `into_json()` because the project does not enable ureq's `json` feature.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::llm::prompt::ChatMessage;
use crate::llm::LlmConfig;

/// Timeout for the LLM HTTP request (seconds).
const TIMEOUT_SECS: u64 = 30;

/// Send a chat completion request and return the assistant's text content.
pub fn chat_completion(config: &LlmConfig, messages: &[ChatMessage]) -> Result<String> {
    let body = RequestBody {
        model: &config.model,
        messages,
        temperature: 0.2,
    };

    let json_body = serde_json::to_string(&body)?;

    let response = ureq::post(&config.endpoint)
        .set("Authorization", &format!("Bearer {}", config.api_key))
        .set("Content-Type", "application/json")
        .timeout(std::time::Duration::from_secs(TIMEOUT_SECS))
        .send_string(&json_body);

    let response = match response {
        Ok(r) => r,
        Err(ureq::Error::Status(status, resp)) => {
            let body = resp.into_string().unwrap_or_default();
            bail!("LLM API returned {status}: {body}");
        }
        Err(e) => bail!("LLM API request failed: {e}"),
    };

    let text = response
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
