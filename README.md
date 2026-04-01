# dep-doctor 🩺

> Scan a folder of repos for known dependency vulnerabilities — with **real-time CVE lookup** via OSV.dev and optional **deep source-code analysis**.

```
$ dep-doctor scan ./my-projects --online --deep

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
npm install -g dep-doctor
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
# Scan all repos in a folder
dep-doctor scan ./my-projects

# Online mode: query OSV.dev for 50,000+ real-time CVEs
dep-doctor scan ./my-projects --online

# Online + deep scan: find vulnerable code in source files
dep-doctor scan ./my-projects --online --deep

# Filter by ecosystem
dep-doctor scan ./my-projects --ecosystem npm

# Filter by minimum severity
dep-doctor scan ./my-projects --online --severity high

# Output as JSON (good for CI pipelines)
dep-doctor scan ./my-projects --online --reporter json > findings.json

# Output as Markdown report
dep-doctor scan ./my-projects --reporter markdown -o report.md

# List all known built-in problems
dep-doctor problems list

# Show details for a specific problem
dep-doctor problems show npm-axios-csrf-ssrf-CVE-2023-45857
```

---

## Online mode (`--online`)

By default, dep-doctor checks against a small set of built-in vulnerability definitions. With `--online`, it queries [Google's OSV.dev](https://osv.dev) database in real time for every package it finds, giving access to **50,000+ known CVEs** across all supported ecosystems.

**How it works:**
- Sends each unique package name + version to the OSV.dev batch API over HTTPS
- Converts OSV advisories to dep-doctor findings with proper CVSS→severity mapping
- Caches responses to disk for 1 hour (`~/.cache/dep-doctor/osv/`) to avoid repeated API calls
- If the API is unreachable, silently falls back to built-in problems only

**What gets sent:** only package names and versions (e.g. "axios", "0.27.2"). No source code, no credentials, no file paths.

---

## How it works

1. **Repo discovery** — finds every subdirectory containing a manifest file (`package.json`, `requirements.txt`, `go.mod`, `Cargo.toml`)
2. **Manifest parsing** — reads exact installed versions from lock files when available
3. **Version matching** — checks each package against the problems database using semver range matching
4. **OSV lookup** (with `--online`) — queries OSV.dev for additional vulnerabilities, merges with built-in results (built-in definitions take priority on conflicts)
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

## Environment variables

| Variable | Effect |
|----------|--------|
| `DEP_DOCTOR_DEBUG=1` | Enable verbose debug output |

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT
