use anyhow::Result;
use std::path::Path;

/// Fix a package version in package.json.
/// Replaces the version spec for `package_name` in both dependencies and devDependencies.
/// Returns `Ok(true)` if a change was made, `Ok(false)` if the package wasn't found.
pub fn fix_version(repo: &Path, package_name: &str, new_version: &str) -> Result<bool> {
    let pkg_path = repo.join("package.json");
    if !pkg_path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&pkg_path)?;
    let (updated, changed) = replace_json_dep_version(&content, package_name, new_version);

    if changed {
        std::fs::write(&pkg_path, updated)?;
    }
    Ok(changed)
}

/// Text-based replacement of a dependency version in package.json.
/// Preserves formatting by only replacing the version string value.
fn replace_json_dep_version(
    content: &str,
    package_name: &str,
    new_version: &str,
) -> (String, bool) {
    let needle = format!("\"{}\"", package_name);
    let mut result = String::with_capacity(content.len());
    let mut changed = false;
    let mut pos = 0;

    // Strategy: find `"<package_name>"` followed by `:` then a quoted version string.
    // Replace only the version string value.
    while pos < content.len() {
        if content[pos..].starts_with(&needle) {
            // Found the package name key. Copy it, then look for : "version".
            let key_end = pos + needle.len();
            result.push_str(&content[pos..key_end]);
            pos = key_end;

            // Skip whitespace and colon
            let rest = &content[pos..];
            if let Some(colon_offset) = rest.find(':') {
                let before_colon = &rest[..colon_offset];
                // Make sure there's only whitespace before the colon
                if before_colon.trim().is_empty() {
                    result.push_str(&rest[..=colon_offset]);
                    pos += colon_offset + 1;

                    // Skip whitespace after colon
                    let rest2 = &content[pos..];
                    let trimmed = rest2.trim_start();
                    let ws_len = rest2.len() - trimmed.len();
                    result.push_str(&rest2[..ws_len]);
                    pos += ws_len;

                    // Now expect a quoted version string
                    if content[pos..].starts_with('"') {
                        pos += 1; // skip opening quote
                        if let Some(close) = content[pos..].find('"') {
                            // Replace the version value, preserving ^ or ~ prefix
                            let old_ver = &content[pos..pos + close];
                            let prefix = extract_prefix(old_ver);
                            result.push('"');
                            result.push_str(prefix);
                            result.push_str(new_version);
                            result.push('"');
                            pos += close + 1; // skip past closing quote
                            changed = true;
                            continue;
                        }
                    }
                }
            }
            continue;
        }

        result.push(content.as_bytes()[pos] as char);
        pos += 1;
    }

    (result, changed)
}

/// Extract semver prefix (^, ~, >=, etc.) from a version spec.
fn extract_prefix(spec: &str) -> &str {
    let version_start = spec
        .find(|c: char| c.is_ascii_digit())
        .unwrap_or(spec.len());
    &spec[..version_start]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_package_json(dir: &Path, content: &str) {
        fs::write(dir.join("package.json"), content).unwrap();
    }

    #[test]
    fn fixes_dependency_version() {
        let dir = TempDir::new().unwrap();
        write_package_json(
            dir.path(),
            r#"{
  "dependencies": {
    "axios": "^1.5.0",
    "express": "^4.18.0"
  }
}"#,
        );

        let changed = fix_version(dir.path(), "axios", "1.7.4").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("package.json")).unwrap();
        assert!(result.contains("\"^1.7.4\""));
        assert!(result.contains("\"^4.18.0\"")); // express unchanged
    }

    #[test]
    fn fixes_dev_dependency_version() {
        let dir = TempDir::new().unwrap();
        write_package_json(
            dir.path(),
            r#"{
  "devDependencies": {
    "jest": "~29.0.0"
  }
}"#,
        );

        let changed = fix_version(dir.path(), "jest", "29.7.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("package.json")).unwrap();
        assert!(result.contains("\"~29.7.0\""));
    }

    #[test]
    fn preserves_exact_version_no_prefix() {
        let dir = TempDir::new().unwrap();
        write_package_json(
            dir.path(),
            r#"{
  "dependencies": {
    "lodash": "4.17.20"
  }
}"#,
        );

        let changed = fix_version(dir.path(), "lodash", "4.17.21").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("package.json")).unwrap();
        assert!(result.contains("\"4.17.21\""));
    }

    #[test]
    fn returns_false_when_package_not_found() {
        let dir = TempDir::new().unwrap();
        write_package_json(
            dir.path(),
            r#"{
  "dependencies": {
    "express": "^4.18.0"
  }
}"#,
        );

        let changed = fix_version(dir.path(), "axios", "1.7.4").unwrap();
        assert!(!changed);
    }

    #[test]
    fn returns_false_when_no_package_json() {
        let dir = TempDir::new().unwrap();
        let changed = fix_version(dir.path(), "axios", "1.7.4").unwrap();
        assert!(!changed);
    }

    #[test]
    fn extract_prefix_works() {
        assert_eq!(extract_prefix("^1.5.0"), "^");
        assert_eq!(extract_prefix("~2.0.0"), "~");
        assert_eq!(extract_prefix(">=1.0.0"), ">=");
        assert_eq!(extract_prefix("1.0.0"), "");
    }
}
