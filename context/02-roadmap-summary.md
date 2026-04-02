# dep-doctor — Roadmap Summary

| Milestone | Status | Key Feature |
|-----------|--------|-------------|
| v0.2.0 | ✅ DONE | OSV online mode + CVSS mapping |
| v0.3.0 | ✅ DONE | Nightly harvest CI + feed consumer |
| v0.4.0 | ✅ DONE | Supply chain attack detection |
| v0.5.0 | ✅ DONE | LLM-assisted source pattern generation |
| v0.6.0 | ✅ DONE | Feed pagination, rate limiting, pattern quality |
| v1.0.0 | ✅ DONE | --fix, watch mode, GitHub Action |

## Completed (v0.6.0)
- OSV querybatch pagination: `next_page_token` loop in `query_batch()`, per-query independent tracking, MAX_PAGES=20 safety cap
- LLM rate limiting: HTTP 429 retry with exponential backoff (2s→4s→8s), `Retry-After` header support, `DEP_DOCTOR_LLM_RATE_LIMIT_MS` inter-request delay
- Pattern quality scoring: per-problem hit/miss stats, disk persistence, `--pattern-stats` flag with low-quality pattern flagging
- Extracted `report_findings()` helper in scan.rs for rustfmt stability
- 9 new tests (31 unit + 7 integration = 38 total), zero new dependencies, clippy clean

## Completed (v0.5.0)
- LLM pattern generator: `--generate-patterns` flag, OpenAI-compatible API, disk cache, regex validation
- Debug logging in fetcher: `log_debug!` for cache hits/misses/API calls
- Zero new dependencies (ureq/serde/regex already present)
- 10 new unit tests (22 total), 7 integration tests, clippy clean

## Completed (v0.4.0)
- Typosquat detector: Levenshtein ≤2 against curated popular-500 package list
- OSV MALICIOUS ecosystem: nightly harvest ingests all confirmed malware advisories
- ProblemKind enum (Cve | SupplyChain) with backward-compatible serde default
- All 3 reporters updated: console shows [SUPPLY CHAIN] label + typosquat section, JSON wraps in {findings, typosquat_warnings}, markdown adds kind column + typosquat table

## Completed (v0.3.0)
- Harvester binary (`cargo run --bin harvest`) downloads OSV ecosystem zips
- Filters 349 curated npm/pip/go/cargo packages → 2,392 problems in problems.feed.json
- GitHub Actions nightly cron (.github/workflows/harvest.yml) publishes feed to releases/feeds/latest
- Feed consumer in scan: cache-first (24h TTL), CDN fallback, local dev fallback
- 3-layer merge: built-in → feed → --online (built-in always wins on ID conflict)
- `default-run = "dep-doctor"` so `cargo run -- scan .` works

## Completed (v0.2.0)
- `--online` flag queries OSV.dev batch API per package
- Disk cache with 1h TTL at ~/.cache/dep-doctor/osv/
- OsvAdvisory → Problem conversion with CVSS→severity mapping
- Merge + dedup (built-in wins on ID conflict)
- SEMVER range extraction with exact-version-list fallback

## Completed (v1.0.0)
- `--fix` mode: auto-update manifests to fixed versions (npm/pip/go/cargo), text-based replacement preserving formatting
- `--watch` / `-w` mode: re-scan on manifest file change via notify + 500ms debounce, Ctrl+C to exit
- GitHub Action: composite action.yml with platform auto-detect, inputs/outputs, release binary download
- Refactored scan.rs: `run()` → `run_once()` for watch reuse
- 27 new tests (58 unit + 7 integration = 65 total), 2 new dependencies (notify, notify-debouncer-mini), clippy clean
