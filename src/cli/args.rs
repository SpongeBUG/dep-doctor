use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "dep-doctor",
    version,
    about = "Scan repos for known dependency problems and find affected source files",
    long_about = "dep-doctor scans your repositories for known dependency vulnerabilities, \
                   supply chain attacks, and typosquatted packages.\n\n\
                   It checks installed packages against a curated problem database, a nightly \
                   OSV feed, and optionally live OSV.dev queries. With --deep, it also searches \
                   your source files for vulnerable usage patterns.",
    after_help = "\x1b[1mExamples:\x1b[0m\n  \
                   dep-doctor scan .                      Scan repos in current directory\n  \
                   dep-doctor scan ./projects --deep       Deep scan with source pattern matching\n  \
                   dep-doctor scan . --online -s high      Live OSV lookup, high+ severity only\n  \
                   dep-doctor scan . -r json -o report.json Export findings as JSON\n  \
                   dep-doctor scan . --deep --generate-patterns  LLM-generated deep-scan patterns\n  \
                   dep-doctor scan . --deep --pattern-stats Show pattern quality report\n  \
                   dep-doctor problems list                List all known problems\n  \
                   dep-doctor problems list -e npm         List npm problems only\n  \
                   dep-doctor problems show <ID>           Show details for a specific problem"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan a folder of repos for known dependency problems
    Scan(ScanArgs),
    /// List, filter, or inspect known vulnerability definitions
    Problems(ProblemsArgs),
}

#[derive(Args)]
#[command(
    after_help = "\x1b[1mProblem sources (layered, built-in wins on ID conflict):\x1b[0m\n  \
                   1. Built-in    4 curated problems, always present\n  \
                   2. Nightly feed  ~2,400 problems from OSV (default, no flag needed)\n  \
                   3. --online    Live OSV.dev batch query per scanned package\n\n\
                   \x1b[1mLLM pattern generation:\x1b[0m\n  \
                   Set DEP_DOCTOR_LLM_API_KEY to enable --generate-patterns.\n  \
                   Optional: DEP_DOCTOR_LLM_ENDPOINT, DEP_DOCTOR_LLM_MODEL,\n  \
                   DEP_DOCTOR_LLM_RATE_LIMIT_MS (delay between API calls)."
)]
pub struct ScanArgs {
    /// Root folder containing repos to scan
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Deep scan: search source files for affected usage patterns
    #[arg(long, short = 'd')]
    pub deep: bool,

    /// Use LLM to generate deep-scan patterns for problems that lack them.
    /// Requires DEP_DOCTOR_LLM_API_KEY env var. Implies --deep.
    #[arg(long)]
    pub generate_patterns: bool,

    /// Show pattern quality report after scan (hit rates across runs).
    /// Only meaningful with --deep or --generate-patterns.
    #[arg(long)]
    pub pattern_stats: bool,

    /// Only scan a specific ecosystem
    #[arg(long, short = 'e', value_enum)]
    pub ecosystem: Option<EcosystemArg>,

    /// Output format
    #[arg(long, short = 'r', value_enum, default_value = "console")]
    pub reporter: ReporterArg,

    /// Output file path (for json/markdown reporters); omit to print to stdout
    #[arg(long, short = 'o')]
    pub output: Option<PathBuf>,

    /// Query OSV.dev for real-time vulnerability data (adds ~2s per ecosystem)
    #[arg(long)]
    pub online: bool,

    /// Minimum severity to report
    #[arg(long, short = 's', value_enum, default_value = "low")]
    pub severity: SeverityArg,

    /// Hide the summary table at the end
    #[arg(long = "no-summary")]
    pub no_summary: bool,
}

impl ScanArgs {
    /// Whether to show the summary table (default: true, disabled by --no-summary).
    pub fn summary(&self) -> bool {
        !self.no_summary
    }

    /// Whether deep scanning is active (explicit --deep or implied by --generate-patterns).
    pub fn deep_enabled(&self) -> bool {
        self.deep || self.generate_patterns
    }
}

#[derive(Args)]
pub struct ProblemsArgs {
    #[command(subcommand)]
    pub action: ProblemsAction,
}

#[derive(Subcommand)]
pub enum ProblemsAction {
    /// List all known problems (optionally filter by ecosystem)
    List {
        /// Filter to a specific ecosystem
        #[arg(long, short = 'e', value_enum)]
        ecosystem: Option<EcosystemArg>,
    },
    /// Show full details for a specific problem by ID
    Show {
        /// Problem ID (e.g. npm-axios-csrf-ssrf-CVE-2023-45857)
        id: String,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum EcosystemArg {
    /// Node.js / npm packages
    Npm,
    /// Python / pip packages
    Pip,
    /// Go modules
    Go,
    /// Rust / Cargo crates
    Cargo,
}

#[derive(Clone, ValueEnum)]
pub enum ReporterArg {
    /// Colored terminal output (default)
    Console,
    /// Machine-readable JSON
    Json,
    /// Markdown table
    Markdown,
}

#[derive(Clone, ValueEnum, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeverityArg {
    /// Informational only
    Info,
    /// Low severity
    Low,
    /// Medium severity
    Medium,
    /// High severity
    High,
    /// Critical severity
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
