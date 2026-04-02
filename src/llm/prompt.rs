//! Build LLM prompts from `Problem` fields and parse responses into
//! `SourcePatternSet`. All regex patterns are validated before returning.

use anyhow::{bail, Context, Result};
use regex::Regex;

use crate::problems::schema::{Confidence, Problem, SourcePattern, SourcePatternSet};

/// A single message in the chat completion request.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Ecosystem → default languages for source patterns.
fn ecosystem_languages(ecosystem: &str) -> Vec<String> {
    match ecosystem {
        "npm" => vec!["js".into(), "ts".into()],
        "pip" => vec!["py".into()],
        "go" => vec!["go".into()],
        "cargo" => vec!["rs".into()],
        _ => vec!["js".into(), "ts".into(), "py".into()],
    }
}

/// Build the system + user messages for generating source patterns.
pub fn build_messages(problem: &Problem) -> Vec<ChatMessage> {
    let system = SYSTEM_PROMPT.to_string();

    let refs = if problem.references.is_empty() {
        "None".to_string()
    } else {
        problem.references.join("\n")
    };

    let user = format!(
        "Generate deep-scan regex patterns for this vulnerability:\n\n\
         ID: {id}\n\
         Title: {title}\n\
         Severity: {severity}\n\
         Ecosystem: {ecosystem}\n\
         Package: {package}\n\
         Affected range: {range}\n\
         Fixed in: {fixed}\n\
         References:\n{refs}",
        id = problem.id,
        title = problem.title,
        severity = problem.severity,
        ecosystem = problem.ecosystem,
        package = problem.package,
        range = problem.affected_range,
        fixed = problem.fixed_in.as_deref().unwrap_or("unknown"),
        refs = refs,
    );

    vec![
        ChatMessage {
            role: "system".into(),
            content: system,
        },
        ChatMessage {
            role: "user".into(),
            content: user,
        },
    ]
}

/// Parse the LLM's JSON response into a validated `SourcePatternSet`.
///
/// Expects the response to contain a JSON object (possibly inside a
/// markdown code fence) with the shape:
/// ```json
/// {
///   "patterns": [
///     {
///       "description": "...",
///       "regex": "...",
///       "confidence": "definite|likely|possible",
///       "remediation": "..."
///     }
///   ]
/// }
/// ```
pub fn parse_response(raw: &str, ecosystem: &str) -> Result<SourcePatternSet> {
    let json_str = extract_json(raw).context("no JSON object found in LLM response")?;

    let parsed: RawResponse =
        serde_json::from_str(&json_str).context("LLM response is not valid JSON")?;

    if parsed.patterns.is_empty() {
        bail!("LLM returned zero patterns");
    }

    let mut validated = Vec::new();
    for raw_pat in &parsed.patterns {
        // Validate regex compiles.
        if Regex::new(&raw_pat.regex).is_err() {
            continue; // skip broken regexes silently
        }
        validated.push(SourcePattern {
            description: raw_pat.description.clone(),
            regex: raw_pat.regex.clone(),
            confidence: parse_confidence(&raw_pat.confidence),
            remediation: raw_pat.remediation.clone(),
        });
    }

    if validated.is_empty() {
        bail!("all LLM-generated regex patterns failed to compile");
    }

    Ok(SourcePatternSet {
        languages: ecosystem_languages(ecosystem),
        patterns: validated,
    })
}

// ── Internal helpers ──────────────────────────────────────────────────

/// Extract a JSON object from text that may contain markdown fences.
fn extract_json(raw: &str) -> Option<String> {
    let trimmed = raw.trim();

    // Try stripping ```json ... ``` fences first.
    if let Some(start) = trimmed.find("```") {
        let after_fence = &trimmed[start + 3..];
        // Skip the optional language tag on the same line.
        let body_start = after_fence.find('\n').unwrap_or(0) + 1;
        let body = &after_fence[body_start..];
        if let Some(end) = body.find("```") {
            return Some(body[..end].trim().to_string());
        }
    }

    // Try finding bare { ... }.
    let start = trimmed.find('{')?;
    let end = trimmed.rfind('}')?;
    if end > start {
        return Some(trimmed[start..=end].to_string());
    }

    None
}

fn parse_confidence(s: &str) -> Confidence {
    match s.to_lowercase().as_str() {
        "definite" => Confidence::Definite,
        "likely" => Confidence::Likely,
        _ => Confidence::Possible,
    }
}

/// Raw deserialization target for the LLM JSON response.
#[derive(serde::Deserialize)]
struct RawResponse {
    patterns: Vec<RawPattern>,
}

#[derive(serde::Deserialize)]
struct RawPattern {
    description: String,
    regex: String,
    confidence: String,
    remediation: String,
}

const SYSTEM_PROMPT: &str = "\
You are a security expert that generates regex patterns for source code scanning.

Given a CVE / vulnerability description, generate 2-4 regex patterns that would \
detect affected usage of the vulnerable package in source code.

Rules:
- Each regex must be valid Rust `regex` crate syntax (no look-behind).
- Patterns should match ONE LINE at a time.
- Aim for a mix of confidence levels: one 'definite' pattern (very specific), \
  one or two 'likely' patterns, and optionally a broad 'possible' pattern.
- Keep regex readable — avoid catastrophic backtracking.
- Remediation should be a concise upgrade/fix instruction.

Respond with ONLY a JSON object (no markdown, no explanation):
{
  \"patterns\": [
    {
      \"description\": \"what this pattern detects\",
      \"regex\": \"the_regex_here\",
      \"confidence\": \"definite|likely|possible\",
      \"remediation\": \"how to fix\"
    }
  ]
}";

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_problem() -> Problem {
        Problem {
            id: "CVE-2023-45857".into(),
            title: "Axios CSRF token leak".into(),
            severity: "high".into(),
            ecosystem: "npm".into(),
            package: "axios".into(),
            affected_range: ">=0.8.1,<1.6.0".into(),
            fixed_in: Some("1.6.0".into()),
            references: vec!["https://example.com/advisory".into()],
            source_patterns: None,
            kind: crate::problems::schema::ProblemKind::Cve,
        }
    }

    #[test]
    fn build_messages_produces_system_and_user() {
        let msgs = build_messages(&sample_problem());
        assert_eq!(msgs.len(), 2);
        assert_eq!(msgs[0].role, "system");
        assert_eq!(msgs[1].role, "user");
        assert!(msgs[1].content.contains("CVE-2023-45857"));
        assert!(msgs[1].content.contains("axios"));
    }

    #[test]
    fn parse_response_valid_json() {
        let raw = r#"{
            "patterns": [
                {
                    "description": "axios request",
                    "regex": "axios\\.get\\(",
                    "confidence": "likely",
                    "remediation": "upgrade to >=1.6.0"
                }
            ]
        }"#;
        let result = parse_response(raw, "npm").unwrap();
        assert_eq!(result.languages, vec!["js", "ts"]);
        assert_eq!(result.patterns.len(), 1);
        assert!(matches!(result.patterns[0].confidence, Confidence::Likely));
    }

    #[test]
    fn parse_response_strips_markdown_fence() {
        let raw = "```json\n{\"patterns\":[{\"description\":\"d\",\"regex\":\"foo\",\"confidence\":\"possible\",\"remediation\":\"r\"}]}\n```";
        let result = parse_response(raw, "pip").unwrap();
        assert_eq!(result.languages, vec!["py"]);
        assert_eq!(result.patterns.len(), 1);
    }

    #[test]
    fn parse_response_skips_invalid_regex() {
        let raw = r#"{
            "patterns": [
                {
                    "description": "bad",
                    "regex": "(?<=broken)",
                    "confidence": "likely",
                    "remediation": "fix"
                },
                {
                    "description": "good",
                    "regex": "require\\(.axios",
                    "confidence": "possible",
                    "remediation": "fix"
                }
            ]
        }"#;
        let result = parse_response(raw, "npm").unwrap();
        assert_eq!(result.patterns.len(), 1);
        assert_eq!(result.patterns[0].description, "good");
    }

    #[test]
    fn parse_response_rejects_empty_patterns() {
        let raw = r#"{"patterns": []}"#;
        assert!(parse_response(raw, "npm").is_err());
    }

    #[test]
    fn parse_response_rejects_all_broken_regex() {
        let raw = r#"{"patterns":[{"description":"d","regex":"(?<=x)","confidence":"likely","remediation":"r"}]}"#;
        assert!(parse_response(raw, "npm").is_err());
    }

    #[test]
    fn extract_json_bare_object() {
        let input = "  some preamble {\"a\":1} trailing  ";
        assert_eq!(extract_json(input).unwrap(), "{\"a\":1}");
    }

    #[test]
    fn ecosystem_languages_defaults() {
        assert_eq!(ecosystem_languages("npm"), vec!["js", "ts"]);
        assert_eq!(ecosystem_languages("pip"), vec!["py"]);
        assert_eq!(ecosystem_languages("go"), vec!["go"]);
        assert_eq!(ecosystem_languages("cargo"), vec!["rs"]);
        assert_eq!(ecosystem_languages("unknown").len(), 3); // fallback
    }
}
