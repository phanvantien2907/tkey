---
name: add-telex-rule
description: >
  Use when adding or fixing a Vietnamese Telex or VNI typing rule in crates/core.
  Covers tone placement, vowel modification, compound rimes, undo behaviour,
  and any change to how keystrokes are transformed into Vietnamese characters.
---

# Skill: add-telex-rule

## When to Use

Activate this skill when:
- Adding a new Telex or VNI transformation (e.g. a new rime, edge case, or undo behaviour)
- Fixing incorrect tone placement for specific vowel clusters
- Changing or documenting tone placement config (`modern` vs `traditional`)

---

## Workflow

### Step 1 – Read the domain rules
Read `.agent/rules/40-ime-domain.md` in full. Identify which rule is being added or fixed.

### Step 2 – Write failing tests FIRST

In `crates/core/src/` (or the appropriate test file), add a table-driven test:

```rust
#[cfg(test)]
mod telex_tests {
    use super::*;

    #[test]
    fn compound_rime_uong_tone() {
        let cases = [
            // (raw_input, tone_digit, expected_output)
            ("uong", 2, "uồng"),
            ("uong", 3, "uổng"),
            ("uong", 5, "uộng"),
        ];
        for (input, tone, expected) in cases {
            assert_eq!(
                apply_telex_tone(input, tone).unwrap(),
                expected,
                "failed for input={input} tone={tone}"
            );
        }
    }
}
```

Run `cargo test --workspace` → confirm the new test **fails**.

### Step 3 – Implement the rule

- Edit only `crates/core` — no changes to `src-tauri` or `src/`.
- Keep the implementation as a small pure function.
- If the rule is configurable, add a field to the core config schema.

### Step 4 – Run tests

```bash
cargo test --workspace
```

All tests must pass, including pre-existing ones.

### Step 5 – Run quality gates

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

Both must exit 0.

### Step 6 – Report

List the edge cases the new tests cover:

```
Edge cases tested:
- Empty input → Ok("")
- Single vowel `a` + tone 1 → `á`
- Compound rime `uong` + tones 2/3/5 → `uồng`/`uổng`/`uộng`
- Undo: `aas` → `âs` → `aas`
```

---

## Verification Checklist

- [ ] Failing test written before implementation
- [ ] `cargo test --workspace` → all pass
- [ ] `cargo fmt --check` → exit 0
- [ ] `cargo clippy --all-targets -- -D warnings` → exit 0
- [ ] Edge cases documented in test names or comments
- [ ] No change to `src-tauri` or `src/`
