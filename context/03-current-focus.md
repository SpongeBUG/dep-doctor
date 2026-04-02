# dep-doctor — Current Focus

## v1.0.0 — ✅ SHIPPED

### Feature 1: --fix mode
- `src/fixer/mod.rs` — NEW: `apply_fixes()` orchestrator, `FixResult` type, `print_summary()` colored output
- `src/fixer/npm.rs` — NEW: text-based package.json version replacement, preserves ^/~ prefixes
- `src/fixer/pip.rs` — NEW: requirements.txt + pyproject.toml fixer, ==, ~=, >= operators, hyphen/underscore normalization
- `src/fixer/go.rs` — NEW: go.mod fixer, block + single-line require, preserves // indirect, auto v-prefix
- `src/fixer/cargo.rs` — NEW: Cargo.toml fixer, simple + table form, preserves features/other keys
- `src/cli/args.rs` — Added `--fix` flag
- `src/cli/commands/scan.rs` — Calls fixer after reporting when `--fix` is set
- 25 new unit tests across fixer module

### Feature 2: --watch mode
- `src/watcher/mod.rs` — NEW: `watch_loop()` with notify + debouncer, manifest file filter, 500ms debounce
- `src/cli/args.rs` — Added `--watch` / `-w` flag
- `src/cli/commands/scan.rs` — Refactored `run()` → `run_once()`, watch loop calls `run_once()` on change
- `Cargo.toml` — Added `notify = "7"`, `notify-debouncer-mini = "0.5"`
- 2 new unit tests (manifest path recognition)

### Feature 3: GitHub Action
- `action.yml` — NEW: composite action, inputs (path/severity/online/fix/deep/reporter/version), outputs (findings_count/exit_code), auto-downloads release binary

### Housekeeping
- `Cargo.toml` — Version bump to 1.0.0
- `src/lib.rs` — Added `pub mod fixer;`, `pub mod watcher;`
- 2 new dependencies: notify, notify-debouncer-mini
- All verified: cargo fmt + build + test (58 unit + 7 integration = 65 total) + clippy zero warnings
