use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

use crate::scanner::manifest::InstalledPackage;
use crate::scanner::repo_finder::Repo;

#[derive(Deserialize)]
struct PackageJson {
    dependencies: Option<HashMap<String, String>>,
    #[serde(rename = "devDependencies")]
    dev_dependencies: Option<HashMap<String, String>>,
}

/// Reads package.json (and package-lock.json for resolved versions if present).
pub fn read(repo: &Repo) -> Result<Vec<InstalledPackage>> {
    let pkg_path = repo.path.join("package.json");
    if !pkg_path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&pkg_path)?;
    let pkg: PackageJson = serde_json::from_str(&content)?;

    let mut packages = Vec::new();

    let lock_versions = read_lock_versions(repo);

    let all_deps = pkg.dependencies.unwrap_or_default();
    // Include devDependencies too — dev-only vulns still matter in CI
    let dev_deps = pkg.dev_dependencies.unwrap_or_default();

    for (name, version_spec) in all_deps.iter().chain(dev_deps.iter()) {
        // Prefer lock file resolved version, fall back to spec with ^ ~ stripped
        let version = lock_versions
            .get(name)
            .cloned()
            .unwrap_or_else(|| clean_version(version_spec));

        if !version.is_empty() {
            packages.push(InstalledPackage {
                repo_name: repo.name.clone(),
                repo_path: repo.path.display().to_string(),
                ecosystem: "npm".into(),
                name: name.clone(),
                version,
            });
        }
    }

    Ok(packages)
}

/// Parse package-lock.json v2/v3 for exact resolved versions.
fn read_lock_versions(repo: &Repo) -> HashMap<String, String> {
    let lock_path = repo.path.join("package-lock.json");
    if !lock_path.exists() {
        return HashMap::new();
    }
    let Ok(content) = std::fs::read_to_string(&lock_path) else {
        return HashMap::new();
    };
    let Ok(lock) = serde_json::from_str::<serde_json::Value>(&content) else {
        return HashMap::new();
    };

    let mut map = HashMap::new();

    // lockfileVersion 2 & 3 use "packages" key
    if let Some(packages) = lock["packages"].as_object() {
        for (key, val) in packages {
            // key is like "node_modules/axios"
            if let Some(name) = key.strip_prefix("node_modules/") {
                if let Some(v) = val["version"].as_str() {
                    map.insert(name.to_string(), v.to_string());
                }
            }
        }
    }

    map
}

fn clean_version(spec: &str) -> String {
    spec.trim_start_matches(['^', '~', '=', '>', '<', ' '])
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}
