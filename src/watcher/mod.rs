use anyhow::Result;
use notify::RecursiveMode;
use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use std::path::Path;
use std::sync::mpsc;
use std::time::Duration;

use crate::log_debug;

/// Manifest filenames we care about.
const MANIFEST_FILES: &[&str] = &[
    "package.json",
    "package-lock.json",
    "requirements.txt",
    "pyproject.toml",
    "go.mod",
    "go.sum",
    "Cargo.toml",
    "Cargo.lock",
];

/// Enter the watch loop: wait for manifest changes, then call `on_change`.
/// Blocks until Ctrl+C (or an unrecoverable watcher error).
pub fn watch_loop<F>(root: &Path, mut on_change: F) -> Result<()>
where
    F: FnMut(),
{
    let (tx, rx) = mpsc::channel();
    let debounce_duration = Duration::from_millis(500);

    let mut debouncer = new_debouncer(debounce_duration, tx)?;
    debouncer.watcher().watch(root, RecursiveMode::Recursive)?;

    println!(
        "\x1b[1;36m\u{25b6} Watching for manifest changes in {}\x1b[0m (Ctrl+C to stop)",
        root.display()
    );

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                let dominated = events
                    .iter()
                    .any(|e| e.kind == DebouncedEventKind::Any && is_manifest_path(&e.path));
                if dominated {
                    let changed: Vec<_> = events
                        .iter()
                        .filter(|e| is_manifest_path(&e.path))
                        .map(|e| e.path.display().to_string())
                        .collect();
                    log_debug!("Manifest change detected: {:?}", changed);
                    println!("\n\x1b[1;36m\u{21bb} Change detected, re-scanning\u{2026}\x1b[0m\n");
                    on_change();
                }
            }
            Ok(Err(error)) => {
                eprintln!("Watch error: {error}");
            }
            Err(_) => {
                // Channel closed — watcher dropped, exit cleanly.
                break;
            }
        }
    }

    Ok(())
}

/// Check whether a path points to a manifest file we monitor.
fn is_manifest_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|name| MANIFEST_FILES.contains(&name))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_manifest_files() {
        assert!(is_manifest_path(Path::new("/foo/bar/package.json")));
        assert!(is_manifest_path(Path::new("Cargo.toml")));
        assert!(is_manifest_path(Path::new("/a/go.mod")));
        assert!(is_manifest_path(Path::new(
            "/home/user/project/requirements.txt",
        )));
    }

    #[test]
    fn rejects_non_manifest_files() {
        assert!(!is_manifest_path(Path::new("/foo/bar/main.rs")));
        assert!(!is_manifest_path(Path::new("README.md")));
        assert!(!is_manifest_path(Path::new("/a/b/index.js")));
    }
}
