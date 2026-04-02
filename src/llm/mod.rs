//! LLM-assisted source pattern generation.
//!
//! Uses an OpenAI-compatible chat API to generate `SourcePatternSet` from
//! CVE descriptions. Patterns are cached forever per problem ID.
//!
//! Configuration via environment variables:
//! - `DEP_DOCTOR_LLM_API_KEY` — required (no default)
//! - `DEP_DOCTOR_LLM_ENDPOINT` — defaults to OpenAI chat completions
//! - `DEP_DOCTOR_LLM_MODEL` — defaults to `gpt-4o-mini`
//! - `DEP_DOCTOR_LLM_RATE_LIMIT_MS` — delay between LLM calls (default: 0)

pub mod cache;
pub mod client;
pub mod prompt;
pub mod quality;

use std::thread;
use std::time::Duration;

use crate::problems::schema::{Problem, SourcePatternSet};
use crate::{log_debug, log_warn};

/// Configuration for the LLM pattern generator, read from environment.
pub struct LlmConfig {
    pub endpoint: String,
    pub api_key: String,
    pub model: String,
    /// Milliseconds to wait between consecutive LLM API calls.
    pub rate_limit_ms: u64,
}

impl LlmConfig {
    /// Build from environment variables. Returns `None` if the API key is missing.
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("DEP_DOCTOR_LLM_API_KEY").ok()?;
        if api_key.is_empty() {
            return None;
        }

        let endpoint = std::env::var("DEP_DOCTOR_LLM_ENDPOINT")
            .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".into());
        let model = std::env::var("DEP_DOCTOR_LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".into());
        let rate_limit_ms = std::env::var("DEP_DOCTOR_LLM_RATE_LIMIT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(0);

        Some(Self {
            endpoint,
            api_key,
            model,
            rate_limit_ms,
        })
    }
}

/// Generate source-scan patterns for a problem using an LLM.
///
/// 1. Returns existing hand-crafted patterns unchanged.
/// 2. Checks the disk cache (patterns are immutable per problem ID).
/// 3. Calls the LLM, validates regex output, and caches the result.
/// 4. On any failure, logs a warning and returns `None` (scan continues).
pub fn generate_patterns(problem: &Problem, config: &LlmConfig) -> Option<SourcePatternSet> {
    // Already has hand-crafted patterns — don't overwrite.
    if problem.source_patterns.is_some() {
        return problem.source_patterns.clone();
    }

    // 1. Cache hit?
    if let Some(cached) = cache::get(&problem.id) {
        log_debug!("LLM pattern cache hit for {}", problem.id);
        return Some(cached);
    }

    // 2. Rate-limit delay (applied before each API call, not cache hits).
    if config.rate_limit_ms > 0 {
        thread::sleep(Duration::from_millis(config.rate_limit_ms));
    }

    // 3. Build prompt and call LLM.
    log_debug!("Generating patterns via LLM for {}", problem.id);
    let messages = prompt::build_messages(problem);

    let raw_response = match client::chat_completion(config, &messages) {
        Ok(text) => text,
        Err(e) => {
            log_warn!("LLM call failed for {}: {e}", problem.id);
            return None;
        }
    };

    // 4. Parse and validate.
    let pattern_set = match prompt::parse_response(&raw_response, &problem.ecosystem) {
        Ok(ps) => ps,
        Err(e) => {
            log_warn!("LLM response parse failed for {}: {e}", problem.id);
            return None;
        }
    };

    // 5. Cache for future runs.
    cache::set(&problem.id, &pattern_set);
    log_debug!(
        "Cached {} patterns for {}",
        pattern_set.patterns.len(),
        problem.id,
    );

    Some(pattern_set)
}
