/// Extract `context_lines` lines above and below a match index.
/// Returns them as strings with line-number prefixes.
pub fn extract_context(lines: &[&str], match_idx: usize, context_lines: usize) -> Vec<String> {
    let start = match_idx.saturating_sub(context_lines);
    let end = (match_idx + context_lines + 1).min(lines.len());

    lines[start..end]
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = start + i + 1;
            let marker = if start + i == match_idx { ">" } else { " " };
            format!("{} {:4} | {}", marker, line_num, line)
        })
        .collect()
}
