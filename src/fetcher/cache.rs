use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::fetcher::osv::Advisory;
use crate::log_warn;

const TTL: Duration = Duration::from_secs(3600); // 1 hour

/// Base directory: ~/.cache/dep-doctor/osv (or OS equivalent).
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("dep-doctor").join("osv"))
}

/// Build a filename-safe cache key.
pub fn cache_key(ecosystem: &str, name: &str, version: &str) -> String {
    format!(
        "{}-{}-{}",
        sanitize(ecosystem),
        sanitize(name),
        sanitize(version),
    )
}

/// Read cached advisories if the cache file exists and is fresh.
pub fn get(key: &str) -> Option<Vec<Advisory>> {
    let path = cache_dir()?.join(format!("{key}.json"));

    let meta = fs::metadata(&path).ok()?;
    let modified = meta.modified().ok()?;
    if SystemTime::now().duration_since(modified).unwrap_or(TTL) >= TTL {
        return None; // expired
    }

    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Write advisories to the cache. Silently ignores write errors.
pub fn set(key: &str, advisories: &[Advisory]) {
    let Some(dir) = cache_dir() else { return };

    if fs::create_dir_all(&dir).is_err() {
        log_warn!("Could not create cache directory: {}", dir.display());
        return;
    }

    let path = dir.join(format!("{key}.json"));
    let Ok(json) = serde_json::to_string(advisories) else {
        return;
    };

    if let Err(e) = fs::write(&path, json) {
        log_warn!("Could not write cache file {}: {e}", path.display());
    }
}

/// Replace anything that isn't alphanumeric, dash, or dot with an underscore.
fn sanitize(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_sanitizes() {
        assert_eq!(cache_key("npm", "lodash", "4.17.21"), "npm-lodash-4.17.21");
        assert_eq!(
            cache_key("PyPI", "@scope/pkg", "1.0"),
            "pypi-_scope_pkg-1.0"
        );
    }
}
