pub mod cargo;
pub mod go;
pub mod npm;
pub mod pip;

use crate::problems::schema::Finding;
use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

/// Outcome of a single fix attempt.
#[derive(Debug)]
pub struct FixResult {
    pub problem_id: String,
    pub package: String,
    pub ecosystem: String,
    pub old_version: String,
    pub new_version: String,
    pub applied: bool,
    pub reason: Option<String>,
}

/// Apply fixes for all findings that have a `fixed_in` version.
/// Groups by (repo_path, ecosystem) so each manifest is edited per-package.
/// Returns a list of results describing what was and wasn't fixed.
pub fn apply_fixes(findings: &[Finding<'_>]) -> Vec<FixResult> {
    let mut groups: HashMap<(String, String), Vec<FixCandidate>> = HashMap::new();
    let mut results = Vec::new();

    for f in findings {
        let fixed_in = match &f.problem.fixed_in {
            Some(v) if !v.is_empty() => v.clone(),
            _ => {
                results.push(FixResult {
                    problem_id: f.problem.id.clone(),
                    package: f.package.clone(),
                    ecosystem: f.problem.ecosystem.clone(),
                    old_version: f.installed_version.clone(),
                    new_version: String::new(),
                    applied: false,
                    reason: Some("no fixed version known".into()),
                });
                continue;
            }
        };

        groups
            .entry((f.repo_path.clone(), f.problem.ecosystem.to_lowercase()))
            .or_default()
            .push(FixCandidate {
                problem_id: f.problem.id.clone(),
                package: f.package.clone(),
                old_version: f.installed_version.clone(),
                new_version: fixed_in,
            });
    }

    for ((repo_path, ecosystem), candidates) in &groups {
        let repo = Path::new(repo_path);
        for candidate in candidates {
            results.push(apply_single_fix(repo, ecosystem, candidate));
        }
    }

    results
}

/// Route a single fix to the ecosystem-specific fixer.
fn apply_single_fix(repo: &Path, ecosystem: &str, c: &FixCandidate) -> FixResult {
    let result: Result<bool> = match ecosystem {
        "npm" => npm::fix_version(repo, &c.package, &c.new_version),
        "pip" => pip::fix_version(repo, &c.package, &c.new_version),
        "go" => go::fix_version(repo, &c.package, &c.new_version),
        "cargo" => cargo::fix_version(repo, &c.package, &c.new_version),
        other => {
            return FixResult {
                problem_id: c.problem_id.clone(),
                package: c.package.clone(),
                ecosystem: other.to_string(),
                old_version: c.old_version.clone(),
                new_version: c.new_version.clone(),
                applied: false,
                reason: Some(format!("unsupported ecosystem: {other}")),
            };
        }
    };

    match result {
        Ok(true) => FixResult {
            problem_id: c.problem_id.clone(),
            package: c.package.clone(),
            ecosystem: ecosystem.to_string(),
            old_version: c.old_version.clone(),
            new_version: c.new_version.clone(),
            applied: true,
            reason: None,
        },
        Ok(false) => FixResult {
            problem_id: c.problem_id.clone(),
            package: c.package.clone(),
            ecosystem: ecosystem.to_string(),
            old_version: c.old_version.clone(),
            new_version: c.new_version.clone(),
            applied: false,
            reason: Some("package not found in manifest".into()),
        },
        Err(e) => FixResult {
            problem_id: c.problem_id.clone(),
            package: c.package.clone(),
            ecosystem: ecosystem.to_string(),
            old_version: c.old_version.clone(),
            new_version: c.new_version.clone(),
            applied: false,
            reason: Some(format!("error: {e}")),
        },
    }
}

struct FixCandidate {
    problem_id: String,
    package: String,
    old_version: String,
    new_version: String,
}

/// Print a human-readable fix summary to stdout.
pub fn print_summary(results: &[FixResult]) {
    if results.is_empty() {
        return;
    }

    let applied: Vec<_> = results.iter().filter(|r| r.applied).collect();
    let skipped: Vec<_> = results.iter().filter(|r| !r.applied).collect();

    println!();
    if !applied.is_empty() {
        println!(
            "\x1b[1;32m\u{2713} Fixed {} finding(s):\x1b[0m",
            applied.len()
        );
        for r in &applied {
            println!(
                "  {} ({}) \u{2192} {}",
                r.package, r.old_version, r.new_version
            );
        }
    }

    if !skipped.is_empty() {
        println!(
            "\x1b[1;33m\u{26a0} Skipped {} finding(s):\x1b[0m",
            skipped.len()
        );
        for r in &skipped {
            let reason = r.reason.as_deref().unwrap_or("unknown");
            println!("  {} \u{2014} {}", r.package, reason);
        }
    }

    println!(
        "\n\x1b[1mFixed {}/{} findings.\x1b[0m \
         Run your package manager's install/update command to apply lock file changes.",
        applied.len(),
        results.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problems::schema::{Problem, ProblemKind};

    fn make_finding<'a>(problem: &'a Problem, version: &str, repo_path: &str) -> Finding<'a> {
        Finding {
            repo_name: "test-repo".into(),
            repo_path: repo_path.into(),
            package: problem.package.clone(),
            installed_version: version.into(),
            problem,
            source_hits: vec![],
        }
    }

    #[test]
    fn skips_findings_without_fixed_in() {
        let problem = Problem {
            id: "TEST-001".into(),
            title: "Test".into(),
            severity: "high".into(),
            ecosystem: "npm".into(),
            package: "foo".into(),
            affected_range: ">=1.0.0 <2.0.0".into(),
            fixed_in: None,
            references: vec![],
            source_patterns: None,
            kind: ProblemKind::Cve,
        };
        let findings = vec![make_finding(&problem, "1.5.0", "/tmp/fake")];
        let results = apply_fixes(&findings);

        assert_eq!(results.len(), 1);
        assert!(!results[0].applied);
        assert_eq!(results[0].reason.as_deref(), Some("no fixed version known"));
    }

    #[test]
    fn skips_findings_with_empty_fixed_in() {
        let problem = Problem {
            id: "TEST-002".into(),
            title: "Test".into(),
            severity: "high".into(),
            ecosystem: "npm".into(),
            package: "bar".into(),
            affected_range: ">=1.0.0 <2.0.0".into(),
            fixed_in: Some(String::new()),
            references: vec![],
            source_patterns: None,
            kind: ProblemKind::Cve,
        };
        let findings = vec![make_finding(&problem, "1.5.0", "/tmp/fake")];
        let results = apply_fixes(&findings);

        assert_eq!(results.len(), 1);
        assert!(!results[0].applied);
    }
}
