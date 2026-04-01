use std::path::{Path, PathBuf};

/// Returns all files matching a set of extensions under a path.
/// Does NOT respect .gitignore — use deep_scan::file_walker for that.
#[allow(dead_code)]
pub fn find_files_by_ext(root: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut results = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.extend(find_files_by_ext(&path, extensions));
            } else if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if extensions.contains(&ext) {
                    results.push(path);
                }
            }
        }
    }
    results
}
