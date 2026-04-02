# dep-doctor — Roadmap Summary

| Milestone | Status | Key Feature |
|-----------|--------|-------------|
| v0.2.0 | ✅ DONE | OSV online mode + CVSS mapping |
| v0.3.0 | ✅ DONE | Nightly harvest CI + feed consumer |
| v0.4.0 | ✅ DONE | Supply chain attack detection |
| v0.5.0 | ✅ DONE | LLM-assisted source pattern generation |
| v0.6.0 | Next | Feed pagination, polish |
| v1.0.0 | Backlog | --fix, watch mode, GitHub Action |

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

## Next Priorities (v0.6.0)
1. **Feed pagination** — handle OSV querybatch next_page_token for packages with >1000 advisories
2. **Rate limiting** — respect LLM API rate limits during pattern generation
3. **Pattern quality scoring** — track LLM pattern hit rates to identify low-quality patterns
