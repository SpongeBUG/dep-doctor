use anyhow::Result;
use std::path::Path;

/// Fix a package version in requirements.txt or pyproject.toml.
/// Returns `Ok(true)` if a change was made.
pub fn fix_version(repo: &Path, package_name: &str, new_version: &str) -> Result<bool> {
    let mut changed = false;

    if fix_requirements_txt(repo, package_name, new_version)? {
        changed = true;
    }
    if fix_pyproject_toml(repo, package_name, new_version)? {
        changed = true;
    }

    Ok(changed)
}

/// Replace version in requirements.txt for the given package.
fn fix_requirements_txt(repo: &Path, package_name: &str, new_version: &str) -> Result<bool> {
    let path = repo.join("requirements.txt");
    if !path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&path)?;
    let normalized = normalize_pip_name(package_name);
    let mut changed = false;
    let mut lines: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            lines.push(line.to_string());
            continue;
        }

        if let Some((name, op, _old_ver, extras, rest)) = parse_req_line(trimmed) {
            if normalize_pip_name(&name) == normalized {
                let new_line = format!("{}{}{}{}{}", name, extras, op, new_version, rest);
                lines.push(new_line);
                changed = true;
                continue;
            }
        }

        lines.push(line.to_string());
    }

    if changed {
        let mut out = lines.join("\n");
        if content.ends_with('\n') {
            out.push('\n');
        }
        std::fs::write(&path, out)?;
    }

    Ok(changed)
}

/// Parse a requirements.txt line into (name, operator, version, extras, trailing).
fn parse_req_line(line: &str) -> Option<(String, String, String, String, String)> {
    let base = line.split('#').next().unwrap_or("").trim();

    // Extract extras like [security]
    let (name_part, extras, rest_after_extras) = if let Some(bracket_start) = base.find('[') {
        if let Some(bracket_end) = base[bracket_start..].find(']') {
            let end = bracket_start + bracket_end + 1;
            (
                &base[..bracket_start],
                base[bracket_start..end].to_string(),
                &base[end..],
            )
        } else {
            (base, String::new(), "")
        }
    } else {
        (base, String::new(), "")
    };

    let search = if rest_after_extras.is_empty() {
        name_part
    } else {
        rest_after_extras
    };

    for op in &["==", "~=", ">=", "<=", "!=", ">", "<"] {
        if let Some(idx) = search.find(op) {
            let name = if rest_after_extras.is_empty() {
                search[..idx].trim().to_string()
            } else {
                name_part.trim().to_string()
            };
            let after_op = &search[idx + op.len()..];
            let version = after_op.split(',').next().unwrap_or("").trim().to_string();
            let trailing_start = after_op.find(',').map(|i| &after_op[i..]).unwrap_or("");
            if !name.is_empty() && !version.is_empty() {
                return Some((
                    name,
                    op.to_string(),
                    version,
                    extras,
                    trailing_start.to_string(),
                ));
            }
        }
    }

    None
}

/// Replace version in pyproject.toml [project.dependencies] for the given package.
fn fix_pyproject_toml(repo: &Path, package_name: &str, new_version: &str) -> Result<bool> {
    let path = repo.join("pyproject.toml");
    if !path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&path)?;
    let normalized = normalize_pip_name(package_name);
    let mut result = String::with_capacity(content.len());
    let mut changed = false;

    for line in content.lines() {
        let trimmed = line.trim().trim_matches('"').trim_matches('\'');
        if let Some((name, _op, _ver, _extras, _rest)) = parse_req_line(trimmed) {
            if normalize_pip_name(&name) == normalized {
                // Replace the version in the quoted dependency string
                let new_dep = trimmed.replacen(&_ver, new_version, 1);
                // Preserve indentation and quoting
                let indent = &line[..line.len() - line.trim_start().len()];
                let quote = if line.trim_start().starts_with('"') {
                    "\""
                } else if line.trim_start().starts_with('\'') {
                    "'"
                } else {
                    ""
                };
                let trimmed_line = line.trim_end();
                let has_comma = trimmed_line.ends_with(',')
                    || trimmed_line.ends_with("\",")
                    || trimmed_line.ends_with("',");
                let comma = if has_comma { "," } else { "" };
                result.push_str(&format!("{indent}{quote}{new_dep}{quote}{comma}\n"));
                changed = true;
                continue;
            }
        }
        result.push_str(line);
        result.push('\n');
    }

    // Remove trailing newline if original didn't have one
    if !content.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }

    if changed {
        std::fs::write(&path, result)?;
    }

    Ok(changed)
}

fn normalize_pip_name(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn fixes_requirements_txt_exact_pin() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("requirements.txt"),
            "requests==2.28.0\nflask==2.3.0\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "requests", "2.31.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("requirements.txt")).unwrap();
        assert!(result.contains("requests==2.31.0"));
        assert!(result.contains("flask==2.3.0")); // unchanged
    }

    #[test]
    fn fixes_requirements_txt_compatible_release() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "django~=4.2.0\n").unwrap();

        let changed = fix_version(dir.path(), "django", "4.2.11").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("requirements.txt")).unwrap();
        assert!(result.contains("django~=4.2.11"));
    }

    #[test]
    fn handles_hyphen_underscore_normalization() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "my-package==1.0.0\n").unwrap();

        // Query with underscore should match hyphen in file
        let changed = fix_version(dir.path(), "my_package", "1.1.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("requirements.txt")).unwrap();
        assert!(result.contains("my-package==1.1.0"));
    }

    #[test]
    fn returns_false_when_package_not_found() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("requirements.txt"), "flask==2.3.0\n").unwrap();

        let changed = fix_version(dir.path(), "requests", "2.31.0").unwrap();
        assert!(!changed);
    }

    #[test]
    fn returns_false_when_no_manifests() {
        let dir = TempDir::new().unwrap();
        let changed = fix_version(dir.path(), "requests", "2.31.0").unwrap();
        assert!(!changed);
    }
}
