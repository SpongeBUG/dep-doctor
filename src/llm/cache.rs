//! Disk cache for LLM-generated source patterns.
//!
//! Patterns are cached forever (no TTL) because CVE descriptions don't change.
//! Location: `~/.cache/dep-doctor/patterns/<sanitized-id>.json`

use std::fs;
use std::path::PathBuf;

use crate::log_warn;
use crate::problems::schema::SourcePatternSet;

/// Base directory: `~/.cache/dep-doctor/patterns/` (or OS equivalent).
fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("dep-doctor").join("patterns"))
}

/// Sanitize a problem ID into a filename-safe string.
fn sanitize_id(id: &str) -> String {
    id.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Read a cached `SourcePatternSet` for the given problem ID, if it exists.
pub fn get(problem_id: &str) -> Option<SourcePatternSet> {
    let path = cache_dir()?.join(format!("{}.json", sanitize_id(problem_id)));
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Write a `SourcePatternSet` to the cache. Silently ignores write errors.
pub fn set(problem_id: &str, patterns: &SourcePatternSet) {
    let Some(dir) = cache_dir() else { return };

    if fs::create_dir_all(&dir).is_err() {
        log_warn!("Could not create pattern cache dir: {}", dir.display());
        return;
    }

    let path = dir.join(format!("{}.json", sanitize_id(problem_id)));
    let Ok(json) = serde_json::to_string_pretty(patterns) else {
        return;
    };

    if let Err(e) = fs::write(&path, json) {
        log_warn!("Could not write pattern cache {}: {e}", path.display());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::schema::{Confidence, SourcePattern, SourcePatternSet};

    #[test]
    fn sanitize_id_handles_special_chars() {
        assert_eq!(sanitize_id("CVE-2023-45857"), "CVE-2023-45857");
        assert_eq!(
            sanitize_id("npm-axios-csrf-ssrf-CVE-2023-45857"),
            "npm-axios-csrf-ssrf-CVE-2023-45857"
        );
        assert_eq!(sanitize_id("GHSA/2024:foo bar"), "GHSA_2024_foo_bar");
    }

    #[test]
    fn round_trip_through_cache() {
        // Skip if no cache dir available (CI containers, etc.).
        let Some(dir) = cache_dir() else { return };

        let test_id = "dep-doctor-test-round-trip";
        let patterns = SourcePatternSet {
            languages: vec!["js".into()],
            patterns: vec![SourcePattern {
                description: "test pattern".into(),
                regex: r"require\(.*axios".into(),
                confidence: Confidence::Possible,
                remediation: "upgrade".into(),
            }],
        };

        set(test_id, &patterns);

        let cached = get(test_id).expect("should read back cached patterns");
        assert_eq!(cached.languages, vec!["js"]);
        assert_eq!(cached.patterns.len(), 1);
        assert_eq!(cached.patterns[0].description, "test pattern");

        // Cleanup.
        let _ = fs::remove_file(dir.join(format!("{test_id}.json")));
    }
}
