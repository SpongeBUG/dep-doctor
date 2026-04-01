use serde::{Deserialize, Serialize};

/// A known dependency problem (CVE, breaking change, bug).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Problem {
    pub id: String,
    pub title: String,
    pub severity: String,   // "critical" | "high" | "medium" | "low" | "info"
    pub ecosystem: String,  // "npm" | "pip" | "go" | "cargo"
    pub package: String,
    pub affected_range: String,
    pub fixed_in: Option<String>,
    pub references: Vec<String>,
    pub source_patterns: Option<SourcePatternSet>,
}

impl Problem {
    /// Numeric rank for severity comparison (higher = more severe).
    pub fn severity_rank(&self) -> u8 {
        match self.severity.as_str() {
            "critical" => 5,
            "high"     => 4,
            "medium"   => 3,
            "low"      => 2,
            _          => 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePatternSet {
    pub languages: Vec<String>,
    pub patterns: Vec<SourcePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePattern {
    pub description: String,
    pub regex: String,
    pub confidence: Confidence,
    pub remediation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    Definite,
    Likely,
    Possible,
}

// ── Runtime findings ──────────────────────────────────────────────────────────

/// A resolved (repo, package, problem) match.
#[derive(Debug, Serialize)]
pub struct Finding<'a> {
    pub repo_name: String,
    pub repo_path: String,
    pub package: String,
    pub installed_version: String,
    pub problem: &'a Problem,
    pub source_hits: Vec<SourceHit>,
}

/// One source-code location that matched a pattern during deep scan.
#[derive(Debug, Clone, Serialize)]
pub struct SourceHit {
    pub file: String,
    pub line_number: usize,
    pub line_content: String,
    pub context: Vec<String>,
    pub pattern_description: String,
    pub confidence: String,
    pub remediation: String,
}
