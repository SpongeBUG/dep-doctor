use anyhow::Result;
use crate::scanner::manifest::InstalledPackage;
use crate::scanner::repo_finder::Repo;

/// Reads go.mod and extracts direct + indirect dependencies.
pub fn read(repo: &Repo) -> Result<Vec<InstalledPackage>> {
    let path = repo.path.join("go.mod");
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&path)?;
    let packages = parse_go_mod(&content, &repo.name, &repo.path.display().to_string());
    Ok(packages)
}

fn parse_go_mod(content: &str, repo_name: &str, repo_path: &str) -> Vec<InstalledPackage> {
    let mut packages = Vec::new();
    let mut in_require_block = false;

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("require (") || line == "require (" {
            in_require_block = true;
            continue;
        }
        if in_require_block && line == ")" {
            in_require_block = false;
            continue;
        }

        // Single-line require: require github.com/foo/bar v1.2.3
        let target = if in_require_block {
            line
        } else if let Some(rest) = line.strip_prefix("require ") {
            rest.trim()
        } else {
            continue;
        };

        if let Some(pkg) = parse_go_dep_line(target, repo_name, repo_path) {
            packages.push(pkg);
        }
    }

    packages
}

fn parse_go_dep_line(line: &str, repo_name: &str, repo_path: &str) -> Option<InstalledPackage> {
    // Skip comments and replace directives
    let line = line.split("//").next().unwrap_or("").trim();
    if line.is_empty() || line.starts_with("//") {
        return None;
    }

    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let name = parts[0].to_string();
    let version = parts[1].trim_start_matches('v').to_string();

    Some(InstalledPackage {
        repo_name: repo_name.to_string(),
        repo_path: repo_path.to_string(),
        ecosystem: "go".into(),
        name,
        version,
    })
}
