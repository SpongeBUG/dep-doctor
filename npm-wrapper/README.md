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

## Install

```bash
npm install -g @spongebug/dep-doctor
```

Or download binaries from [GitHub Releases](https://github.com/SpongeBUG/dep-doctor/releases).

## Usage

```bash
dep-doctor scan ./my-projects              # Scan with nightly feed
dep-doctor scan . --online --deep          # Live OSV + source analysis
dep-doctor scan . --fix                    # Auto-fix manifests
dep-doctor scan . --watch                  # Re-scan on file changes
dep-doctor scan . -r json -o report.json   # JSON output
dep-doctor scan . --severity high          # Filter by severity
dep-doctor problems list                   # List known problems
```

## Features

- **3-layer vulnerability data**: built-in → nightly OSV feed → live OSV.dev lookup
- **4 ecosystems**: npm, pip, Go, Rust
- **Deep scan**: regex source patterns find vulnerable API usage in code
- **Auto-fix**: `--fix` updates manifests to safe versions
- **Watch mode**: `--watch` re-scans on manifest changes
- **LLM patterns**: `--generate-patterns` uses AI to create source detection rules
- **GitHub Action**: use `SpongeBUG/dep-doctor@v1.0.0` in CI

Full documentation: https://github.com/SpongeBUG/dep-doctor
