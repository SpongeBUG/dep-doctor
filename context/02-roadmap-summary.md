# dep-doctor — Roadmap Summary

| Milestone | Status | Key Feature |
|-----------|--------|-------------|
| v0.2.0 | ✅ DONE | OSV online mode + CVSS mapping |
| v0.3.0 | Next | Nightly harvest CI + feed consumer |
| v0.4.0 | Backlog | Supply chain attack detection |
| v0.5.0 | Backlog | LLM-assisted source pattern generation |
| v1.0.0 | Backlog | --fix, watch mode, GitHub Action |

## Completed (v0.2.0)
- `--online` flag queries OSV.dev batch API per package
- Disk cache with 1h TTL at ~/.cache/dep-doctor/osv/
- OsvAdvisory → Problem conversion with CVSS→severity mapping
- Merge + dedup (built-in wins on ID conflict)
- SEMVER range extraction with exact-version-list fallback

## Next Priorities
1. **Nightly harvest CI** — auto-publish problems.feed.json to GitHub Releases
2. **Feed consumer** — fetch + cache feed, refresh daily, offline fallback
3. **Add debug logging to fetcher** — log_debug! for cache hits/misses/API calls
