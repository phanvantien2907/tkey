---
name: ime-manual-test-matrix
description: >
  Use when running the cross-app manual test checklist before a release or after
  significant changes to the core engine or platform shell. Covers Safari, Chrome,
  WPS Office, Gmail compose, Facebook, Notes, and Terminal.
---

# Skill: ime-manual-test-matrix

## When to Use

Activate this skill when:
- Preparing for a release or beta
- After merging a significant change to `crates/core` or the macOS shell
- Verifying a bug fix across multiple host applications

---

## Test Environment Setup

1. Build and install the latest IME bundle (follow `macos-imk-shell` skill).
2. Enable TKey in System Settings → Keyboard → Input Sources.
3. Select TKey as the active input method.

---

## Test Cases

For each scenario below, record **Pass / Fail / Notes** in the results table.

### A – Telex: basic word
- Input: `toi5` → Expected: `tôi`
- Input: `hoaf` → Expected: `hòa` (or `hoà` depending on `tone_placement` config)

### B – Telex: compound vowels
- Input: `uong6` (VNI) or `uongj` (Telex) → Expected: `uộng`
- Input: `dduwowngf` (Telex) → Expected: `đường`

### C – Telex: undo (double key)
- Input: `aaa` → Expected: `aa` (literal, not `âa`)
- Input: `dds` → Expected: `đs` → then `ddds` → `dds` (literal)

### D – Pinyin: tone marks
- Input: `ni3hao3` → Expected: `nǐhǎo`
- Input: `lv4` → Expected: `lǜ`
- Input: `gui4` → Expected: `guì`

### E – Shortcut expansion
- Configure shortcut: `nh` → `níhǎo`
- Type `nh` then Space → Expected: `níhǎo ` (expanded)
- Type `nha` then Space → Expected: `nha ` (not expanded — mid-word guard)
- Type `nh` then `nh` (double commit) → Expected: `nh ` (literal escape)

### F – Backspace mid-word
- Type `hoa`, press Backspace → Expected: preedit shows `ho`
- Continue: type `ng5` (VNI) → Expected: `hóng`

### G – Escape (cancel composition)
- Type `hoaf`, press Esc → Expected: preedit cleared, nothing inserted

### H – Commit on punctuation
- Type `toi5,` → Expected: `tôi,` (comma commits the word)

---

## Host Application Matrix

| Test | Safari | Chrome | WPS Office | Gmail (web) | Facebook (web) | Notes.app | Terminal |
|------|--------|--------|------------|-------------|----------------|-----------|----------|
| A – basic word | | | | | | | |
| B – compound vowel | | | | | | | |
| C – undo | | | | | | | |
| D – Pinyin | | | | | | | |
| E – shortcut | | | | | | | |
| F – backspace | | | | | | | |
| G – escape | | | | | | | |
| H – punctuation commit | | | | | | | |
| Secure input (password) | crash? | crash? | n/a | n/a | n/a | n/a | n/a |

Fill each cell: **P** (pass), **F** (fail – describe), **N/A**.

---

## Special Notes

- **Terminal**: most terminal emulators do not support preedit/marked text. TKey must fall back to direct character insertion (commit on each keypress without composition).
- **WPS Office**: has known issues with some IMK signals — verify no duplicate characters.
- **Facebook / Gmail (Electron/Chromium)**: preedit may not display; direct insert must still work.
- **Password fields**: IMK deactivates during secure input — verify no crash and no leaked characters.

---

## Reporting Format

After running the matrix, report:

```
IME Manual Test – vX.Y.Z (YYYY-MM-DD)
Build: <git commit hash>
OS: macOS <version>

Results:
[paste the filled table]

Failures:
- Chrome / Test B: đường shows "d??ng" in address bar — preedit issue
  → Filed as: #<issue number>

Blockers for release: <none | list>
```

---

## Verification Checklist

- [ ] All 8 host applications tested (or marked N/A with reason)
- [ ] All test cases A–H covered
- [ ] No crash in any app (including password fields)
- [ ] Results table filled and committed to `docs/test-reports/`
- [ ] Any failures filed as issues
