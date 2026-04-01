use anyhow::Result;
use std::io::Write;
use std::path::Path;

use crate::problems::schema::Finding;

pub fn report(findings: &[Finding], output: Option<&Path>) -> Result<()> {
    let json = serde_json::to_string_pretty(findings)?;
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
