# dep-doctor 🩺

> Scan a folder of repos for known dependency vulnerabilities — with a **nightly-updated CVE feed**, **real-time OSV.dev lookup**, and optional **deep source-code analysis**.

```
$ dep-doctor scan ./my-projects --deep

◆ repo: my-api
  [HIGH    ] npm-axios-csrf-ssrf-CVE-2023-45857  axios @ 0.27.2 → 1.6.0
             Axios CSRF token leak via cross-origin redirect
             ⚑ 2 affected location(s):
               src/server.js line 8 [likely]
                 const result = await axios.get(req.query.url);
                 → Upgrade axios to >=1.6.0 and validate URLs before passing to axios.

◆ repo: legacy-app
  [CRITICAL] npm-lodash-prototype-pollution-CVE-2019-10744  lodash @ 4.17.15 → 4.17.21
             Lodash prototype pollution via defaultsDeep / merge / set
```

---

## Install

### npx (no install required)
```bash
npx dep-doctor scan ./my-projects
```

### npm global
```bash
npm install -g @spongebug/dep-doctor
dep-doctor scan ./my-projects
```

### pip
```bash
pip install dep-doctor
dep-doctor scan ./my-projects
```

### cargo
```bash
cargo install dep-doctor
dep-doctor scan ./my-projects
```

### Binary (GitHub Releases)
Download the latest binary for your platform from the [Releases page](https://github.com/SpongeBUG/dep-doctor/releases).

---

## Usage

```bash
# Scan all repos in a folder (uses nightly feed automatically)
dep-doctor scan ./my-projects

# Online mode: adds live OSV.dev lookup on top of the feed
dep-doctor scan ./my-projects --online

# Online + deep scan: find vulnerable code in source files
dep-doctor scan ./my-projects --online --deep

# Filter by minimum severity
dep-doctor scan ./my-projects --severity high

# Output as JSON (good for CI pipelines)
dep-doctor scan ./my-projects --reporter json > findings.json

# Output as Markdown report
dep-doctor scan ./my-projects --reporter markdown -o report.md

# Auto-fix manifests to safe versions
dep-doctor scan ./my-projects --fix

# Watch mode: re-scan on manifest changes
dep-doctor scan ./my-projects --watch

# List all known built-in problems
dep-doctor problems list

# Show details for a specific problem
dep-doctor problems show npm-axios-csrf-ssrf-CVE-2023-45857
```

---

## How vulnerability data works

dep-doctor uses three layers, merged in priority order (built-in wins on conflict):

### Layer 1 — Built-in problems (always available, offline)
A small curated set of high-impact vulnerabilities compiled into the binary.

### Layer 2 — Nightly feed (default, no flag required)
A `problems.feed.json` published to GitHub Releases every night at 02:00 UTC by a CI harvester. It covers **2,000+ CVEs** across the top 350 npm, pip, Go, and Rust packages — sourced directly from [OSV.dev](https://osv.dev).

- Cached at `~/.cache/dep-doctor/problems.feed.json` (refreshed every 24 hours)
- Falls back to stale cache if the CDN is unreachable
- Falls back to built-in problems only if no cache exists

### Layer 3 — Live OSV.dev lookup (`--online`)
Queries OSV.dev in real time for every package found in your repos — catches CVEs published since the last nightly run. Results are cached for 1 hour at `~/.cache/dep-doctor/osv/`.

**What gets sent:** only package names and versions (e.g. `"axios"`, `"0.27.2"`). No source code, no credentials, no file paths.

---

## How it works

1. **Repo discovery** — finds every subdirectory containing a manifest file (`package.json`, `requirements.txt`, `go.mod`, `Cargo.toml`)
2. **Manifest parsing** — reads exact installed versions from lock files when available
3. **Problem loading** — merges built-in + nightly feed (+ live OSV if `--online`)
4. **Version matching** — checks each package against the merged problem set using semver range matching
5. **Deep scan** (with `--deep`) — walks source files (respecting `.gitignore`) and runs regex patterns to find lines of code that actually use the affected API
6. **Report** — outputs results to console, JSON, or Markdown

---

## Supported ecosystems

| Ecosystem | Manifest files read | OSV ecosystem |
|-----------|-------------------|---------------|
| npm       | `package.json`, `package-lock.json` | npm |
| pip       | `requirements.txt`, `pyproject.toml` | PyPI |
| Go        | `go.mod` | Go |
| Rust      | `Cargo.toml`, `Cargo.lock` | crates.io |

---

## GitHub Action

```yaml
- uses: SpongeBUG/dep-doctor@v1.0.0
  with:
    path: .
    severity: high
```

See [`action.yml`](action.yml) for all inputs and outputs.

---

## Environment variables

| Variable | Effect |
|----------|--------|
| `DEP_DOCTOR_DEBUG=1` | Enable verbose debug output |
| `DEP_DOCTOR_LLM_API_KEY` | API key for `--generate-patterns` |
| `DEP_DOCTOR_LLM_ENDPOINT` | Override LLM endpoint (default: OpenAI) |
| `DEP_DOCTOR_LLM_MODEL` | Override LLM model (default: gpt-4o-mini) |
| `DEP_DOCTOR_LLM_RATE_LIMIT_MS` | Delay between LLM API calls |
| `FEED_OUTPUT_PATH` | Override feed output path (harvester CI) |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
