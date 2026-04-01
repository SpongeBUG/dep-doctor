use anyhow::Result;
use std::fmt::Write as FmtWrite;
use std::path::Path;

use crate::problems::schema::Finding;

pub fn report(findings: &[Finding], output: Option<&Path>) -> Result<()> {
    let mut md = String::new();

    writeln!(md, "# dep-doctor Report\n")?;

    if findings.is_empty() {
        writeln!(md, "✅ No known problems found.")?;
    } else {
        writeln!(md, "| Repo | Severity | Problem | Package | Version | Fix |")?;
        writeln!(md, "|------|----------|---------|---------|---------|-----|")?;

        for f in findings {
            writeln!(
                md,
                "| {} | {} | {} | {} | {} | {} |",
                f.repo_name,
                f.problem.severity.to_uppercase(),
                f.problem.id,
                f.package,
                f.installed_version,
                f.problem.fixed_in.as_deref().unwrap_or("—"),
            )?;
        }

        writeln!(md)?;

        for f in findings {
            writeln!(md, "## {} — {}", f.repo_name, f.problem.id)?;
            writeln!(md, "**{}**\n", f.problem.title)?;
            writeln!(md, "- **Package**: `{}` @ `{}`", f.package, f.installed_version)?;
            writeln!(md, "- **Severity**: {}", f.problem.severity.to_uppercase())?;
            if let Some(fix) = &f.problem.fixed_in {
                writeln!(md, "- **Fix**: upgrade to `>={}`", fix)?;
            }
            for r in &f.problem.references {
                writeln!(md, "- {}", r)?;
            }

            if !f.source_hits.is_empty() {
                writeln!(md, "\n### Affected source locations\n")?;
                for hit in &f.source_hits {
                    writeln!(md, "**`{}` line {}** [{}]", hit.file, hit.line_number, hit.confidence)?;
                    writeln!(md, "```")?;
                    writeln!(md, "{}", hit.line_content)?;
                    writeln!(md, "```")?;
                    writeln!(md, "> {}\n", hit.remediation)?;
                }
            }
            writeln!(md)?;
        }
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
