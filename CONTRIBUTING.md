# Contributing to dep-doctor

Thank you for helping make dep-doctor better!

## Ways to contribute

- **Add a new problem definition** — the highest-value contribution
- **Improve an existing definition** — better regex patterns, more sources
- **Fix a bug** — open an issue first for anything non-trivial
- **Improve docs**

---

## Adding a problem definition

### Option A — Community TOML (easiest, no Rust required)

1. Fork the repo
2. Create `problems.d/YOUR-PROBLEM-ID.toml` following the schema in [`problems.d/README.md`](problems.d/README.md)
3. Add a test fixture under `tests/fixtures/` (a minimal fake repo that triggers the problem)
4. Open a PR — CI validates automatically

### Option B — Built-in Rust definition

1. Copy `src/problems/builtin/npm_axios_csrf.rs` as a template
2. Create `src/problems/builtin/YOUR_PROBLEM.rs`
3. Register it in `src/problems/builtin/mod.rs` and `src/problems/registry.rs`
4. Add test fixtures and run `cargo test`

---

## Development setup

```bash
# Requires Rust (https://rustup.rs)
git clone https://github.com/SpongeBUG/dep-doctor
cd dep-doctor
cargo build
cargo test

# Run against a real folder
cargo run -- scan ./tests/fixtures --deep
```

---

## Code standards

- No file over 300 lines
- No function over 30 lines  
- Run `cargo fmt` before committing
- Run `cargo clippy -- -D warnings` — zero warnings policy
- All new problem definitions must include at least one test fixture

## PR checklist

- [ ] `cargo fmt --check` passes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] New problem has a test fixture
- [ ] Problem ID format: `{ecosystem}-{package}-{type}-{CVE-or-short-desc}`
