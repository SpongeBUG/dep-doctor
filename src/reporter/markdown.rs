use anyhow::Result;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use crate::problems::schema::{Finding, ProblemKind};
use crate::supply_chain::typosquat::TyposquatWarning;

pub fn report(
    findings: &[Finding],
    typosquat_warnings: &[TyposquatWarning],
    output: Option<&Path>,
) -> Result<()> {
    let mut md = String::new();

    writeln!(md, "# dep-doctor Report\n")?;

    if findings.is_empty() && typosquat_warnings.is_empty() {
        writeln!(md, "✅ No known problems found.")?;
    } else {
        write_findings_table(&mut md, findings)?;
        write_finding_details(&mut md, findings)?;
        write_typosquat_section(&mut md, typosquat_warnings)?;
    }

    match output {
        Some(path) => {
            std::fs::write(path, &md)?;
            eprintln!("Written to {}", path.display());
        }
        None => print!("{}", md),
    }

    Ok(())
}

fn write_findings_table(md: &mut String, findings: &[Finding]) -> Result<()> {
    if findings.is_empty() {
        return Ok(());
    }

    writeln!(md, "| Repo | Kind | Problem | Package | Version | Fix |")?;
    writeln!(md, "|------|------|---------|---------|---------|-----|")?;

    for f in findings {
        let label = kind_label(f);
        writeln!(
            md,
            "| {} | {} | {} | {} | {} | {} |",
            f.repo_name,
            label,
            f.problem.id,
            f.package,
            f.installed_version,
            f.problem.fixed_in.as_deref().unwrap_or("—"),
        )?;
    }

    writeln!(md)?;
    Ok(())
}

fn write_finding_details(md: &mut String, findings: &[Finding]) -> Result<()> {
    for f in findings {
        writeln!(md, "## {} — {}", f.repo_name, f.problem.id)?;
        writeln!(md, "**{}**\n", f.problem.title)?;
        writeln!(
            md,
            "- **Package**: `{}` @ `{}`",
            f.package, f.installed_version
        )?;
        writeln!(md, "- **Kind**: {}", kind_label(f))?;
        if let Some(fix) = &f.problem.fixed_in {
            writeln!(md, "- **Fix**: upgrade to `>={}`", fix)?;
        }
        for r in &f.problem.references {
            writeln!(md, "- {}", r)?;
        }

        if !f.source_hits.is_empty() {
            writeln!(md, "\n### Affected source locations\n")?;
            for hit in &f.source_hits {
                writeln!(
                    md,
                    "**`{}` line {}** [{}]",
                    hit.file, hit.line_number, hit.confidence
                )?;
                writeln!(md, "```")?;
                writeln!(md, "{}", hit.line_content)?;
                writeln!(md, "```")?;
                writeln!(md, "> {}\n", hit.remediation)?;
            }
        }
        writeln!(md)?;
    }
    Ok(())
}

fn write_typosquat_section(md: &mut String, warnings: &[TyposquatWarning]) -> Result<()> {
    if warnings.is_empty() {
        return Ok(());
    }

    writeln!(md, "## ⚠ Possible Typosquats\n")?;
    writeln!(md, "| Package | Ecosystem | Similar To | Edit Distance |")?;
    writeln!(md, "|---------|-----------|------------|---------------|")?;

    for w in warnings {
        writeln!(
            md,
            "| `{}` | {} | `{}` | {} |",
            w.scanned_name, w.ecosystem, w.similar_to, w.edit_distance
        )?;
    }

    writeln!(md)?;
    Ok(())
}

fn kind_label(finding: &Finding) -> &'static str {
    match finding.problem.kind {
        ProblemKind::SupplyChain => "SUPPLY CHAIN",
        ProblemKind::Cve => match finding.problem.severity.as_str() {
            "critical" => "CRITICAL",
            "high" => "HIGH",
            "medium" => "MEDIUM",
            "low" => "LOW",
            _ => "INFO",
        },
    }
}
