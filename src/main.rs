use anyhow::Result;
use clap::Parser;
use dep_doctor::cli::args::{Cli, Commands};
use dep_doctor::cli::commands;

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Scan(args) => commands::scan::run(args),
        Commands::Problems(args) => commands::problems::run(args),
    }
}
