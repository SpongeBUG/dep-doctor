use anyhow::Result;
use std::path::{Path, PathBuf};

/// A discovered repo within the root scan directory.
#[derive(Debug)]
pub struct Repo {
    pub name: String,
    pub path: PathBuf,
}

/// Finds all repos (directories containing at least one manifest file)
/// directly inside `root`. Does NOT recurse deeper than one level
/// to avoid accidentally scanning vendor or node_modules trees.
pub fn find_repos(root: &Path) -> Result<Vec<Repo>> {
    let manifest_files = [
        "package.json",
        "requirements.txt",
        "pyproject.toml",
        "go.mod",
        "Cargo.toml",
    ];

    let mut repos = Vec::new();

    // Check if the root itself is a repo
    if is_repo(root, &manifest_files) {
        repos.push(Repo {
            name: dir_name(root),
            path: root.to_path_buf(),
        });
        return Ok(repos);
    }

    // Otherwise each subdirectory is a candidate repo
    for entry in std::fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }
        if is_hidden_or_ignored(&path) {
            continue;
        }
        if is_repo(&path, &manifest_files) {
            repos.push(Repo {
                name: dir_name(&path),
                path,
            });
        }
    }

    repos.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(repos)
}

fn is_repo(path: &Path, manifest_files: &[&str]) -> bool {
    manifest_files.iter().any(|f| path.join(f).exists())
}

fn is_hidden_or_ignored(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.') || n == "node_modules" || n == "vendor" || n == "target")
        .unwrap_or(false)
}

fn dir_name(path: &Path) -> String {
    path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}
