use anyhow::Result;
use colored::Colorize;

use crate::cli::args::{EcosystemArg, ProblemsAction, ProblemsArgs};
use crate::problems::registry::all_problems;

pub fn run(args: ProblemsArgs) -> Result<()> {
    match args.action {
        ProblemsAction::List { ecosystem } => list(ecosystem),
        ProblemsAction::Show { id } => show(&id),
    }
}

fn list(ecosystem: Option<EcosystemArg>) -> Result<()> {
    let problems = all_problems();
    let filtered: Vec<_> = problems
        .iter()
        .filter(|p| match &ecosystem {
            None => true,
            Some(e) => {
                let want = match e {
                    EcosystemArg::Npm   => "npm",
                    EcosystemArg::Pip   => "pip",
                    EcosystemArg::Go    => "go",
                    EcosystemArg::Cargo => "cargo",
                };
                p.ecosystem.to_lowercase() == want
            }
        })
        .collect();

    if filtered.is_empty() {
        println!("{}", "No problems found.".yellow());
        return Ok(());
    }

    println!("{}", format!("{} known problems:\n", filtered.len()).bold());
    for p in &filtered {
        let sev = crate::reporter::console::severity_colored(&p.severity);
        println!(
            "  {} {} {} ({})",
            sev,
            p.id.bold(),
            p.title.dimmed(),
            p.package.cyan()
        );
    }
    println!();
    println!("Run {} to see details.", "dep-doctor problems show <ID>".cyan());
    Ok(())
}

fn show(id: &str) -> Result<()> {
    let problems = all_problems();
    match problems.iter().find(|p| p.id == id) {
        None => {
            eprintln!("{} Unknown problem ID: {}", "error:".red().bold(), id);
            std::process::exit(1);
        }
        Some(p) => {
            println!("\n{} {}", "●".bold(), p.id.bold().cyan());
            println!("  Title     : {}", p.title);
            println!("  Severity  : {}", crate::reporter::console::severity_colored(&p.severity));
            println!("  Ecosystem : {}", p.ecosystem);
            println!("  Package   : {}", p.package.cyan());
            println!("  Affected  : {}", p.affected_range.yellow());
            if let Some(fix) = &p.fixed_in {
                println!("  Fixed in  : {}", fix.green());
            }
            if !p.references.is_empty() {
                println!("  References:");
                for r in &p.references {
                    println!("    - {}", r.dimmed());
                }
            }
            if let Some(sp) = &p.source_patterns {
                println!("  Deep-scan patterns ({}):", sp.patterns.len());
                for pat in &sp.patterns {
                    println!(
                        "    [{}] {}",
                        format!("{:?}", pat.confidence).to_lowercase().yellow(),
                        pat.description
                    );
                }
            }
            println!();
        }
    }
    Ok(())
}
