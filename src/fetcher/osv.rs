use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::log_warn;

const OSV_BATCH_URL: &str = "https://api.osv.dev/v1/querybatch";

// ── Request types ──────────────────────────────────────────────

#[derive(Serialize)]
pub struct BatchRequest {
    pub queries: Vec<Query>,
}

#[derive(Serialize)]
pub struct Query {
    pub version: String,
    pub package: QueryPackage,
}

#[derive(Serialize)]
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

// ── HTTP call ──────────────────────────────────────────────────

/// Post a batch of queries to OSV.dev.
/// Returns an empty vec on network error (graceful degradation).
pub fn query_batch(queries: &[Query]) -> Result<Vec<QueryResult>> {
    if queries.is_empty() {
        return Ok(Vec::new());
    }

    let body = BatchRequest {
        queries: queries
            .iter()
            .map(|q| Query {
                version: q.version.clone(),
                package: QueryPackage {
                    name: q.package.name.clone(),
                    ecosystem: q.package.ecosystem.clone(),
                },
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
            log_warn!("OSV API request failed: {e}");
            Ok(Vec::new())
        }
    }
}
