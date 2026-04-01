use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

use crate::scanner::manifest::InstalledPackage;
use crate::scanner::repo_finder::Repo;

#[derive(Deserialize)]
struct CargoToml {
    dependencies: Option<HashMap<String, toml::Value>>,
    #[serde(rename = "dev-dependencies")]
    dev_dependencies: Option<HashMap<String, toml::Value>>,
}

/// Reads Cargo.toml for dependency versions.
/// Prefers Cargo.lock exact versions when available.
pub fn read(repo: &Repo) -> Result<Vec<InstalledPackage>> {
    let path = repo.path.join("Cargo.toml");
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&path)?;
    let cargo: CargoToml = toml::from_str(&content)?;

    let lock_versions = read_lock_versions(repo);
    let mut packages = Vec::new();

    let all_deps = cargo.dependencies.unwrap_or_default();
    let dev_deps = cargo.dev_dependencies.unwrap_or_default();

    for (name, spec) in all_deps.iter().chain(dev_deps.iter()) {
        let version = lock_versions
            .get(name)
            .cloned()
            .unwrap_or_else(|| extract_version_from_spec(spec));

        if !version.is_empty() {
            packages.push(InstalledPackage {
                repo_name: repo.name.clone(),
                repo_path: repo.path.display().to_string(),
                ecosystem: "cargo".into(),
                name: name.clone(),
                version,
            });
        }
    }

    Ok(packages)
}

fn extract_version_from_spec(spec: &toml::Value) -> String {
    match spec {
        toml::Value::String(s) => clean_version(s),
        toml::Value::Table(t) => t
            .get("version")
            .and_then(|v| v.as_str())
            .map(clean_version)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn clean_version(s: &str) -> String {
    s.trim_start_matches(['^', '~', '=', '>', '<', ' '])
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_string()
}

fn read_lock_versions(repo: &Repo) -> HashMap<String, String> {
    let lock_path = repo.path.join("Cargo.lock");
    if !lock_path.exists() {
        return HashMap::new();
    }
    let Ok(content) = std::fs::read_to_string(&lock_path) else {
        return HashMap::new();
    };
    let Ok(lock) = content.parse::<toml::Value>() else {
        return HashMap::new();
    };

    let mut map = HashMap::new();
    if let Some(packages) = lock.get("package").and_then(|p| p.as_array()) {
        for pkg in packages {
            if let (Some(name), Some(version)) = (
                pkg.get("name").and_then(|n| n.as_str()),
                pkg.get("version").and_then(|v| v.as_str()),
            ) {
                map.entry(name.to_string())
                    .or_insert_with(|| version.to_string());
            }
        }
    }
    map
}
