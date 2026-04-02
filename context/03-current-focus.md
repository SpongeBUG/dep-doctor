# dep-doctor — Current Focus

## v0.3.0 — ✅ SHIPPED

All three subtasks complete and verified:
- Subtask A: `src/bin/harvest.rs` + `src/harvest/` — downloads OSV zips, produces 2,392 problems
- Subtask B: `src/feed/` — 24h cache, CDN fetch, local dev fallback, integrated into scan
- Subtask C: `.github/workflows/harvest.yml` — nightly 02:00 UTC, publishes to feeds/latest release

## Task 1: v0.4.0 — Supply Chain Attack Detection
**Status:** Not started
**What:** Detect packages with suspicious publish patterns (new maintainer, rapid version bump, install scripts)
**See:** ROADMAP.md → "Supply chain attack detection"
