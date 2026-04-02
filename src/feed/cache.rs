
//! Feed disk cache: ~/.cache/dep-doctor/problems.feed.json with 24h TTL.
//! One responsibility: read/write the feed JSON to/from disk.

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::problems::schema::Problem;

const FEED_TTL: Duration = Duration::from_secs(60 * 60 * 24); // 24 hours
const FEED_FILENAME: &str = "problems.feed.json";

/// Directory: ~/.cache/dep-doctor/
pub fn cache_dir() -> Option<PathBuf> {
    dirs::cache_dir().map(|d| d.join("dep-doctor"))
}

/// Full path to the cached feed file.
pub fn feed_path() -> Option<PathBuf> {
    cache_dir().map(|d| d.join(FEED_FILENAME))
}

/// Returns true if the cached feed exists and is younger than 24h.
pub fn is_fresh() -> bool {
    let Some(path) = feed_path() else { return false };
    let Ok(meta) = fs::metadata(&path) else { return false };
    let Ok(modified) = meta.modified() else { return false };
    SystemTime::now()
        .duration_since(modified)
        .map(|age| age < FEED_TTL)
        .unwrap_or(false)
}

/// Load problems from the cache. Returns None on miss, expired, or parse error.
pub fn load() -> Option<Vec<Problem>> {
    let path = feed_path()?;
    let data = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&data).ok()
}

/// Persist problems to the cache directory. Silently ignores write errors.
pub fn save(problems: &[Problem]) {
    let Some(dir) = cache_dir() else { return };
    if fs::create_dir_all(&dir).is_err() {
        return;
    }
    let Some(path) = feed_path() else { return };
    if let Ok(json) = serde_json::to_string(problems) {
        let _ = fs::write(path, json);
    }
}
