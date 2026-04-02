use std::collections::HashSet;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::args::{ReporterArg, ScanArgs};
use crate::deep_scan;
use crate::llm::{self, LlmConfig};
use crate::problems::registry::all_problems;
use crate::problems::schema::{Finding, Problem};
use crate::reporter::{console, json, markdown};
use crate::scanner::manifest;
use crate::scanner::{repo_finder, version_matcher};
use crate::supply_chain::typosquat;
use crate::{log_debug, log_warn};

pub fn run(args: ScanArgs) -> Result<()> {
    let repos = repo_finder::find_repos(&args.path)?;

    if repos.is_empty() {
        println!("No repos found in {}", args.path.display());
        return Ok(());
    }

    // Resolve LLM config early so we can warn before the scan starts.
    let llm_config = resolve_llm_config(&args);

    // Layer 1: built-in problems (always present).
    let mut problems = all_problems();

    // Layer 2: nightly feed (default enrichment — no flag required).
    let feed_problems = crate::feed::load_feed();
    merge_problems(&mut problems, feed_problems);

    // Layer 3: --online adds live OSV on top of feed + built-in.
    let all_repo_packages = read_all_packages(&repos)?;
    if args.online {
        let osv_problems = crate::fetcher::query_packages(&all_repo_packages);
        merge_problems(&mut problems, osv_problems);
    }

    // LLM pattern generation pass (opt-in).
    if let Some(ref config) = llm_config {
        enrich_patterns(&mut problems, config);
    }

    let pb = build_progress_bar(repos.len() as u64);
    let mut all_findings: Vec<Finding> = Vec::new();

    for repo in &repos {
        pb.set_message(format!("Scanning {}", repo.name));

        let packages = manifest::read_all(repo)?;
        let matches = version_matcher::match_problems(&packages, &problems);

        let mut matches = apply_deep_scan(matches, &args, repo)?;

        let min_sev = args.severity.clone();
        matches.retain(|f| f.problem.severity_rank() >= min_sev.rank());

        all_findings.extend(matches);
        pb.inc(1);
    }

    pb.finish_and_clear();

    // Supply chain: typosquat check on all scanned packages.
    let typosquat_warnings = typosquat::check(&all_repo_packages);

    match args.reporter {
        ReporterArg::Console => console::report(&all_findings, &typosquat_warnings, args.summary()),
        ReporterArg::Json => {
            json::report(&all_findings, &typosquat_warnings, args.output.as_deref())
        }
        ReporterArg::Markdown => {
            markdown::report(&all_findings, &typosquat_warnings, args.output.as_deref())
        }
    }
}

/// Resolve LLM config when `--generate-patterns` is requested.
/// Warns and returns `None` if the flag is set but the API key is missing.
fn resolve_llm_config(args: &ScanArgs) -> Option<LlmConfig> {
    if !args.generate_patterns {
        return None;
    }

    match LlmConfig::from_env() {
        Some(config) => {
            log_debug!(
                "LLM pattern generation enabled (model={}, endpoint={})",
                config.model,
                config.endpoint,
            );
            Some(config)
        }
        None => {
            log_warn!("--generate-patterns requires DEP_DOCTOR_LLM_API_KEY env var; skipping");
            None
        }
    }
}

/// Walk all problems and generate source patterns for those that lack them.
fn enrich_patterns(problems: &mut [Problem], config: &LlmConfig) {
    let need_patterns = problems
        .iter()
        .filter(|p| p.source_patterns.is_none())
        .count();

    if need_patterns == 0 {
        return;
    }

    let pb = ProgressBar::new(need_patterns as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.magenta} [{bar:30.magenta/blue}] {pos}/{len} generating patterns {msg}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );

    let mut generated = 0usize;
    for problem in problems.iter_mut() {
        if problem.source_patterns.is_some() {
            continue;
        }

        pb.set_message(problem.id.clone());
        if let Some(patterns) = llm::generate_patterns(problem, config) {
            problem.source_patterns = Some(patterns);
            generated += 1;
        }
        pb.inc(1);
    }

    pb.finish_and_clear();
    log_debug!("Generated patterns for {generated}/{need_patterns} problems");
}

/// Merge `incoming` into `base`, skipping any ID already present (base wins).
fn merge_problems(
    base: &mut Vec<crate::problems::schema::Problem>,
    incoming: Vec<crate::problems::schema::Problem>,
) {
    let existing_ids: HashSet<String> = base.iter().map(|p| p.id.clone()).collect();
    for p in incoming {
        if !existing_ids.contains(&p.id) {
            base.push(p);
        }
    }
}

/// Run deep scan if requested and there are findings.
fn apply_deep_scan<'a>(
    mut matches: Vec<Finding<'a>>,
    args: &ScanArgs,
    repo: &repo_finder::Repo,
) -> Result<Vec<Finding<'a>>> {
    if args.deep_enabled() && !matches.is_empty() {
        for finding in &mut matches {
            finding.source_hits = deep_scan::scan_repo(repo, finding.problem)?;
        }
    }
    Ok(matches)
}

/// Read all packages from all repos in one pass (used for OSV batch query).
fn read_all_packages(repos: &[repo_finder::Repo]) -> Result<Vec<manifest::InstalledPackage>> {
    let mut all = Vec::new();
    for repo in repos {
        all.extend(manifest::read_all(repo)?);
    }
    Ok(all)
}

fn build_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template("{spinner:.cyan} [{bar:30.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("=>-"),
    );
    pb
}
