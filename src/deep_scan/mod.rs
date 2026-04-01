pub mod context_extractor;
pub mod file_walker;
pub mod pattern_matcher;

use crate::problems::schema::{Problem, SourceHit};
use crate::scanner::repo_finder::Repo;
use anyhow::Result;

/// Entry point: given a repo and a matched problem, walk source files
/// and return every line that matches any of the problem's source patterns.
pub fn scan_repo(repo: &Repo, problem: &Problem) -> Result<Vec<SourceHit>> {
    let Some(sp) = &problem.source_patterns else {
        return Ok(vec![]);
    };

    let extensions = languages_to_extensions(&sp.languages);
    let files = file_walker::walk_source_files(&repo.path, &extensions)?;

    let mut hits = Vec::new();
    for file_path in files {
        let file_hits = pattern_matcher::scan_file(&file_path, &sp.patterns)?;
        hits.extend(file_hits);
    }

    Ok(hits)
}

fn languages_to_extensions(langs: &[String]) -> Vec<&'static str> {
    langs
        .iter()
        .flat_map(|l| match l.as_str() {
            "js" => vec!["js", "mjs", "cjs"],
            "ts" => vec!["ts", "tsx"],
            "py" => vec!["py"],
            "go" => vec!["go"],
            "rs" => vec!["rs"],
            _ => vec![],
        })
        .collect()
}
