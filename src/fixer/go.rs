use anyhow::Result;
use std::path::Path;

/// Fix a module version in go.mod.
/// Returns `Ok(true)` if a change was made.
pub fn fix_version(repo: &Path, module_name: &str, new_version: &str) -> Result<bool> {
    let path = repo.join("go.mod");
    if !path.exists() {
        return Ok(false);
    }

    let content = std::fs::read_to_string(&path)?;
    let (updated, changed) = replace_go_mod_version(&content, module_name, new_version);

    if changed {
        std::fs::write(&path, updated)?;
    }
    Ok(changed)
}

/// Text-based replacement of a module version in go.mod.
/// Handles both single-line `require foo v1.0.0` and block `require (...)` forms.
fn replace_go_mod_version(content: &str, module_name: &str, new_version: &str) -> (String, bool) {
    let new_ver = if new_version.starts_with('v') {
        new_version.to_string()
    } else {
        format!("v{new_version}")
    };

    let mut result = String::with_capacity(content.len());
    let mut changed = false;

    for line in content.lines() {
        let trimmed = line.trim();
        let without_comment = trimmed.split("//").next().unwrap_or("").trim();

        // Extract the dependency portion: either the whole line (inside a block)
        // or everything after `require ` (single-line form).
        let dep_part = if let Some(rest) = without_comment.strip_prefix("require ") {
            rest.trim()
        } else {
            without_comment
        };

        // Match: `module_name vX.Y.Z` (possibly with // indirect suffix)
        if let Some(after_name) = dep_part.strip_prefix(module_name) {
            let after_name = after_name.trim_start();
            if after_name.starts_with('v') {
                let indent = &line[..line.len() - line.trim_start().len()];
                // Preserve any comment from original line (e.g. // indirect)
                let comment = trimmed
                    .find("//")
                    .map(|idx| trimmed[idx..].to_string())
                    .unwrap_or_default();
                // Preserve `require ` prefix for single-line form
                let prefix = if without_comment.starts_with("require ") {
                    "require "
                } else {
                    ""
                };
                if comment.is_empty() {
                    result.push_str(&format!("{indent}{prefix}{module_name} {new_ver}\n"));
                } else {
                    result.push_str(&format!(
                        "{indent}{prefix}{module_name} {new_ver} {comment}\n"
                    ));
                }
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

    (result, changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn fixes_require_block_version() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("go.mod"),
            "module example.com/app\n\ngo 1.21\n\nrequire (\n\tgolang.org/x/net v0.10.0\n\tgolang.org/x/text v0.9.0\n)\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "golang.org/x/net", "0.17.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("go.mod")).unwrap();
        assert!(result.contains("golang.org/x/net v0.17.0"));
        assert!(result.contains("golang.org/x/text v0.9.0")); // unchanged
    }

    #[test]
    fn fixes_single_line_require() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("go.mod"),
            "module example.com/app\n\ngo 1.21\n\nrequire golang.org/x/net v0.10.0\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "golang.org/x/net", "0.17.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("go.mod")).unwrap();
        assert!(result.contains("golang.org/x/net v0.17.0"));
    }

    #[test]
    fn preserves_indirect_comment() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("go.mod"),
            "module example.com/app\n\nrequire (\n\tgolang.org/x/net v0.10.0 // indirect\n)\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "golang.org/x/net", "0.17.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("go.mod")).unwrap();
        assert!(result.contains("golang.org/x/net v0.17.0 // indirect"));
    }

    #[test]
    fn returns_false_when_module_not_found() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("go.mod"),
            "module example.com/app\n\ngo 1.21\n\nrequire golang.org/x/text v0.9.0\n",
        )
        .unwrap();

        let changed = fix_version(dir.path(), "golang.org/x/net", "0.17.0").unwrap();
        assert!(!changed);
    }

    #[test]
    fn returns_false_when_no_go_mod() {
        let dir = TempDir::new().unwrap();
        let changed = fix_version(dir.path(), "golang.org/x/net", "0.17.0").unwrap();
        assert!(!changed);
    }

    #[test]
    fn adds_v_prefix_when_missing() {
        let dir = TempDir::new().unwrap();
        fs::write(
            dir.path().join("go.mod"),
            "module example.com/app\n\nrequire golang.org/x/net v0.10.0\n",
        )
        .unwrap();

        // new_version without v prefix — should still produce v-prefixed output
        let changed = fix_version(dir.path(), "golang.org/x/net", "0.17.0").unwrap();
        assert!(changed);

        let result = fs::read_to_string(dir.path().join("go.mod")).unwrap();
        assert!(result.contains("v0.17.0"));
    }
}
