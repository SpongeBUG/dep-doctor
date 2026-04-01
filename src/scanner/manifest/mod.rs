pub mod cargo;
pub mod go;
pub mod npm;
pub mod pip;

use crate::scanner::repo_finder::Repo;
use anyhow::Result;

/// A resolved package version found inside a repo.
#[derive(Debug)]
pub struct InstalledPackage {
    pub repo_name: String,
    pub repo_path: String,
    pub ecosystem: String,
    pub name: String,
    pub version: String,
}

/// Read all manifests from a repo and return every installed package.
pub fn read_all(repo: &Repo) -> Result<Vec<InstalledPackage>> {
    let mut packages = Vec::new();
    packages.extend(npm::read(repo)?);
    packages.extend(pip::read(repo)?);
    packages.extend(go::read(repo)?);
    packages.extend(cargo::read(repo)?);
    Ok(packages)
}
