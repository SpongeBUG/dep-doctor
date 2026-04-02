# dep-doctor — Current Focus

## v0.6.0 — ✅ SHIPPED

Feed pagination + rate limiting + pattern quality scoring:

### Feature 1: OSV Feed Pagination
- `src/fetcher/osv.rs` — `Query.page_token`, `QueryResult.next_page_token`, pagination loop in `query_batch()` with `send_batch()` helper, MAX_PAGES=20 safety cap
- `src/fetcher/mod.rs` — Updated Query construction with `page_token: None`
- 4 new unit tests: token serialization, response parsing, empty input

### Feature 2: LLM Rate Limiting
- `src/llm/client.rs` — Retry-on-429 with exponential backoff (2s→4s→8s), respects `Retry-After` header
- `src/llm/mod.rs` — `DEP_DOCTOR_LLM_RATE_LIMIT_MS` env var for inter-request delay, `LlmConfig.rate_limit_ms` field
- 1 new unit test: backoff calculation

### Feature 3: Pattern Quality Scoring
- `src/llm/quality.rs` — NEW: `PatternStats`/`ProblemPatternStats` with hit/miss tracking, disk persistence at `~/.cache/dep-doctor/pattern-stats.json`, `print_report()` with low-quality flagging
- `src/cli/commands/scan.rs` — Quality stats loaded/saved around scan, recorded per-finding in `apply_deep_scan()`, extracted `report_findings()` helper
- `src/cli/args.rs` — `--pattern-stats` flag
- 4 new unit tests: record tracking, hit rate math, serialization roundtrip

### Housekeeping
- `Cargo.toml` — Version bump to 0.6.0
- Zero new dependencies
- All verified: cargo fmt + build + test (31 unit + 7 integration = 38 total) + clippy zero warnings

## Task 1: v1.0.0 — --fix, watch mode, GitHub Action
**Status:** Not started
**See:** ROADMAP.md → v1.0.0
