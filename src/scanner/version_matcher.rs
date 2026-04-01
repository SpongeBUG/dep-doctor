use crate::problems::schema::{Finding, Problem};
use crate::scanner::manifest::InstalledPackage;
use crate::utils::semver_utils::version_matches_range;

/// Matches installed packages against known problems.
/// Returns a Finding for each (repo, package, problem) hit.
pub fn match_problems<'a>(
    packages: &[InstalledPackage],
    problems: &'a [Problem],
) -> Vec<Finding<'a>> {
    let mut findings = Vec::new();

    for pkg in packages {
        for problem in problems {
            if problem.ecosystem.to_lowercase() != pkg.ecosystem.to_lowercase() {
                continue;
            }
            if problem.package != pkg.name {
                continue;
            }
            if version_matches_range(&pkg.version, &problem.affected_range) {
                findings.push(Finding {
                    repo_name: pkg.repo_name.clone(),
                    repo_path: pkg.repo_path.clone(),
                    package: pkg.name.clone(),
                    installed_version: pkg.version.clone(),
                    problem,
                    source_hits: Vec::new(),
                });
            }
        }
    }

    findings
}
