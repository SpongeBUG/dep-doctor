# dep-doctor — Project Overview

Rust CLI that scans repos for known dependency vulnerabilities.

## Tech Stack
- Rust 2021 edition, clap 4 (derive), serde, semver, ureq 2, dirs 5, zip 2
- Reporters: console (colored), JSON, markdown
- Distribution: GitHub Releases, npm wrapper, pypi wrapper

## Module Map
```
src/
├── bin/
│   └── harvest.rs         # Harvester binary entry point (cargo run --bin harvest)
├── cli/
│   ├── args.rs            # ScanArgs, ProblemsArgs, flag enums (+--pattern-stats v0.6)
│   └── commands/
│       └── scan.rs        # Scan orchestrator — 3-layer merge + LLM enrich + quality stats
├── fetcher/               # OSV live query (--online mode)
│   ├── mod.rs             # query_packages() entry point, dedup, orchestration
│   ├── osv.rs             # OSV.dev batch API types + HTTP + pagination (v0.6)
│   ├── cache.rs           # Disk cache (~/.cache/dep-doctor/osv/), 1h TTL
│   └── converter.rs       # OsvAdvisory → Problem, CVSS → severity
├── feed/                  # Nightly feed consumer (default enrichment)
│   ├── mod.rs             # load_feed() — cache-first, CDN, stale, local dev
│   ├── cache.rs           # ~/.cache/dep-doctor/problems.feed.json, 24h TTL
│   └── fetcher.rs         # GET from GitHub Releases CDN
├── harvest/               # Harvester logic (used by bin/harvest.rs)
│   ├── mod.rs
│   ├── packages.rs        # 349 curated targets (npm/pip/go/cargo)
│   └── runner.rs          # Downloads OSV ecosystem zips, filters, converts
├── llm/                   # LLM-assisted source pattern generation (v0.5.0)
│   ├── mod.rs             # generate_patterns() orchestrator, LlmConfig + rate_limit_ms
│   ├── prompt.rs          # System/user prompt builder, JSON response parser
│   ├── client.rs          # OpenAI-compatible chat completion + 429 retry (v0.6)
│   ├── cache.rs           # Disk cache (~/.cache/dep-doctor/patterns/), no TTL
│   └── quality.rs         # Pattern quality scoring + persistence (v0.6)
├── scanner/
│   ├── repo_finder.rs
│   ├── manifest/          # npm, pip, go, cargo manifest readers
│   └── version_matcher.rs
├── deep_scan/             # Source-level regex pattern matching
├── problems/
│   ├── schema.rs          # Problem, Finding, SourceHit structs
│   └── registry.rs        # 4 built-in problems
├── reporter/              # console, json, markdown output
└── utils/
    ├── semver_utils.rs    # Range matching, space_to_comma_and (pub)
    ├── logger.rs          # log_debug!, log_warn! macros
    └── fs.rs
```

## Key Types
- `Problem` — a known vuln (id, severity, ecosystem, package, affected_range)
- `Finding<'a>` — a resolved match (repo + package + &Problem + source_hits)
- `InstalledPackage` — parsed from manifests (ecosystem, name, version)
- `LlmConfig` — endpoint, api_key, model, rate_limit_ms from env vars
- `PatternStats` — per-problem pattern hit/miss counts across runs (v0.6)

## Scan Flow (v0.6.0)
1. `repo_finder::find_repos()` — discover repos
2. `manifest::read_all()` — read package.json / requirements.txt / go.mod / Cargo.toml
3. Problem loading — 3 layers merged in order (built-in wins on ID conflict):
   - Layer 1: `problems::registry::all_problems()` — 4 built-in, always present
   - Layer 2: `feed::load_feed()` — 2,392 problems from nightly feed (default, no flag)
   - Layer 3: `fetcher::query_packages()` — live OSV lookup with pagination (only with `--online`)
4. **LLM enrichment** (only with `--generate-patterns`):
   - `llm::generate_patterns()` — check cache → rate-limit delay → call LLM (with 429 retry) → validate regex → cache
5. `version_matcher::match_problems()` — check packages against merged problem set
6. `deep_scan::scan_repo()` — if `--deep` or `--generate-patterns`, regex source patterns
   - Records pattern hit/miss in `PatternStats` for quality tracking
7. Reporter output (console / JSON / markdown)
8. **Pattern quality report** (with `--pattern-stats`): hit rates, low-quality flagging

## Environment Variables (v0.6.0)
- `DEP_DOCTOR_DEBUG` — enable debug logging to stderr
- `DEP_DOCTOR_LLM_API_KEY` — required for `--generate-patterns`
- `DEP_DOCTOR_LLM_ENDPOINT` — override default OpenAI endpoint
- `DEP_DOCTOR_LLM_MODEL` — override default `gpt-4o-mini`
- `DEP_DOCTOR_LLM_RATE_LIMIT_MS` — delay between LLM API calls (default: 0)

## Nightly Harvest Flow
1. GitHub Actions runs at 02:00 UTC (`harvest.yml`)
2. `bin/harvest.rs` calls `harvest::runner::run_with_progress()`
3. Downloads `https://storage.googleapis.com/osv-vulnerabilities/<ECO>/all.zip` (4 ecosystems)
4. Unzips in memory, filters to 349 curated package names
5. Converts matching advisories → Problem structs via `fetcher::converter`
6. Writes `problems.feed.json` → uploads to GitHub Release tag `feeds/latest`

## Feed Consumer Resolution Order
1. `~/.cache/dep-doctor/problems.feed.json` — if fresh (< 24h), use immediately
2. GitHub Releases CDN — fetch, save to cache, return
3. Stale cache — use even if expired (graceful degradation)
4. `./problems.feed.json` — local dev fallback (auto-promoted to cache)
5. Empty vec — scan still works with built-in problems only
