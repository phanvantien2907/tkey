---
name: add-pinyin-tone-rule
description: >
  Use when adding or fixing a Pinyin tone mark or ü placement rule in crates/core.
  Covers digit-to-tone mapping, vowel selection algorithm (a/e first, ou→o, last vowel),
  ü variants (v / u: input), and lü combinations.
---

# Skill: add-pinyin-tone-rule

## When to Use

Activate this skill when:
- A Pinyin syllable gets the wrong vowel marked
- ü input (`v` or `u:`) is not converting correctly
- Adding a new edge case to the tone placement algorithm
- Updating the reference test table

---

## Tone Mark Placement Reference Table

| Vowel cluster | Rule | Example | Expected |
|---------------|------|---------|----------|
| contains `a` | mark `a` | `hao3` | `hǎo` |
| contains `e` | mark `e` | `bei4` | `bèi` |
| contains `ou` | mark `o` | `dou1` | `dōu` |
| `iu` (no a/e) | mark last → `u` | `liu2` | `liú` |
| `ui` (no a/e) | mark last → `i` | `gui4` | `guì` |
| single vowel | mark it | `ni3` | `nǐ` |
| ü via `v`/`u:` | ü + tone | `lv4` | `lǜ` |

---

## Workflow

### Step 1 – Read domain rules
Read `.agent/rules/40-ime-domain.md` → "Pinyin Tones" section.

### Step 2 – Write failing tests FIRST

```rust
#[cfg(test)]
mod pinyin_tone_tests {
    use super::*;

    #[test]
    fn placement_reference_table() {
        let cases = [
            // (syllable_raw, tone_digit, expected)
            ("hao", 3, "hǎo"),
            ("bei", 4, "bèi"),
            ("dou", 1, "dōu"),
            ("liu", 2, "liú"),
            ("gui", 4, "guì"),
            ("ni",  3, "nǐ"),
            ("lv",  4, "lǜ"),
            ("lv",  2, "lǘ"),
            ("lv",  1, "lǖ"),
            ("lv",  3, "lǚ"),
        ];
        for (syllable, tone, expected) in cases {
            assert_eq!(
                apply_pinyin_tone(syllable, tone).unwrap(),
                expected,
                "syllable={syllable} tone={tone}"
            );
        }
    }
}
```

Run `cargo test --workspace` → confirm **failure**.

### Step 3 – Implement

- Edit only `crates/core`.
- The vowel selection algorithm must follow the priority order in the reference table above.
- For `ü`: normalise `v` and `u:` to `ü` before tone application.

### Step 4 – Run & verify

```bash
cargo test --workspace
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

All exit 0.

### Step 5 – Report

List which cells of the reference table are now covered by tests.

---

## Verification Checklist

- [ ] Failing test written before implementation
- [ ] Reference table rows all covered by test cases
- [ ] `lǖ lǘ lǚ lǜ` all tested
- [ ] `cargo test --workspace` → all pass
- [ ] `cargo fmt --check` and `cargo clippy` → exit 0
- [ ] No change outside `crates/core`
