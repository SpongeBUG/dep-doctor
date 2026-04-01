# dep-doctor 🩺

> Scan a folder of repos for known dependency problems and **deep-dive into source code** to find exactly where you're affected.

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
Download the latest binary for your platform from the [Releases page](https://github.com/YOUR_USERNAME/dep-doctor/releases).

---

## Usage

```bash
# Scan all repos in a folder (manifest check only)
dep-doctor scan ./my-projects

# Deep scan: also search source files for affected usage
dep-doctor scan ./my-projects --deep

# Filter by ecosystem
dep-doctor scan ./my-projects --ecosystem npm

# Filter by minimum severity
dep-doctor scan ./my-projects --severity high

# Output as JSON (good for CI pipelines)
dep-doctor scan ./my-projects --reporter json > findings.json

# Output as Markdown report
dep-doctor scan ./my-projects --reporter markdown -o report.md

# List all known problems
dep-doctor problems list

# Show details for a specific problem
dep-doctor problems show npm-axios-csrf-ssrf-CVE-2023-45857
```

---

## How it works

1. **Repo discovery** — finds every subdirectory containing a manifest file (`package.json`, `requirements.txt`, `go.mod`, `Cargo.toml`)
2. **Manifest parsing** — reads exact installed versions from lock files when available
3. **Version matching** — checks each package against the problems database using semver range matching
4. **Deep scan** (optional `--deep`) — walks source files (respecting `.gitignore`) and runs regex patterns to find lines of code that actually use the affected API
5. **Report** — outputs results to console, JSON, or Markdown

---

## Supported ecosystems

| Ecosystem | Manifest files read |
|-----------|-------------------|
| npm       | `package.json`, `package-lock.json` |
| pip       | `requirements.txt`, `pyproject.toml` |
| Go        | `go.mod` |
| Rust      | `Cargo.toml`, `Cargo.lock` |

---

## Adding a new problem definition

Built-in definitions live in `src/problems/builtin/`. Community definitions live in `problems.d/` as TOML files.

See [`problems.d/README.md`](problems.d/README.md) for the full schema and contribution guide.

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
