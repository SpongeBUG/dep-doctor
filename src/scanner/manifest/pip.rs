use crate::scanner::manifest::InstalledPackage;
use crate::scanner::repo_finder::Repo;
use anyhow::Result;

/// Reads requirements.txt and pyproject.toml [project.dependencies].
pub fn read(repo: &Repo) -> Result<Vec<InstalledPackage>> {
    let mut packages = Vec::new();
    packages.extend(read_requirements_txt(repo)?);
    packages.extend(read_pyproject_toml(repo)?);
    Ok(packages)
}

fn read_requirements_txt(repo: &Repo) -> Result<Vec<InstalledPackage>> {
    let path = repo.path.join("requirements.txt");
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&path)?;
    let packages = content
        .lines()
        .filter_map(|line| {
            parse_requirement_line(line, &repo.name, &repo.path.display().to_string())
        })
        .collect();

    Ok(packages)
}

fn parse_requirement_line(
    line: &str,
    repo_name: &str,
    repo_path: &str,
) -> Option<InstalledPackage> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') || line.starts_with('-') {
        return None;
    }

    let (name, version) = split_requirement(line)?;

    Some(InstalledPackage {
        repo_name: repo_name.to_string(),
        repo_path: repo_path.to_string(),
        ecosystem: "pip".into(),
        name: normalize_pip_name(&name),
        version,
    })
}

fn split_requirement(spec: &str) -> Option<(String, String)> {
    let base = spec.split('[').next().unwrap_or(spec);

    for op in &["==", "~=", ">=", "<=", ">", "<", "!="] {
        if let Some(idx) = base.find(op) {
            let name = base[..idx].trim().to_string();
            let version = spec[idx + op.len()..]
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .to_string();
            if !name.is_empty() && !version.is_empty() {
                return Some((name, version));
            }
        }
    }
    None
}

fn read_pyproject_toml(repo: &Repo) -> Result<Vec<InstalledPackage>> {
    let path = repo.path.join("pyproject.toml");
    if !path.exists() {
        return Ok(vec![]);
    }

    let content = std::fs::read_to_string(&path)?;
    let Ok(doc) = content.parse::<toml::Value>() else {
        return Ok(vec![]);
    };

    let Some(deps) = doc
        .get("project")
        .and_then(|p| p.get("dependencies"))
        .and_then(|d| d.as_array())
    else {
        return Ok(vec![]);
    };

    let packages = deps
        .iter()
        .filter_map(|v| v.as_str())
        .filter_map(|s| parse_requirement_line(s, &repo.name, &repo.path.display().to_string()))
        .collect();

    Ok(packages)
}

fn normalize_pip_name(name: &str) -> String {
    name.to_lowercase().replace('-', "_")
}
