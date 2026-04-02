//! HTTP fetcher for the problems.feed.json from GitHub Releases CDN.
//! One responsibility: download the feed and return Vec<Problem>.

use anyhow::{Context, Result};

use crate::problems::schema::Problem;

const FEED_URL: &str =
    "https://github.com/SpongeBUG/dep-doctor/releases/download/feeds%2Flatest/problems.feed.json";

/// Download the feed from GitHub Releases and parse it.
/// Returns an empty vec on network error so the caller degrades gracefully.
pub fn fetch() -> Result<Vec<Problem>> {
    let resp = ureq::get(FEED_URL)
        .call()
        .context("Failed to fetch problems feed from GitHub Releases")?;

    let text = resp
        .into_string()
        .context("Failed to read feed response body")?;

    let problems: Vec<Problem> =
        serde_json::from_str(&text).context("Failed to parse problems feed JSON")?;

    Ok(problems)
}
