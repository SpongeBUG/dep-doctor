pub mod cache;
pub mod converter;
pub mod osv;

use std::collections::HashSet;

use crate::problems::schema::Problem;
use crate::scanner::manifest::InstalledPackage;

/// Query OSV.dev for every unique (ecosystem, name, version) tuple.
/// Returns additional `Problem` structs to merge with built-in problems.
pub fn query_packages(packages: &[InstalledPackage]) -> Vec<Problem> {
    let unique = dedup_packages(packages);

    let mut all_problems = Vec::new();

    for (eco, name, version) in &unique {
        let key = cache::cache_key(eco, name, version);

        // Cache hit?
        if let Some(advisories) = cache::get(&key) {
            all_problems.extend(converter::to_problems(&advisories, eco, name));
            continue;
        }

        // Cache miss → query OSV for this single package
        let query = osv::Query {
            version: version.clone(),
            package: osv::QueryPackage {
                name: name.clone(),
                ecosystem: to_osv_ecosystem(eco).to_string(),
            },
        };

        let results = osv::query_batch(&[query]).unwrap_or_default();
        let advisories = results
            .into_iter()
            .flat_map(|r| r.vulns)
            .collect::<Vec<_>>();

        cache::set(&key, &advisories);
        all_problems.extend(converter::to_problems(&advisories, eco, name));
    }

    all_problems
}

/// Dedup packages by (ecosystem, name, version) so we don't query the same
/// package twice if it appears in multiple repos.
fn dedup_packages(packages: &[InstalledPackage]) -> Vec<(String, String, String)> {
    let mut seen = HashSet::new();
    let mut out = Vec::new();

    for pkg in packages {
        let key = (
            pkg.ecosystem.to_lowercase(),
            pkg.name.clone(),
            pkg.version.clone(),
        );
        if seen.insert(key.clone()) {
            out.push(key);
        }
    }

    out
}

/// Map dep-doctor ecosystem names → OSV ecosystem names.
fn to_osv_ecosystem(eco: &str) -> &str {
    match eco {
        "npm" => "npm",
        "pip" => "PyPI",
        "go" => "Go",
        "cargo" => "crates.io",
        other => other,
    }
}
