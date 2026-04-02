use anyhow::Result;
use std::path::Path;

/// Fix a crate version in Cargo.toml.
/// Handles both `crate = "version"` and `crate = { version = "..." }` forms.
/// Returns `Ok(true)` if a change was made.
pub fn fix_version(repo: &Path, crate_name: &str, new_version: &str) -> Result<bool> {
    let path = repo.join("Cargo.toml");
    if !path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&path)?;
    let (updated, changed) = replace_cargo_version(&content, crate_name, new_version);

    if changed {
        std::fs::write(&path, updated)?;
    }
    Ok(changed)
}

/// Text-based replacement of a crate version in Cargo.toml.
/// Handles two forms:
///   `crate_name = "^1.0.0"`
///   `crate_name = { version = "^1.0.0", features = [...] }`
fn replace_cargo_version(content: &str, crate_name: &str, new_version: &str) -> (String, bool) {
    let mut result = String::with_capacity(content.len());
    let mut changed = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Match: `crate_name = "version"` or `crate_name = { version = "..." ... }`
        if let Some(after_name) = strip_toml_key(trimmed, crate_name) {
            let after_eq = after_name.trim();

            if after_eq.starts_with('"') {
                // Simple form: crate = "^1.0.0"
                if let Some(new_line) = replace_quoted_version(line, after_eq, new_version) {
                    result.push_str(&new_line);
                    result.push('\n');
                    changed = true;
                    continue;
                }
            } else if after_eq.starts_with('{') {
                // Table form: crate = { version = "^1.0.0", ... }
                if let Some(new_line) = replace_table_version(line, new_version) {
                    result.push_str(&new_line);
                    result.push('\n');
                    changed = true;
                    continue;
                }
            }
        }

        result.push_str(line);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    (result, changed)
}

/// Strip a TOML key from the beginning of a line, returning everything after `=`.
/// Returns None if the line doesn't start with the key.
fn strip_toml_key<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let trimmed = line.trim();
    if !trimmed.starts_with(key) {
        return None;
    }
    let rest = &trimmed[key.len()..];
    let rest = rest.trim_start();
    rest.strip_prefix('=')
}

/// Replace version in simple form: `crate = "^1.0.0"` → `crate = "^1.7.0"`
fn replace_quoted_version(line: &str, after_eq: &str, new_version: &str) -> Option<String> {
    // after_eq starts with '"'
    let inner = &after_eq[1..]; // skip opening quote
    let close = inner.find('"')?;
    let old_spec = &inner[..close];
    let prefix = extract_prefix(old_spec);
    let new_spec = format!("{prefix}{new_version}");

    // Replace the first occurrence of the old spec in the original line
    Some(line.replacen(old_spec, &new_spec, 1))
}

/// Replace version in table form: `crate = { version = "^1.0.0", ... }`
fn replace_table_version(line: &str, new_version: &str) -> Option<String> {
    // Find `version = "..."` within the line
    let ver_key = "version";
    let ver_idx = line.find(ver_key)?;
    let after_ver_key = &line[ver_idx + ver_key.len()..];
    let eq_offset = after_ver_key.find('=')?;
    let after_eq = &after_ver_key[eq_offset + 1..].trim_start();

    if !after_eq.starts_with('"') {
        return None;
    }
    let inner = &after_eq[1..];
    let close = inner.find('"')?;
    let old_spec = &inner[..close];
    let prefix = extract_prefix(old_spec);
    let new_spec = format!("{prefix}{new_version}");

    Some(line.replacen(old_spec, &new_spec, 1))
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

    #[test]
    fn fixes_simple_version() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nserde = \"1.0.180\"\nregex = \"1.9.0\"\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "serde", "1.0.197").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(result.contains("serde = \"1.0.197\""));
        assert!(result.contains("regex = \"1.9.0\"")); // unchanged
    }

    #[test]
    fn fixes_caret_version() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\ntokio = \"^1.28.0\"\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "tokio", "1.36.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(result.contains("tokio = \"^1.36.0\""));
    }

    #[test]
    fn fixes_table_form_version() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nclap = { version = \"4.4.0\", features = [\"derive\"] }\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "clap", "4.5.4").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(result.contains("version = \"4.5.4\""));
        assert!(result.contains("features = [\"derive\"]")); // preserved
    }

    #[test]
    fn returns_false_when_crate_not_found() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dependencies]\nserde = \"1.0.180\"\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "tokio", "1.36.0").unwrap();
        assert!(!changed);
    }

    #[test]
    fn returns_false_when_no_cargo_toml() {
        let dir = TempDir::new().unwrap();
        let changed = fix_version(dir.path(), "serde", "1.0.197").unwrap();
        assert!(!changed);
    }

    #[test]
    fn fixes_dev_dependencies() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("Cargo.toml"),
            "[dev-dependencies]\ntempfile = \"3.8.0\"\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "tempfile", "3.10.1").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("Cargo.toml")).unwrap();
        assert!(result.contains("tempfile = \"3.10.1\""));
    }
}
