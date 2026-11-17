# 40 – IME Domain Rules

> This is the most important domain-specific rule file. Read it fully before touching any typing logic.
> All typing logic lives in `crates/core`. See `00-project-overview.md`.

---

## Vietnamese: Telex & VNI

### Tone Marks

| Telex key | VNI key | Tone | Mark |
|-----------|---------|------|------|
| `s`       | `1`     | sắc  | ´    |
| `f`       | `2`     | huyền| `    |
| `r`       | `3`     | hỏi  | ̉    |
| `x`       | `4`     | ngã  | ̃    |
| `j`       | `5`     | nặng | .    |
| (none)    | `0`     | flat |      |

### Vowel Modifications (Telex)

| Input | Result |
|-------|--------|
| `aa`  | `â`    |
| `aw`  | `ă`    |
| `ee`  | `ê`    |
| `oo`  | `ô`    |
| `ow` / `w` after `o` | `ơ` |
| `uw` / `w` after `u` | `ư` |
| `dd`  | `đ`    |
| `w` standalone / before vowel | `ư` or `ươ` |

### Undo (Double Key)

- Typing the modifier key a second time undoes the modification:
  - `aas` → `âs` → (second `a`) → `aas` (literal)
  - `aaa` → `âa` → `aa` (literal `aa`)
- This allows typing the actual ASCII pair when needed.

### Complex Vowel Clusters

- Handle `qu` (q is always followed by u, treat `qu` as a unit).
- Handle `gi` (g + i forms the initial `gi`, tone goes on the vowel after).
- Handle compound rimes: `ươ`, `uô`, `oa`, `oe`, `uy`, `uê`, `iê`, `yê`.
- Tone placement in compound rimes follows standard modern Vietnamese orthography.

### Tone Placement (configurable)

- Default: **modern standard** (e.g. `hoà` not `hòa` — tone on the main vowel).
- Expose as a config option in `crates/core` schema: `tone_placement: "modern" | "traditional"`.
- Document the default in the schema.

---

## Pinyin Tones

### Tone Input

| Digit | Tone | Mark |
|-------|------|------|
| `1`   | flat  | ā    |
| `2`   | rising | á   |
| `3`   | falling-rising | ǎ |
| `4`   | falling | à  |
| `5` or `0` | neutral/no mark | a |

### ü Variants

- `v` or `u:` → `ü`
- Combinations: `lǖ lǘ lǚ lǜ` (ü + tones 1–4)

### Tone Mark Placement Rules

Apply in this order:

1. If the syllable contains `a` or `e`, mark that vowel.
2. If the syllable contains `ou`, mark `o`.
3. Otherwise, mark the **last vowel** in the vowel cluster:
   - `iu` → mark `u` → `iú`
   - `ui` → mark `i` → `uí`

### Examples

| Input  | Expected |
|--------|----------|
| `ni3`  | `nǐ`     |
| `hao3` | `hǎo`    |
| `lv4`  | `lǜ`     |
| `gui4` | `guì`    |
| `liu2` | `liú`    |

---

## Shortcuts

- User-defined map: `trigger → output` (e.g. `nh` → `níhǎo`).
- Stored in the config file managed by `crates/core`.
- **Expansion rules**:
  - Expand **only** when the commit key (default: Space) is pressed after an **exact whole-word match**.
  - Never expand mid-word (e.g. `nha` must not trigger `nh`).
  - **Escape mechanism**: if the user types the trigger and then the commit key a second time, output the literal trigger text.
- Shortcut processing runs **after** IME composition is committed.

---

## Composition Model

- **One word at a time**: the preedit buffer holds exactly one word being composed.
- **Commit triggers**: Space, punctuation, Enter → commit the current composition.
- **Cancel**: Esc → discard the preedit buffer, output nothing.
- **No candidate window** for Chinese characters unless explicitly requested in future.
- Use the OS's official preedit/marked-text API (IMK on macOS, TSF on Windows) — never fake-backspace.

---

## Test-First Policy

> Every behavior change or new rule requires a **failing test in `crates/core` first**.
>
> 1. Write the table-driven test cases.
> 2. Run `cargo test --workspace` → confirm it fails.
> 3. Implement the rule.
> 4. Run `cargo test --workspace` → confirm it passes.
> 5. Add edge cases: empty input, single char, max-length buffer, unicode boundaries.
