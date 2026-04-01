# dep-doctor Roadmap

## Current State (v0.1.2)
- ✅ Manifest scanner: npm, pip, go, cargo
- ✅ Deep source scan with regex patterns
- ✅ 4 built-in problems: axios CSRF, lodash prototype, requests leak, rustls MitM
- ✅ Reporters: console (colored), json, markdown
- ✅ GitHub Actions CI + multi-platform release builds
- ✅ npm + pypi distribution wrappers

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
│  │  Built-in DB    │    │   Remote Feed        │    │
│  │  (compiled in)  │    │   (fetched at scan)  │    │
│  │  ~10 problems   │    │   100s of problems   │    │
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
│  Sources:                                            │
│  ├── OSV.dev API (Google — covers npm/pip/go/cargo)  │
│  ├── GitHub Advisory DB API                          │
│  ├── NVD (NIST) API                                  │
│  ├── RustSec Advisory DB (git)                       │
│  └── Socket.dev API (supply chain attacks)           │
│                                                      │
│  Output: problems.feed.json → GitHub Releases CDN   │
└─────────────────────────────────────────────────────┘
```

### Key insight: OSV.dev does the heavy lifting

Google's OSV.dev aggregates CVEs from GitHub Advisory, NVD, RustSec, PyPA, and npm
advisories into a single unified API — free, no auth required.

```
POST https://api.osv.dev/v1/query
{ "package": { "name": "axios", "ecosystem": "npm" }, "version": "0.27.2" }
→ Returns all known vulns for that exact version instantly
```

---

## Task Backlog

### 🔴 P0 — OSV Online Mode
**What:** `--online` flag queries OSV.dev in real-time for every scanned package.
**Impact:** Instant access to ALL CVEs without waiting for a DB update.
**Files:**
```
src/fetcher/mod.rs        # fetch trait + dispatcher
src/fetcher/osv.rs        # OSV.dev API client
src/fetcher/cache.rs      # disk cache (TTL: 1h) to avoid hammering API
src/fetcher/converter.rs  # OSV Advisory → Problem struct
```
**Usage:**
```bash
dep-doctor scan ./my-projects --online --deep
```
**OSV → Problem field mapping:**
```
id                  → id
summary             → title
severity[].score    → severity (CVSS score → critical/high/medium/low)
affected[].ranges   → affected_range (semver)
references          → references
```

---

### 🔴 P0 — Nightly Harvest CI Job
**What:** GitHub Actions runs nightly at 02:00 UTC, harvests OSV for all popular
packages, publishes `problems.feed.json` to GitHub Releases.
**Impact:** Zero-lag problem updates. No human needed to add new CVEs.
**Files:**
```
src/bin/harvest.rs           # standalone harvester binary
.github/workflows/harvest.yml # nightly scheduled job
```
**Flow:**
```
1. Query OSV for top 500 npm + pip + go + cargo packages
2. Convert advisories → Problem structs
3. Write problems.feed.json
4. Upload to GitHub Release tag "feeds/latest"
5. dep-doctor fetches on first scan of the day
```

---

### 🟡 P1 — Feed Consumer
**What:** dep-doctor fetches and caches `problems.feed.json` from the CDN.
Refreshes once per day. Falls back to built-in DB if offline.
**Cache location:** `~/.cache/dep-doctor/problems.feed.json`

---

### 🟡 P1 — CVSS → Severity Mapping
**What:** Proper conversion of CVSS scores to our severity enum.
```
CVSS 9.0–10.0 → critical
CVSS 7.0–8.9  → high
CVSS 4.0–6.9  → medium
CVSS 0.1–3.9  → low
```

---

### 🟠 P2 — Supply Chain Attack Detection
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

### 🟠 P2 — LLM-Assisted Source Pattern Generation
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

| Milestone | Tasks | Goal |
|-----------|-------|------|
| v0.2.0 | OSV online mode + CVSS mapping | Real-time CVE lookup |
| v0.3.0 | Nightly harvest + feed consumer | Fully automated DB |
| v0.4.0 | Supply chain detection | Beyond CVEs |
| v0.5.0 | LLM pattern generation | Deep scan for all CVEs |
| v1.0.0 | --fix + watch + GitHub Action | Production-ready |
