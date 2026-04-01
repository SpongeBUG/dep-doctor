use semver::{Version, VersionReq};

/// Returns true if `version_str` satisfies `range_str`.
///
/// Range format:
/// - AND range: ">=0.8.1 <1.6.0"  (space between comparators)
/// - OR ranges: ">=0.21.0 <0.21.11,>=0.22.0 <0.22.4"  (comma = OR)
pub fn version_matches_range(version_str: &str, range_str: &str) -> bool {
    let Ok(version) = Version::parse(&normalize_version(version_str)) else {
        return false;
    };

    for segment in range_str.split(',') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }

        let normalized = space_to_comma_and(segment);

        if let Ok(req) = VersionReq::parse(&normalized) {
            if req.matches(&version) {
                return true;
            }
        }
    }

    false
}

/// Converts space-separated comparators to comma-separated AND conditions.
/// ">=0.8.1 <1.6.0" → ">=0.8.1, <1.6.0"
fn space_to_comma_and(segment: &str) -> String {
    let comparator_chars = ['>', '<', '=', '~', '^'];
    let mut result = String::with_capacity(segment.len() + 4);

    for (i, token) in segment.split_whitespace().enumerate() {
        if i == 0 {
            result.push_str(token);
        } else if token.starts_with(comparator_chars) {
            result.push_str(", ");
            result.push_str(token);
        } else {
            result.push(' ');
            result.push_str(token);
        }
    }

    result
}

fn normalize_version(v: &str) -> String {
    let v = v.trim().trim_start_matches('v');
    let parts: Vec<&str> = v.split('.').collect();
    match parts.len() {
        1 => format!("{}.0.0", parts[0]),
        2 => format!("{}.{}.0", parts[0], parts[1]),
        _ => v.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_space_to_comma_and() {
        assert_eq!(space_to_comma_and(">=0.8.1 <1.6.0"), ">=0.8.1, <1.6.0");
        assert_eq!(space_to_comma_and(">=0.8.1"), ">=0.8.1");
        assert_eq!(space_to_comma_and("<4.17.21"), "<4.17.21");
        assert_eq!(
            space_to_comma_and(">=0.21.0 <0.21.11"),
            ">=0.21.0, <0.21.11"
        );
    }

    #[test]
    fn test_axios_range() {
        assert!(version_matches_range("0.27.2", ">=0.8.1 <1.6.0"));
        assert!(version_matches_range("1.5.9", ">=0.8.1 <1.6.0"));
        assert!(!version_matches_range("1.6.0", ">=0.8.1 <1.6.0"));
        assert!(!version_matches_range("1.7.0", ">=0.8.1 <1.6.0"));
    }

    #[test]
    fn test_multi_range() {
        let range = ">=0.21.0 <0.21.11,>=0.22.0 <0.22.4";
        assert!(version_matches_range("0.21.5", range));
        assert!(version_matches_range("0.22.3", range));
        assert!(!version_matches_range("0.21.11", range));
        assert!(!version_matches_range("0.23.0", range));
    }

    #[test]
    fn test_normalize_short_version() {
        assert!(version_matches_range("4", "<4.17.21"));
        assert!(version_matches_range("4.17", "<4.17.21"));
    }

    #[test]
    fn test_exact_boundary() {
        assert!(!version_matches_range("1.6.0", ">=0.8.1 <1.6.0"));
        assert!(!version_matches_range("4.17.21", "<4.17.21"));
    }
}
