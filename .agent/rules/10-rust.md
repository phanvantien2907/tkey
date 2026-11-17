# 10 – Rust Standards

## Edition & Tooling

- Use **Rust edition 2024** (matches `src-tauri/Cargo.toml`).
- Always run before committing:
  ```
  cargo fmt
  cargo clippy --all-targets -- -D warnings
  cargo test --workspace
  ```
- Both commands must exit with code 0. Zero warnings tolerated.

---

## Safety

- `crates/core`: **`#![forbid(unsafe_code)]`** at crate root.
- Unsafe code is allowed **only** in the FFI boundary crate.
- Every `unsafe` block must have a `// SAFETY: <reason>` comment explaining why it is sound.

---

## Error Handling

- **No `unwrap()`, `expect()`, or index panics in non-test code.**
  - The IME runs inside the host app — a panic kills the host app.
  - Return `Result<T, E>` or `Option<T>` instead.
- Library errors: use **`thiserror`** to derive typed error enums.
- Do **not** use `anyhow` in `crates/core` (library crate). `anyhow` may be used in binary crates if needed.
- Propagate errors to callers; do not silently swallow them.

```rust
// ✅ DO
pub fn apply_tone(syllable: &str, tone: Tone) -> Result<String, CoreError> { ... }

// ❌ DON'T
pub fn apply_tone(syllable: &str, tone: Tone) -> String {
    some_map[syllable].unwrap() // panics in host app!
}
```

---

## Hot Path Rules (per-keystroke code)

- **Avoid heap allocation** where practical (prefer stack buffers, `SmallVec`, or pre-allocated structures).
- **No blocking I/O** (no file reads, no locks that can block indefinitely).
- **No logging at `info` level or above** in the keystroke handler. Use `trace`/`debug` only, behind a compile-time feature flag if possible.
- Keep the hot path as a chain of small pure functions — easy to benchmark and test.

---

## Code Style

- Prefer **small pure functions** over large mutable state blobs.
- Model state machines with **enums** — exhaustive match arms, no default fallthrough.
- Write **property-based** and **table-driven tests** for engine logic:
  ```rust
  #[test]
  fn telex_tone_placement() {
      let cases = [
          ("toan", 2, "toàn"),
          ("hoang", 5, "hoàng"),
          // ...
      ];
      for (input, tone, expected) in cases {
          assert_eq!(apply_tone(input, tone).unwrap(), expected);
      }
  }
  ```

---

## Dependencies

- **Minimal**: justify every new crate with: (1) what it solves, (2) crate size/compile time impact, (3) maintenance status.
- Prefer crates with no `unsafe` themselves, or well-audited ones.
- Never add a crate just for convenience if the stdlib suffices.

---

## Public API Documentation

- Every public item must have a `///` doc comment.
- Include at least one `# Examples` block that runs as a doctest.
- Run `cargo test --doc` as part of CI.
