//! Typosquatting detector — compares scanned package names against the
//! curated popular-package list and flags anything within Levenshtein
//! edit distance ≤ 2 that isn't itself a popular name.

use serde::Serialize;
use std::collections::HashSet;

use crate::harvest::packages::popular_names;
use crate::scanner::manifest::InstalledPackage;

/// A warning that a scanned package name looks suspiciously similar
/// to a well-known package (possible typosquat).
#[derive(Debug, Clone, Serialize)]
pub struct TyposquatWarning {
    pub scanned_name: String,
    pub ecosystem: String,
    pub similar_to: String,
    pub edit_distance: usize,
}

/// Maximum edit distance to consider a potential typosquat.
const MAX_DISTANCE: usize = 2;

/// Check all scanned packages against the popular-name list for their
/// ecosystem. Returns warnings for packages within edit distance ≤ 2
/// of a popular name, excluding the popular names themselves.
pub fn check(packages: &[InstalledPackage]) -> Vec<TyposquatWarning> {
    let ecosystems = ["npm", "pip", "go", "cargo"];
    let mut warnings = Vec::new();
    let mut seen = HashSet::new();

    for eco in &ecosystems {
        let popular = popular_names(eco);
        let popular_set: HashSet<&str> = popular.iter().copied().collect();

        let eco_pkgs: Vec<&InstalledPackage> =
            packages.iter().filter(|p| p.ecosystem == *eco).collect();

        for pkg in &eco_pkgs {
            if popular_set.contains(pkg.name.as_str()) {
                continue;
            }

            let key = (pkg.name.clone(), eco.to_string());
            if seen.contains(&key) {
                continue;
            }

            if let Some(warning) = closest_match(&pkg.name, eco, &popular) {
                seen.insert(key);
                warnings.push(warning);
            }
        }
    }

    warnings
}

/// Find the closest popular name within MAX_DISTANCE.
fn closest_match(name: &str, ecosystem: &str, popular: &[&str]) -> Option<TyposquatWarning> {
    let mut best: Option<(usize, &str)> = None;

    for &pop in popular {
        let dist = levenshtein(name, pop);
        if dist == 0 || dist > MAX_DISTANCE {
            continue;
        }
        if best.is_none() || dist < best.unwrap().0 {
            best = Some((dist, pop));
        }
    }

    best.map(|(dist, pop)| TyposquatWarning {
        scanned_name: name.to_string(),
        ecosystem: ecosystem.to_string(),
        similar_to: pop.to_string(),
        edit_distance: dist,
    })
}

/// Classic Levenshtein distance (no external dep — strsim is transitive only).
fn levenshtein(a: &str, b: &str) -> usize {
    let a_len = a.len();
    let b_len = b.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut prev: Vec<usize> = (0..=b_len).collect();
    let mut curr = vec![0usize; b_len + 1];

    for (i, ca) in a.chars().enumerate() {
        curr[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            curr[j + 1] = (prev[j] + cost).min(prev[j + 1] + 1).min(curr[j] + 1);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levenshtein_identical() {
        assert_eq!(levenshtein("lodash", "lodash"), 0);
    }

    #[test]
    fn levenshtein_one_off() {
        assert_eq!(levenshtein("lodash", "lodas"), 1);
        assert_eq!(levenshtein("lodash", "l0dash"), 1);
    }

    #[test]
    fn levenshtein_two_off() {
        assert_eq!(levenshtein("lodash", "lodsh"), 1); // deletion
        assert_eq!(levenshtein("express", "exprss"), 1);
    }

    #[test]
    fn levenshtein_too_far() {
        assert!(levenshtein("lodash", "xyz") > MAX_DISTANCE);
    }

    fn make_pkg(name: &str, eco: &str) -> InstalledPackage {
        InstalledPackage {
            repo_name: "test-repo".to_string(),
            repo_path: "/tmp/test".to_string(),
            name: name.to_string(),
            version: "1.0.0".to_string(),
            ecosystem: eco.to_string(),
        }
    }

    #[test]
    fn check_flags_typosquat() {
        let packages = vec![make_pkg("lodasj", "npm")];
        let warnings = check(&packages);
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].similar_to, "lodash");
        assert!(warnings[0].edit_distance <= MAX_DISTANCE);
    }

    #[test]
    fn check_skips_exact_popular_name() {
        let packages = vec![make_pkg("lodash", "npm")];
        let warnings = check(&packages);
        assert!(warnings.is_empty());
    }
}
