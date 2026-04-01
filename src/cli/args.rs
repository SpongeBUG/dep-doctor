use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "dep-doctor",
    version,
    about = "Scan repos for known dependency problems and find affected source files",
    long_about = None,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan a folder of repos for known dependency problems
    Scan(ScanArgs),
    /// List or show details about known problems
    Problems(ProblemsArgs),
}

#[derive(Args)]
pub struct ScanArgs {
    /// Root folder containing repos to scan (default: current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Deep scan: search source files for affected usage patterns
    #[arg(long, short = 'd')]
    pub deep: bool,

    /// Only scan a specific ecosystem
    #[arg(long, short = 'e', value_enum)]
    pub ecosystem: Option<EcosystemArg>,

    /// Output format
    #[arg(long, short = 'r', value_enum, default_value = "console")]
    pub reporter: ReporterArg,

    /// Output file (for json/markdown reporters); omit to print to stdout
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// Minimum severity to report
    #[arg(long, short = 's', value_enum, default_value = "low")]
    pub severity: SeverityArg,

    /// Show a summary table at the end
    #[arg(long, default_value = "true")]
    pub summary: bool,
}

#[derive(Args)]
pub struct ProblemsArgs {
    #[command(subcommand)]
    pub action: ProblemsAction,
}

#[derive(Subcommand)]
pub enum ProblemsAction {
    /// List all known problems
    List {
        #[arg(long, short = 'e', value_enum)]
        ecosystem: Option<EcosystemArg>,
    },
    /// Show details for a specific problem
    Show {
        /// Problem ID (e.g. npm-axios-csrf-ssrf-CVE-2023-45857)
        id: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum EcosystemArg {
    Npm,
    Pip,
    Go,
    Cargo,
}

#[derive(Clone, ValueEnum)]
pub enum ReporterArg {
    Console,
    Json,
    Markdown,
}

#[derive(Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityArg {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl SeverityArg {
    pub fn rank(&self) -> u8 {
        match self {
            Self::Info => 1,
            Self::Low => 2,
            Self::Medium => 3,
            Self::High => 4,
            Self::Critical => 5,
        }
    }
}
