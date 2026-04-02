use anyhow::Result;
use serde::Serialize;
use std::io::Write;
use std::path::Path;

use crate::problems::schema::Finding;
use crate::supply_chain::typosquat::TyposquatWarning;

#[derive(Serialize)]
struct JsonReport<'a> {
    findings: &'a [Finding<'a>],
    typosquat_warnings: &'a [TyposquatWarning],
}

pub fn report(
    findings: &[Finding],
    typosquat_warnings: &[TyposquatWarning],
    output: Option<&Path>,
) -> Result<()> {
    let report = JsonReport {
        findings,
        typosquat_warnings,
    };
    let json = serde_json::to_string_pretty(&report)?;
    write_output(json.as_bytes(), output)
}

fn write_output(content: &[u8], output: Option<&Path>) -> Result<()> {
    match output {
        Some(path) => {
            std::fs::write(path, content)?;
            eprintln!("Written to {}", path.display());
        }
        None => std::io::stdout().write_all(content)?,
    }
    Ok(())
}
