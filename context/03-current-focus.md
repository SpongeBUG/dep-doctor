# dep-doctor — Current Focus

## v0.4.0 — ✅ SHIPPED

Supply chain attack detection beyond CVEs:
- Typosquat detector (`src/supply_chain/typosquat.rs`) — Levenshtein ≤2 against popular-500 list
- MALICIOUS ecosystem harvest (`src/harvest/runner.rs`) — unfiltered ingest of OSV MALICIOUS zip
- `ProblemKind` enum (Cve | SupplyChain) with `#[serde(default)]` backward compat
- All 3 reporters updated with [SUPPLY CHAIN] labels + typosquat warning sections

## Task 1: v0.5.0 — LLM-Assisted Source Pattern Generation
**Status:** Not started
**What:** Use LLM to auto-generate deep-scan regex patterns from CVE descriptions
**See:** ROADMAP.md → v0.5.0
