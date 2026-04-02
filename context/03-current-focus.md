# dep-doctor — Current Focus

## v0.5.0 — ✅ SHIPPED

LLM-assisted source pattern generation + fetcher observability:
- `src/llm/mod.rs` — Orchestrator: `generate_patterns()`, `LlmConfig` from env vars
- `src/llm/cache.rs` — Disk cache at `~/.cache/dep-doctor/patterns/`, no TTL (CVEs immutable)
- `src/llm/prompt.rs` — System/user prompt builder, JSON response parser, regex validation
- `src/llm/client.rs` — Thin ureq wrapper for OpenAI-compatible chat completion
- `src/cli/args.rs` — `--generate-patterns` flag (implies `--deep`), `deep_enabled()` method
- `src/cli/commands/scan.rs` — `enrich_patterns()` pass with progress bar
- `src/fetcher/mod.rs` — `log_debug!` for cache hits/misses/API calls/totals
- Config: `DEP_DOCTOR_LLM_API_KEY` (required), `DEP_DOCTOR_LLM_ENDPOINT`, `DEP_DOCTOR_LLM_MODEL`
- All verified: cargo fmt + build + test (22 unit + 7 integration) + clippy zero warnings

## Task 1: v0.6.0 — Feed Pagination + Polish
**Status:** Not started
**What:** Handle OSV querybatch `next_page_token` for packages with >1000 advisories
**See:** ROADMAP.md → v0.6.0
