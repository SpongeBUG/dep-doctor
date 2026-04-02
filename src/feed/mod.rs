//! Feed public API: load_feed() returns a Vec<Problem>.
//!
//! Resolution order:
//!   1. Disk cache (~/.cache/dep-doctor/problems.feed.json) if fresh (< 24h)
//!   2. GitHub Releases CDN fetch → save to cache
//!   3. Stale disk cache (graceful degradation)
//!   4. Local ./problems.feed.json (dev fallback — useful before first CI run)
//!   5. Empty vec (scan still works with built-in problems only)

pub mod cache;
pub mod fetcher;

use crate::problems::schema::Problem;

pub fn load_feed() -> Vec<Problem> {
    // 1. Fresh cache hit.
    if cache::is_fresh() {
        if let Some(problems) = cache::load() {
            return problems;
        }
    }

    // 2. Fetch from CDN and cache.
    match fetcher::fetch() {
        Ok(problems) => {
            cache::save(&problems);
            return problems;
        }
        Err(e) => {
            eprintln!(
                "[dep-doctor] Warning: feed fetch failed ({e}). Falling back to local cache."
            );
        }
    }

    // 3. Stale cache.
    if let Some(problems) = cache::load() {
        return problems;
    }

    // 4. Local dev fallback: ./problems.feed.json
    if let Ok(data) = std::fs::read_to_string("problems.feed.json") {
        if let Ok(problems) = serde_json::from_str::<Vec<Problem>>(&data) {
            if !problems.is_empty() {
                // Promote to cache so future runs are fast.
                cache::save(&problems);
                return problems;
            }
        }
    }

    // 5. Empty — scan works with built-in problems only.
    Vec::new()
}
