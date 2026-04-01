# dep-doctor — Project Overview

Rust CLI that scans repos for known dependency vulnerabilities.

## Tech Stack
- Rust 2021 edition, clap 4 (derive), serde, semver, ureq 2, dirs 5
- Reporters: console (colored), JSON, markdown
- Distribution: GitHub Releases, npm wrapper, pypi wrapper

## Module Map
```
src/
├── cli/              # Clap args + command dispatch
│   ├── args.rs       # ScanArgs, ProblemsArgs, flag enums
│   └── commands/
│       └── scan.rs   # Scan orchestrator (thin — delegates to modules)
├── fetcher/          # OSV online mode (v0.2.0)
│   ├── mod.rs        # query_packages() entry point, dedup, orchestration
│   ├── osv.rs        # OSV.dev batch API types + HTTP call
│   ├── cache.rs      # Disk cache (~/.cache/dep-doctor/osv/), 1h TTL
│   └── converter.rs  # OsvAdvisory → Problem, CVSS → severity
├── scanner/
│   ├── repo_finder.rs
│   ├── manifest/     # npm, pip, go, cargo manifest readers
│   └── version_matcher.rs
├── deep_scan/        # Source-level regex pattern matching
├── problems/
│   ├── schema.rs     # Problem, Finding, SourceHit structs
│   └── registry.rs   # 4 built-in problems
├── reporter/         # console, json, markdown output
└── utils/
    ├── semver_utils.rs  # Range matching, space_to_comma_and (pub)
    ├── logger.rs        # log_debug!, log_warn! macros
    └── fs.rs
```

## Key Types
- `Problem` — a known vuln (id, severity, ecosystem, package, affected_range)
- `Finding<'a>` — a resolved match (repo + package + &Problem + source_hits)
- `InstalledPackage` — parsed from manifests (ecosystem, name, version)

## Scan Flow
1. `repo_finder::find_repos()` — discover repos
2. `manifest::read_all()` — read package.json / requirements.txt / go.mod / Cargo.toml
3. `fetcher::query_packages()` — if `--online`, query OSV for all packages (cached)
4. `version_matcher::match_problems()` — check against built-in + OSV problems
5. `deep_scan::scan_repo()` — if `--deep`, regex source patterns
6. Reporter output
