use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};

use crate::cli::args::{ReporterArg, ScanArgs};
use crate::problems::registry::all_problems;
use crate::reporter::{console, json, markdown};
use crate::scanner::{repo_finder, version_matcher};
use crate::scanner::manifest;
use crate::deep_scan;
use crate::problems::schema::Finding;

pub fn run(args: ScanArgs) -> Result<()> {
    let repos = repo_finder::find_repos(&args.path)?;

    if repos.is_empty() {
        println!("No repos found in {}", args.path.display());
        return Ok(());
    }

    let problems = all_problems();
    let pb = build_progress_bar(repos.len() as u64);
    let mut all_findings: Vec<Finding> = Vec::new();

    for repo in &repos {
        pb.set_message(format!("Scanning {}", repo.name));

        let packages = manifest::read_all(repo)?;
        let mut matches = version_matcher::match_problems(&packages, &problems);

        if args.deep && !matches.is_empty() {
            for finding in &mut matches {
                finding.source_hits =
                    deep_scan::scan_repo(repo, finding.problem)?;
            }
        }

        // Filter by minimum severity
        let min_sev = args.severity.clone();
        let filtered = matches
            .into_iter()
            .filter(|f| f.problem.severity_rank() >= min_sev.rank())
            .collect::<Vec<_>>();

        all_findings.extend(filtered);
        pb.inc(1);
    }

    pb.finish_and_clear();

    match args.reporter {
        ReporterArg::Console => console::report(&all_findings, args.summary),
        ReporterArg::Json => json::report(&all_findings, args.output.as_deref()),
        ReporterArg::Markdown => markdown::report(&all_findings, args.output.as_deref()),
    }
}

fn build_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{bar:30.cyan/blue}] {pos}/{len} {msg}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    pb
}
