# dep-doctor Roadmap

## Current State (v0.3.0)
- ✅ Manifest scanner: npm, pip, go, cargo
- ✅ Deep source scan with regex patterns
- ✅ 4 built-in problems: axios CSRF, lodash prototype, requests leak, rustls MitM
- ✅ Reporters: console (colored), json, markdown
- ✅ GitHub Actions CI + multi-platform release builds
- ✅ npm + pypi distribution wrappers
- ✅ `--online` flag: real-time OSV.dev lookup with 1h disk cache
- ✅ Nightly harvest CI: downloads OSV ecosystem zips, publishes problems.feed.json
- ✅ Feed consumer: 3-layer merge (built-in → feed → --online), 24h cache, offline fallback

---

## The Core Problem with Manual Contribution

Current pipeline: `someone finds CVE` → `opens PR` → `review` → `merge` → `users get it on next release`

That's **weeks of lag**. Supply chain attacks like XZ backdoor or Polyfill.io hijack need to
be in the database **within hours**, not weeks. Manual contribution doesn't scale.

---

## Architecture: Automated Problem Intelligence

```
┌─────────────────────────────────────────────────────┐
│                    dep-doctor                        │
│                                                      │
│  ┌─────────────────┐    ┌──────────────────────┐    │
│  │  Built-in DB    │    │   Nightly Feed       │    │
│  │  (compiled in)  │    │   (24h disk cache)   │    │
│  │  ~10 problems   │    │   2,000+ problems    │    │
│  └─────────────────┘    └──────────────────────┘    │
│           │                       │                  │
│           └──────────┬────────────┘                  │
│                      ▼                               │
│              Problem Registry                        │
│                      ▼                               │
│              Version Matcher                         │
│                      ▼                               │
│              Deep Scanner                            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│        Auto-Harvester (nightly CI job)               │
│                                                      │
│  Source:                                             │
│  └── OSV.dev GCS bucket (ecosystem zip per night)   │
│      covers npm / PyPI / Go / crates.io              │
│                                                      │
│  Output: problems.feed.json → GitHub Releases CDN   │
│          tag: feeds/latest                           │
└─────────────────────────────────────────────────────┘
```

---

## Task Backlog

### ✅ DONE — OSV Online Mode (v0.2.0)
**What:** `--online` flag queries OSV.dev in real-time for every scanned package.
**Files:**
```
src/fetcher/mod.rs        # query_packages() entry point
src/fetcher/osv.rs        # OSV.dev batch API client
src/fetcher/cache.rs      # disk cache (TTL: 1h)
src/fetcher/converter.rs  # OSV Advisory → Problem struct
```

---

### ✅ DONE — Nightly Harvest CI + Feed Consumer (v0.3.0)
**What:** Nightly CI harvests OSV ecosystem zips, publishes feed to GitHub Releases.
Feed is fetched and cached by the CLI automatically on every scan.
**Files:**
```
src/bin/harvest.rs             # harvester binary
src/harvest/packages.rs        # 349 curated target packages
src/harvest/runner.rs          # zip download → filter → convert
src/feed/mod.rs                # load_feed() — 3-layer resolution
src/feed/cache.rs              # ~/.cache/dep-doctor/problems.feed.json (24h TTL)
src/feed/fetcher.rs            # GET from GitHub Releases CDN
.github/workflows/harvest.yml  # nightly cron at 02:00 UTC
```
**Result:** 2,392 problems from 349 packages across npm/pip/go/cargo.

---

### ✅ DONE — CVSS → Severity Mapping (v0.2.0)
```
CVSS 9.0–10.0 → critical
CVSS 7.0–8.9  → high
CVSS 4.0–6.9  → medium
CVSS 0.1–3.9  → low
```

---

### 🟠 P2 — Supply Chain Attack Detection (v0.4.0)
**What:** Beyond CVEs — detect typosquatting, dependency confusion,
malicious maintainer takeovers, intentional sabotage.
**Sources:** Socket.dev API, OSV `ecosystem:MALICIOUS`, OpenSSF Scorecard API
**Examples this would catch:**
- `coa` (npm) — maintainer account hijacked, malware injected
- `colors`/`faker` — intentional sabotage by author
- `node-ipc` — political malware injection
- Packages 1 character off from popular packages (typosquatting)

**New problem kind:**
```rust
pub enum ProblemKind {
    Cve,           // existing
    SupplyChain,   // NEW
    BreakingChange,
}
```

---

### 🟠 P2 — LLM-Assisted Source Pattern Generation (v0.5.0)
**What:** When OSV returns a new advisory, automatically generate
regex source patterns by sending the advisory to an LLM.
**Why:** OSV gives us version ranges but NOT source-level patterns.
Auto-generating patterns means deep-scan works for ALL CVEs,
not just the handful we hand-wrote.

---

### 🟢 P3 — `--fix` Flag
**What:** Auto-bump vulnerable versions in manifests to the fixed version.
```bash
dep-doctor scan ./my-projects --fix
# Rewrites package.json: "axios": "0.27.2" → "axios": "^1.6.0"
```

---

### 🟢 P3 — Watch Mode
**What:** `dep-doctor watch ./my-projects` — monitors manifests for changes
and re-scans automatically.

---

### 🟢 P3 — GitHub Action
**What:** `SpongeBUG/dep-doctor-action` — lets any repo run dep-doctor in CI.
```yaml
- uses: SpongeBUG/dep-doctor-action@v1
  with:
    path: .
    severity: high
    deep: true
    fail-on-findings: true
```

---

## Milestone Plan

| Milestone | Status | Tasks | Goal |
|-----------|--------|-------|------|
| v0.2.0 | ✅ Done | OSV online mode + CVSS mapping | Real-time CVE lookup |
| v0.3.0 | ✅ Done | Nightly harvest + feed consumer | Fully automated DB |
| v0.4.0 | Next | Supply chain detection | Beyond CVEs |
| v0.5.0 | Backlog | LLM pattern generation | Deep scan for all CVEs |
| v1.0.0 | Backlog | --fix + watch + GitHub Action | Production-ready |
