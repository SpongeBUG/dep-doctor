use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::log_warn;

const OSV_BATCH_URL: &str = "https://api.osv.dev/v1/querybatch";

/// Maximum pagination rounds to prevent infinite loops on malformed responses.
const MAX_PAGES: usize = 20;

// ── Request types ──────────────────────────────────────────────

#[derive(Serialize)]
pub struct BatchRequest {
    pub queries: Vec<Query>,
}

#[derive(Serialize)]
pub struct Query {
    pub version: String,
    pub package: QueryPackage,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_token: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct QueryPackage {
    pub name: String,
    pub ecosystem: String,
}

// ── Response types ─────────────────────────────────────────────

#[derive(Deserialize)]
pub struct BatchResponse {
    pub results: Vec<QueryResult>,
}

#[derive(Deserialize)]
pub struct QueryResult {
    #[serde(default)]
    pub vulns: Vec<Advisory>,
    #[serde(default)]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    #[serde(default)]
    pub summary: Option<String>,
    #[serde(default)]
    pub severity: Vec<Severity>,
    #[serde(default)]
    pub affected: Vec<Affected>,
    #[serde(default)]
    pub references: Vec<Reference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Severity {
    #[serde(default)]
    pub score: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Affected {
    #[serde(default)]
    pub ranges: Vec<Range>,
    #[serde(default)]
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    #[serde(rename = "type")]
    pub range_type: String,
    #[serde(default)]
    pub events: Vec<Event>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    #[serde(default)]
    pub introduced: Option<String>,
    #[serde(default)]
    pub fixed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    #[serde(default)]
    pub url: Option<String>,
}

// ── HTTP call with pagination ──────────────────────────────────

/// Post a batch of versioned queries to OSV.dev with automatic pagination.
///
/// Each query in the batch may paginate independently via `next_page_token`.
/// We loop until all queries are complete (no remaining tokens), merging
/// advisories across pages.
pub fn query_batch(queries: &[Query]) -> Result<Vec<QueryResult>> {
    if queries.is_empty() {
        return Ok(Vec::new());
    }

    // Accumulate advisories per original query index.
    let mut accumulated: Vec<Vec<Advisory>> = vec![Vec::new(); queries.len()];

    // Track which queries still need pagination.
    // Each entry: (original_index, page_token).
    let mut pending: Vec<(usize, Option<String>)> = (0..queries.len()).map(|i| (i, None)).collect();

    for page in 0..MAX_PAGES {
        if pending.is_empty() {
            break;
        }

        let batch_queries: Vec<Query> = pending
            .iter()
            .map(|(idx, token)| Query {
                version: queries[*idx].version.clone(),
                package: QueryPackage {
                    name: queries[*idx].package.name.clone(),
                    ecosystem: queries[*idx].package.ecosystem.clone(),
                },
                page_token: token.clone(),
            })
            .collect();

        let results = match send_batch(&batch_queries) {
            Ok(r) => r,
            Err(e) => {
                log_warn!("OSV API request failed (page {page}): {e}");
                break;
            }
        };

        let mut next_pending = Vec::new();
        for (result_idx, result) in results.into_iter().enumerate() {
            if result_idx >= pending.len() {
                break;
            }

            let original_idx = pending[result_idx].0;
            accumulated[original_idx].extend(result.vulns);

            if let Some(token) = result.next_page_token {
                next_pending.push((original_idx, Some(token)));
            }
        }

        pending = next_pending;
    }

    if !pending.is_empty() {
        log_warn!(
            "OSV pagination: {} queries hit the {MAX_PAGES}-page limit",
            pending.len(),
        );
    }

    Ok(accumulated
        .into_iter()
        .map(|vulns| QueryResult {
            vulns,
            next_page_token: None,
        })
        .collect())
}

/// Send a single HTTP request to the OSV batch endpoint.
fn send_batch(queries: &[Query]) -> Result<Vec<QueryResult>> {
    let body = BatchRequest {
        queries: queries
            .iter()
            .map(|q| Query {
                version: q.version.clone(),
                package: QueryPackage {
                    name: q.package.name.clone(),
                    ecosystem: q.package.ecosystem.clone(),
                },
                page_token: q.page_token.clone(),
            })
            .collect(),
    };

    let json_body = serde_json::to_string(&body)?;

    let resp = ureq::post(OSV_BATCH_URL)
        .set("Content-Type", "application/json")
        .send_string(&json_body);

    match resp {
        Ok(r) => {
            let text = r.into_string()?;
            let batch: BatchResponse = serde_json::from_str(&text)?;
            Ok(batch.results)
        }
        Err(e) => {
            anyhow::bail!("OSV batch request failed: {e}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_with_page_token_serializes_correctly() {
        let q = Query {
            version: "1.0.0".into(),
            package: QueryPackage {
                name: "lodash".into(),
                ecosystem: "npm".into(),
            },
            page_token: Some("abc123".into()),
        };
        let json = serde_json::to_string(&q).unwrap();
        assert!(json.contains("page_token"));
        assert!(json.contains("abc123"));
    }

    #[test]
    fn query_without_page_token_omits_field() {
        let q = Query {
            version: "1.0.0".into(),
            package: QueryPackage {
                name: "lodash".into(),
                ecosystem: "npm".into(),
            },
            page_token: None,
        };
        let json = serde_json::to_string(&q).unwrap();
        assert!(!json.contains("page_token"));
    }

    #[test]
    fn batch_response_parses_next_page_token() {
        let json = r#"{
            "results": [
                {"vulns": [{"id": "CVE-1"}], "next_page_token": "token1"},
                {"vulns": [{"id": "CVE-2"}]}
            ]
        }"#;
        let resp: BatchResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.results.len(), 2);
        assert_eq!(resp.results[0].next_page_token.as_deref(), Some("token1"));
        assert!(resp.results[1].next_page_token.is_none());
    }

    #[test]
    fn query_batch_returns_empty_for_empty_input() {
        let result = query_batch(&[]).unwrap();
        assert!(result.is_empty());
    }
}
