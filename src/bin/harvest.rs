//! Harvester binary — downloads OSV ecosystem zips and writes problems.feed.json.
//! Run with: cargo run --bin harvest

use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};

fn main() -> Result<()> {
    let start = Instant::now();

    let targets = dep_doctor::harvest::packages::all_targets();
    println!("[harvest] {} targets across 4 ecosystems", targets.len());

    let pb = build_progress_bar(4);
    let problems = dep_doctor::harvest::runner::run_with_progress(&targets, &pb);
    pb.finish_and_clear();

    let count = problems.len();
    let out_path = output_path();
    write_feed(&problems, &out_path)?;

    let elapsed = start.elapsed().as_secs_f32();
    println!(
        "[harvest] Done: {count} problems → {} ({elapsed:.1}s)",
        out_path.display()
    );

    Ok(())
}

fn output_path() -> PathBuf {
    std::env::var("FEED_OUTPUT_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("problems.feed.json"))
}

fn write_feed(problems: &[dep_doctor::problems::schema::Problem], path: &PathBuf) -> Result<()> {
    let json = serde_json::to_string_pretty(problems).context("Failed to serialize problems")?;
    fs::write(path, json).with_context(|| format!("Failed to write feed to {}", path.display()))?;
    Ok(())
}

fn build_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{bar:35.cyan/blue}] {pos}/{len} ecosystems | {msg}",
        )
        .unwrap()
        .progress_chars("=>-"),
    );
    pb
}
