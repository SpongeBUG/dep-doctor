//! Pattern quality scoring — tracks LLM-generated pattern hit rates.
//!
//! After each deep scan, records which patterns produced hits and which
//! didn't. Persists stats to disk so quality data accumulates across runs.
//! Low-quality patterns can be identified and regenerated.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{log_debug, log_warn};

/// Per-problem hit/miss stats accumulated across scan runs.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternStats {
    /// Maps problem ID → stats for that problem's patterns.
    pub problems: HashMap<String, ProblemPatternStats>,
}

/// Stats for a single problem's source patterns.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProblemPatternStats {
    /// Number of scan runs where at least one pattern hit.
    pub runs_with_hits: u32,
    /// Number of scan runs where patterns were checked but none hit.
    pub runs_with_misses: u32,
}

impl ProblemPatternStats {
    /// Total scan runs where this problem's patterns were evaluated.
    pub fn total_runs(&self) -> u32 {
        self.runs_with_hits + self.runs_with_misses
    }

    /// Hit rate as a percentage (0.0–100.0). Returns 0.0 if no runs.
    pub fn hit_rate(&self) -> f64 {
        let total = self.total_runs();
        if total == 0 {
            return 0.0;
        }
        (self.runs_with_hits as f64 / total as f64) * 100.0
    }
}

/// Path to the stats file: `~/.cache/dep-doctor/pattern-stats.json`.
fn stats_path() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("dep-doctor").join("pattern-stats.json"))
}

/// Load pattern stats from disk. Returns empty stats if file missing or corrupt.
pub fn load() -> PatternStats {
    let Some(path) = stats_path() else {
        return PatternStats::default();
    };

    let Ok(data) = fs::read_to_string(&path) else {
        return PatternStats::default();
    };

    serde_json::from_str(&data).unwrap_or_default()
}

/// Save pattern stats to disk. Silently ignores write errors.
pub fn save(stats: &PatternStats) {
    let Some(path) = stats_path() else { return };

    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            log_warn!("Could not create stats directory");
            return;
        }
    }

    let Ok(json) = serde_json::to_string_pretty(stats) else {
        return;
    };

    if let Err(e) = fs::write(&path, json) {
        log_warn!("Could not write pattern stats: {e}");
    }
}

/// Record the result of a deep scan for a specific problem.
/// `had_hits` is true if any source pattern matched in the scanned repo.
pub fn record(stats: &mut PatternStats, problem_id: &str, had_hits: bool) {
    let entry = stats.problems.entry(problem_id.to_string()).or_default();

    if had_hits {
        entry.runs_with_hits += 1;
    } else {
        entry.runs_with_misses += 1;
    }
}

/// Print a quality report to stderr showing hit rates for all tracked patterns.
pub fn print_report(stats: &PatternStats) {
    if stats.problems.is_empty() {
        eprintln!("No pattern quality data collected yet.");
        return;
    }

    let mut entries: Vec<_> = stats.problems.iter().collect();
    entries.sort_by(|a, b| a.1.hit_rate().partial_cmp(&b.1.hit_rate()).unwrap());

    eprintln!();
    eprintln!("Pattern Quality Report");
    eprintln!("{:-<68}", "");
    eprintln!(
        "{:<40} {:>6} {:>6} {:>8}",
        "Problem ID", "Hits", "Misses", "Hit %"
    );
    eprintln!("{:-<68}", "");

    for (id, s) in &entries {
        let display_id = if id.len() > 38 {
            format!("{}…", &id[..37])
        } else {
            id.to_string()
        };
        eprintln!(
            "{:<40} {:>6} {:>6} {:>7.1}%",
            display_id,
            s.runs_with_hits,
            s.runs_with_misses,
            s.hit_rate(),
        );
    }

    let low_quality: Vec<_> = entries
        .iter()
        .filter(|(_, s)| s.total_runs() >= 3 && s.hit_rate() < 10.0)
        .collect();

    if !low_quality.is_empty() {
        eprintln!();
        eprintln!(
            "Low quality ({} patterns with <10% hit rate after 3+ runs):",
            low_quality.len(),
        );
        for (id, _) in &low_quality {
            eprintln!("  → {id}");
        }
        eprintln!("Consider regenerating with: dep-doctor scan --generate-patterns");
    }

    log_debug!("Pattern stats: {} problems tracked", stats.problems.len(),);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_tracks_hits_and_misses() {
        let mut stats = PatternStats::default();
        record(&mut stats, "CVE-2024-001", true);
        record(&mut stats, "CVE-2024-001", false);
        record(&mut stats, "CVE-2024-001", true);

        let s = &stats.problems["CVE-2024-001"];
        assert_eq!(s.runs_with_hits, 2);
        assert_eq!(s.runs_with_misses, 1);
        assert_eq!(s.total_runs(), 3);
    }

    #[test]
    fn hit_rate_calculates_correctly() {
        let s = ProblemPatternStats {
            runs_with_hits: 3,
            runs_with_misses: 7,
        };
        assert!((s.hit_rate() - 30.0).abs() < 0.01);
    }

    #[test]
    fn hit_rate_zero_when_no_runs() {
        let s = ProblemPatternStats::default();
        assert_eq!(s.hit_rate(), 0.0);
    }

    #[test]
    fn stats_roundtrip_serialization() {
        let mut stats = PatternStats::default();
        record(&mut stats, "CVE-2024-001", true);
        record(&mut stats, "CVE-2024-002", false);

        let json = serde_json::to_string(&stats).unwrap();
        let loaded: PatternStats = serde_json::from_str(&json).unwrap();

        assert_eq!(loaded.problems.len(), 2);
        assert_eq!(loaded.problems["CVE-2024-001"].runs_with_hits, 1);
        assert_eq!(loaded.problems["CVE-2024-002"].runs_with_misses, 1);
    }
}
