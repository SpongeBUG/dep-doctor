use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use crate::problems::schema::Finding;

pub fn report(findings: &[Finding], show_summary: bool) -> Result<()> {
    if findings.is_empty() {
        println!("{}", "\n✓ No known problems found.\n".green().bold());
        return Ok(());
    }

    // Group by repo
    let mut by_repo: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for f in findings {
        by_repo.entry(&f.repo_name).or_default().push(f);
    }

    let mut repo_names: Vec<&str> = by_repo.keys().copied().collect();
    repo_names.sort();

    for repo_name in repo_names {
        let repo_findings = &by_repo[repo_name];
        println!(
            "\n{} {}",
            "◆ repo:".bold(),
            repo_name.bold().cyan()
        );

        for f in repo_findings.iter() {
            let sev = severity_colored(&f.problem.severity);
            println!(
                "  {} {} {} @ {} {} {}",
                sev,
                f.problem.id.bold(),
                f.package.cyan(),
                f.installed_version.yellow(),
                "→".dimmed(),
                f.problem.fixed_in.as_deref().unwrap_or("no fix").green()
            );
            println!("     {}", f.problem.title.dimmed());

            if !f.source_hits.is_empty() {
                println!(
                    "     {} {} affected location(s):",
                    "⚑".yellow(),
                    f.source_hits.len()
                );
                for hit in &f.source_hits {
                    println!(
                        "       {} line {} [{}]",
                        hit.file.dimmed(),
                        hit.line_number.to_string().yellow(),
                        hit.confidence.yellow()
                    );
                    println!("         {}", hit.line_content.dimmed());
                    println!("         → {}", hit.remediation.green());
                }
            }
        }
    }

    if show_summary {
        print_summary(findings);
    }

    Ok(())
}

fn print_summary(findings: &[Finding]) {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for f in findings {
        *counts.entry(f.problem.severity.as_str()).or_insert(0) += 1;
    }

    println!("\n{}", "─── Summary ───────────────────────────────".dimmed());
    for sev in &["critical", "high", "medium", "low", "info"] {
        if let Some(count) = counts.get(sev) {
            println!("  {} {}", severity_colored(sev), count);
        }
    }
    let total_hits: usize = findings.iter().map(|f| f.source_hits.len()).sum();
    if total_hits > 0 {
        println!("  {} source locations flagged", total_hits.to_string().yellow().bold());
    }
    println!("{}", "───────────────────────────────────────────\n".dimmed());
}

pub fn severity_colored(sev: &str) -> colored::ColoredString {
    match sev {
        "critical" => format!("[{:8}]", "CRITICAL").red().bold(),
        "high"     => format!("[{:8}]", "HIGH").red(),
        "medium"   => format!("[{:8}]", "MEDIUM").yellow(),
        "low"      => format!("[{:8}]", "LOW").blue(),
        _          => format!("[{:8}]", "INFO").dimmed(),
    }
}
