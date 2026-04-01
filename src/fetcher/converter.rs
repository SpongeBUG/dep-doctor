use crate::fetcher::osv::Advisory;
use crate::problems::schema::Problem;
use crate::utils::semver_utils::space_to_comma_and;

/// Convert a list of OSV advisories into dep-doctor `Problem` structs.
pub fn to_problems(advisories: &[Advisory], ecosystem: &str, package: &str) -> Vec<Problem> {
    advisories
        .iter()
        .filter_map(|adv| convert_one(adv, ecosystem, package))
        .collect()
}

fn convert_one(adv: &Advisory, ecosystem: &str, package: &str) -> Option<Problem> {
    let affected_range = extract_range(adv)?;
    let severity = cvss_to_severity(extract_cvss(adv));

    let references = adv
        .references
        .iter()
        .filter_map(|r| r.url.clone())
        .collect();

    Some(Problem {
        id: adv.id.clone(),
        title: adv.summary.clone().unwrap_or_else(|| adv.id.clone()),
        severity,
        ecosystem: normalize_ecosystem(ecosystem).to_string(),
        package: package.to_string(),
        affected_range,
        fixed_in: extract_fixed_version(adv),
        references,
        source_patterns: None,
    })
}

/// Extract a semver range string from the advisory's affected data.
/// Prefers SEMVER-typed ranges; falls back to an exact version list.
fn extract_range(adv: &Advisory) -> Option<String> {
    for affected in &adv.affected {
        let semver_ranges: Vec<String> = affected
            .ranges
            .iter()
            .filter(|r| r.range_type == "SEMVER")
            .filter_map(range_to_string)
            .collect();

        if !semver_ranges.is_empty() {
            return Some(semver_ranges.join(","));
        }

        // Fallback: exact version list → "=X, =Y, =Z"
        if !affected.versions.is_empty() {
            let exact = affected
                .versions
                .iter()
                .map(|v| format!("={v}"))
                .collect::<Vec<_>>()
                .join(",");
            return Some(exact);
        }
    }

    None // no usable range data → skip this advisory
}

/// Convert a single SEMVER range (introduced..fixed) to a range string.
fn range_to_string(range: &crate::fetcher::osv::Range) -> Option<String> {
    let mut introduced: Option<&str> = None;
    let mut fixed: Option<&str> = None;

    for event in &range.events {
        if let Some(v) = &event.introduced {
            introduced = Some(v.as_str());
        }
        if let Some(v) = &event.fixed {
            fixed = Some(v.as_str());
        }
    }

    match (introduced, fixed) {
        (Some(i), Some(f)) => Some(space_to_comma_and(&format!(">={i} <{f}"))),
        (Some(i), None) => Some(format!(">={i}")),
        _ => None,
    }
}

/// Find the earliest fixed version from the advisory.
fn extract_fixed_version(adv: &Advisory) -> Option<String> {
    for affected in &adv.affected {
        for range in &affected.ranges {
            for event in &range.events {
                if let Some(v) = &event.fixed {
                    return Some(v.clone());
                }
            }
        }
    }
    None
}

/// Parse the first CVSS score from severity data.
fn extract_cvss(adv: &Advisory) -> Option<f64> {
    adv.severity
        .first()
        .and_then(|s| s.score.as_ref())
        .and_then(|score| parse_cvss_score(score))
}

/// Extract the numeric base score from a CVSS vector or plain number.
fn parse_cvss_score(raw: &str) -> Option<f64> {
    // Plain number: "7.5"
    if let Ok(n) = raw.parse::<f64>() {
        return Some(n);
    }
    // CVSS vector: "CVSS:3.1/AV:N/.../S:7.5" — score is usually not
    // embedded this way, but handle any trailing float after '/'.
    raw.rsplit('/').find_map(|part| part.parse::<f64>().ok())
}

fn cvss_to_severity(score: Option<f64>) -> String {
    match score {
        Some(s) if s >= 9.0 => "critical",
        Some(s) if s >= 7.0 => "high",
        Some(s) if s >= 4.0 => "medium",
        Some(s) if s > 0.0 => "low",
        _ => "medium", // missing score → safe default
    }
    .to_string()
}

/// Map OSV ecosystem names to dep-doctor ecosystem names.
fn normalize_ecosystem(eco: &str) -> &str {
    match eco {
        "npm" => "npm",
        "PyPI" => "pip",
        "Go" => "go",
        "crates.io" => "cargo",
        other => other,
    }
}
