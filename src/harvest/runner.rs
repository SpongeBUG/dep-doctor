
//! Harvest runner — downloads per-ecosystem OSV zip files and filters to
//! the target package list. Zero per-advisory HTTP calls after the download.
//!
//! Source: https://storage.googleapis.com/osv-vulnerabilities/<ECO>/all.zip
//! Each zip contains one JSON file per advisory in OSV format.

use std::collections::{HashMap, HashSet};
use std::io::{Cursor, Read};

use anyhow::{Context, Result};
use indicatif::ProgressBar;

use crate::fetcher::converter;
use crate::fetcher::osv::Advisory;
use crate::harvest::packages::HarvestTarget;
use crate::problems::schema::Problem;

const GCS_BASE: &str = "https://storage.googleapis.com/osv-vulnerabilities";

static ECOSYSTEM_BUCKETS: &[(&str, &str)] = &[
    ("npm",       "npm"),
    ("PyPI",      "pip"),
    ("Go",        "go"),
    ("crates.io", "cargo"),
];

// ── Public API ─────────────────────────────────────────────────

pub fn run_with_progress(targets: &[&HarvestTarget], pb: &ProgressBar) -> Vec<Problem> {
    let mut wanted: HashMap<&str, HashSet<&str>> = HashMap::new();
    for t in targets {
        wanted.entry(t.osv_ecosystem).or_default().insert(t.name);
    }

    pb.set_length(ECOSYSTEM_BUCKETS.len() as u64);

    let mut all: HashMap<String, Problem> = HashMap::new();

    for (osv_eco, dep_eco) in ECOSYSTEM_BUCKETS {
        pb.set_message(format!("Downloading {osv_eco}..."));

        let pkg_filter = match wanted.get(osv_eco) {
            Some(s) => s,
            None => { pb.inc(1); continue; }
        };

        match process_ecosystem(osv_eco, dep_eco, pkg_filter) {
            Ok(problems) => {
                let n = problems.len();
                for p in problems {
                    all.entry(p.id.clone()).or_insert(p);
                }
                pb.set_message(format!("{osv_eco} done ({n} problems)"));
            }
            Err(e) => eprintln!("[harvest] {osv_eco} failed: {e}"),
        }

        pb.inc(1);
    }

    all.into_values().collect()
}

pub fn run(targets: &[&HarvestTarget]) -> Vec<Problem> {
    run_with_progress(targets, &ProgressBar::hidden())
}

// ── Private ────────────────────────────────────────────────────

fn process_ecosystem(
    osv_eco: &str,
    dep_eco: &str,
    pkg_filter: &HashSet<&str>,
) -> Result<Vec<Problem>> {
    let zip_bytes = download_ecosystem_zip(osv_eco)
        .with_context(|| format!("Failed to download {osv_eco} zip"))?;

    let pairs = extract_matching_advisories(&zip_bytes, pkg_filter)
        .with_context(|| format!("Failed to extract advisories from {osv_eco} zip"))?;

    let problems = pairs
        .into_iter()
        .flat_map(|(pkg_name, adv)| {
            converter::to_problems(std::slice::from_ref(&adv), dep_eco, &pkg_name)
        })
        .collect();

    Ok(problems)
}

fn download_ecosystem_zip(osv_eco: &str) -> Result<Vec<u8>> {
    let url = format!("{GCS_BASE}/{osv_eco}/all.zip");
    let resp = ureq::get(&url)
        .call()
        .with_context(|| format!("GET {url}"))?;
    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// Parse each JSON in the zip as a `serde_json::Value` to extract
/// `affected[].package.name`, then deserialize as `Advisory` separately.
/// This avoids the serde flatten+duplicate-field conflict.
fn extract_matching_advisories(
    zip_bytes: &[u8],
    pkg_filter: &HashSet<&str>,
) -> Result<Vec<(String, Advisory)>> {
    let mut archive = zip::ZipArchive::new(Cursor::new(zip_bytes))
        .context("Failed to open zip archive")?;

    let mut out = Vec::new();

    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if !file.name().ends_with(".json") {
            continue;
        }

        let mut content = String::new();
        if file.read_to_string(&mut content).is_err() {
            continue;
        }
        drop(file); // release borrow on archive

        let matched_pkg = matched_package_name(&content, pkg_filter);
        let pkg_name = match matched_pkg {
            Some(n) => n,
            None => continue,
        };

        let advisory: Advisory = match serde_json::from_str(&content) {
            Ok(a) => a,
            Err(_) => continue,
        };

        out.push((pkg_name, advisory));
    }

    Ok(out)
}

/// Extract the first `affected[].package.name` that matches `pkg_filter`.
/// Operates on raw `serde_json::Value` to avoid any struct field conflicts.
fn matched_package_name(json: &str, pkg_filter: &HashSet<&str>) -> Option<String> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let affected = v.get("affected")?.as_array()?;

    for entry in affected {
        let name = entry
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())?;

        if pkg_filter.iter().any(|f| f.eq_ignore_ascii_case(name)) {
            return Some(name.to_string());
        }
    }
    None
}
