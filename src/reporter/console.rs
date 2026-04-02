use anyhow::Result;
use colored::Colorize;
use std::collections::HashMap;

use crate::problems::schema::{Finding, ProblemKind};
use crate::supply_chain::typosquat::TyposquatWarning;

pub fn report(
    findings: &[Finding],
    typosquat_warnings: &[TyposquatWarning],
    show_summary: bool,
) -> Result<()> {
    if findings.is_empty() && typosquat_warnings.is_empty() {
        println!("{}", "\n✓ No known problems found.\n".green().bold());
        return Ok(());
    }

    if !findings.is_empty() {
        print_findings(findings);
    }

    if !typosquat_warnings.is_empty() {
        print_typosquat_warnings(typosquat_warnings);
    }

    if show_summary {
        print_summary(findings, typosquat_warnings);
    }

    Ok(())
}

fn print_findings(findings: &[Finding]) {
    let mut by_repo: HashMap<&str, Vec<&Finding>> = HashMap::new();
    for f in findings {
        by_repo.entry(&f.repo_name).or_default().push(f);
    }

    let mut repo_names: Vec<&str> = by_repo.keys().copied().collect();
    repo_names.sort();

    for repo_name in repo_names {
        let repo_findings = &by_repo[repo_name];
        println!("\n{} {}", "◆ repo:".bold(), repo_name.bold().cyan());

        for f in repo_findings.iter() {
            let label = kind_label(f);
            println!(
                "  {} {} {} @ {} {} {}",
                label,
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
}

fn print_typosquat_warnings(warnings: &[TyposquatWarning]) {
    println!("\n{}", "⚠ Possible typosquats detected".yellow().bold());
    for w in warnings {
        println!(
            "  {} {} looks like {} (edit distance {}, ecosystem: {})",
            "⚠".yellow(),
            w.scanned_name.red().bold(),
            w.similar_to.cyan().bold(),
            w.edit_distance,
            w.ecosystem.dimmed()
        );
    }
}

fn print_summary(findings: &[Finding], typosquat_warnings: &[TyposquatWarning]) {
    let mut counts: HashMap<&str, usize> = HashMap::new();
    for f in findings {
        *counts.entry(f.problem.severity.as_str()).or_insert(0) += 1;
    }

    println!(
        "\n{}",
        "─── Summary ───────────────────────────────".dimmed()
    );
    for sev in &["critical", "high", "medium", "low", "info"] {
        if let Some(count) = counts.get(sev) {
            println!("  {} {}", severity_colored(sev), count);
        }
    }

    let supply_chain_count = findings
        .iter()
        .filter(|f| f.problem.kind == ProblemKind::SupplyChain)
        .count();
    if supply_chain_count > 0 {
        println!("  {} {}", "[SUPPLY CHAIN]".red().bold(), supply_chain_count);
    }

    if !typosquat_warnings.is_empty() {
        println!(
            "  {} {} possible typosquat(s)",
            "⚠".yellow().bold(),
            typosquat_warnings.len()
        );
    }

    let total_hits: usize = findings.iter().map(|f| f.source_hits.len()).sum();
    if total_hits > 0 {
        println!(
            "  {} source locations flagged",
            total_hits.to_string().yellow().bold()
        );
    }
    println!(
        "{}",
        "───────────────────────────────────────────\n".dimmed()
    );
}

/// Render severity or supply-chain label for a finding.
fn kind_label(finding: &Finding) -> colored::ColoredString {
    match finding.problem.kind {
        ProblemKind::SupplyChain => "[SUPPLY CHAIN]".red().bold(),
        ProblemKind::Cve => severity_colored(&finding.problem.severity),
    }
}

pub fn severity_colored(sev: &str) -> colored::ColoredString {
    match sev {
        "critical" => format!("[{:8}]", "CRITICAL").red().bold(),
        "high" => format!("[{:8}]", "HIGH").red(),
        "medium" => format!("[{:8}]", "MEDIUM").yellow(),
        "low" => format!("[{:8}]", "LOW").blue(),
        _ => format!("[{:8}]", "INFO").dimmed(),
    }
}
