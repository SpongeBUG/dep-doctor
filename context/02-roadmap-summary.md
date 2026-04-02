# dep-doctor — Roadmap Summary

| Milestone | Status | Key Feature |
|-----------|--------|-------------|
| v0.2.0 | ✅ DONE | OSV online mode + CVSS mapping |
| v0.3.0 | ✅ DONE | Nightly harvest CI + feed consumer |
| v0.4.0 | Next | Supply chain attack detection |
| v0.5.0 | Backlog | LLM-assisted source pattern generation |
| v1.0.0 | Backlog | --fix, watch mode, GitHub Action |

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

## Next Priorities (v0.4.0)
1. **Supply chain attack detection** — flag packages with suspicious publish patterns
2. **Add debug logging to fetcher** — log_debug! for cache hits/misses/API calls
3. **Feed pagination** — handle OSV querybatch next_page_token for packages with >1000 advisories
