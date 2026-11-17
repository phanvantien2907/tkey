# 50 – Quality Gates

> Before claiming any task is **done**, run all applicable checks below and report the results.

---

## Mandatory Checks

Run these in order and paste the exit codes / relevant output in your response:

### 1. Rust Formatting
```bash
cargo fmt --check
```
Must exit 0. If it fails, run `cargo fmt` and re-check.

### 2. Rust Lints
```bash
cargo clippy --all-targets -- -D warnings
```
Must exit 0. Zero warnings allowed. Fix all warnings — do not use `#[allow(...)]` without justification.

### 3. Rust Tests
```bash
cargo test --workspace
```
Must exit 0. All tests pass, including doctests (`cargo test --doc`).

### 4. TypeScript Type Check
```bash
pnpm tsc --noEmit
```
Must exit 0. No type errors allowed.

### 5. Lint (if configured)
```bash
pnpm lint
```
Run if `eslint` or similar is configured in `package.json`. Must exit 0.

### 6. Tauri Debug Build (conditional)
```bash
pnpm tauri build --debug
```
Run **only** when you have changed:
- `src-tauri/tauri.conf.json`
- `src-tauri/capabilities/*.json`
- Tauri command registrations
- Cargo feature flags

This build is slow — skip it for pure logic or UI changes.

---

## Reporting Format

When done, include a checklist like this in your response:

```
Quality gates:
- [x] cargo fmt --check       → exit 0
- [x] cargo clippy            → exit 0, 0 warnings
- [x] cargo test --workspace  → exit 0, 42 passed
- [x] pnpm tsc --noEmit       → exit 0
- [ ] pnpm lint               → not configured yet
- [N/A] pnpm tauri build      → no Tauri config changed
```

---

## Notes

- If a check fails, **fix it before claiming done** — do not ask the user to run it themselves.
- If a check is genuinely not applicable (e.g. no Rust changes → skip cargo checks), mark it `[N/A]` with a reason.
- Do not add `#[allow(clippy::...)]` or `// @ts-ignore` as a shortcut. Fix the root cause.
