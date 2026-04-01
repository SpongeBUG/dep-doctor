use anyhow::Result;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

/// Walk source files in a repo, respecting .gitignore.
/// Only returns files with the specified extensions.
pub fn walk_source_files(root: &Path, extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let walker = WalkBuilder::new(root)
        .hidden(true) // respect hidden files
        .git_ignore(true) // respect .gitignore
        .git_global(true) // respect global gitignore
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !matches!(
                name.as_ref(),
                "node_modules" | "vendor" | "target" | ".git" | "dist" | "build" | ".next"
            )
        })
        .build();

    for entry in walker {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if extensions.contains(&ext) {
                files.push(path.to_path_buf());
            }
        }
    }

    Ok(files)
}
