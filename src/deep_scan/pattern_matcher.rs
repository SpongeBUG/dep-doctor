use regex::Regex;
use std::path::Path;

use crate::deep_scan::context_extractor::extract_context;
use crate::problems::schema::{SourceHit, SourcePattern};

/// Scan a single file for all provided patterns.
pub fn scan_file(path: &Path, patterns: &[SourcePattern]) -> Result<Vec<SourceHit>, anyhow::Error> {
    let content = match std::fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Ok(vec![]),
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut hits = Vec::new();

    for pattern in patterns {
        let Ok(re) = Regex::new(&pattern.regex) else {
            continue;
        };

        for (idx, line) in lines.iter().enumerate() {
            if re.is_match(line) {
                let context = extract_context(&lines, idx, 2);
                hits.push(SourceHit {
                    file: path.display().to_string(),
                    line_number: idx + 1,
                    line_content: line.trim().to_string(),
                    context,
                    pattern_description: pattern.description.clone(),
                    confidence: format!("{:?}", pattern.confidence).to_lowercase(),
                    remediation: pattern.remediation.clone(),
                });
            }
        }
    }

    Ok(hits)
}
