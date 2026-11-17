//! Tone mark application for Pinyin syllables.
//!
//! # Placement rules (priority order)
//! 1. If the syllable contains `a` or `e` → mark that vowel.
//! 2. If the syllable contains `ou` → mark `o`.
//! 3. Otherwise → mark the last vowel in the vowel cluster.
//!
//! All output is NFC-precomposed where a precomposed form exists.

use crate::unicode::{COMBINING_GRAVE, combining_mark, match_case, precomposed, to_nfc};

/// Tone number: 1=ā, 2=á, 3=ǎ, 4=à, 5/0=neutral.
pub(crate) type Tone = u8;

/// Normalize raw user input for a single syllable:
/// - `v` → `ü`
/// - `u:` handled at the engine level before reaching here
/// - Lowercase for lookup; case restored afterwards.
///
/// Returns the normalized lowercase syllable and whether the original had
/// any uppercase letters.
pub(crate) fn normalize_to_lookup(raw: &str) -> (String, Vec<bool>) {
    let mut cases: Vec<bool> = Vec::new();
    let mut lowered: Vec<char> = Vec::new();
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == ':' && i > 0 && (chars[i - 1] == 'u' || chars[i - 1] == 'U') {
            if let Some(last) = lowered.pop() {
                if last == 'u' {
                    lowered.push('ü');
                } else {
                    lowered.push(last);
                }
            }
            i += 1;
            continue;
        }
        let lc = c.to_lowercase().next().unwrap_or(c);
        cases.push(c.is_uppercase());
        if lc == 'v' {
            lowered.push('ü');
        } else {
            lowered.push(lc);
        }
        i += 1;
    }

    let mut s: String = lowered.into_iter().collect();
    if (s.starts_with("lu") || s.starts_with("nu")) && s.len() >= 3 && s[2..].starts_with('e') {
        let mut chars: Vec<char> = s.chars().collect();
        chars[1] = 'ü';
        s = chars.into_iter().collect();
    }

    (s, cases)
}

/// Determine whether the initial (if any) is one of j/q/x/y.
/// For these initials, `ü` written as `u` in the syllable table stays as `u` in output.
pub(crate) fn initial_is_jqxy(syllable_lower: &str) -> bool {
    syllable_lower.starts_with('j')
        || syllable_lower.starts_with('q')
        || syllable_lower.starts_with('x')
        || syllable_lower.starts_with('y')
}

/// Apply a tone mark to a syllable string (may contain multiple letters).
///
/// `syllable_raw`: the raw typed characters for this syllable (may have uppercase,
///                 may contain `v` for ü, may start with an initial).
/// `tone`: 1–4 for tone marks; 5 or 0 for neutral (strip any existing mark).
///
/// Returns the syllable with the tone mark applied, NFC normalized.
pub(crate) fn apply_tone(syllable_raw: &str, tone: Tone) -> String {
    if syllable_raw.is_empty() {
        return String::new();
    }

    // Neutralize: 5 and 0 mean no mark
    let effective_tone = if tone == 5 { 0 } else { tone };

    let (normalized, case_map) = normalize_to_lookup(syllable_raw);
    let is_jqxy = initial_is_jqxy(&normalized);

    // Find the vowel to mark
    let chars: Vec<char> = normalized.chars().collect();
    let mark_idx = find_vowel_index(&chars);

    let result: String = chars
        .iter()
        .enumerate()
        .flat_map(|(i, &c)| {
            let orig_upper = case_map.get(i).copied().unwrap_or(false);

            if Some(i) == mark_idx && effective_tone != 0 {
                // This is the vowel to mark
                let base = if c == 'ü' && is_jqxy { 'u' } else { c };
                apply_mark_to_char(base, effective_tone, orig_upper)
            } else {
                // Not marked: restore case, convert ü→u for j/q/x/y
                let out = if c == 'ü' && is_jqxy { 'u' } else { c };
                let out = if orig_upper {
                    out.to_uppercase().next().unwrap_or(out)
                } else {
                    out
                };
                vec![out]
            }
        })
        .collect();

    to_nfc(&result)
}

/// Find the index (into the char vec) of the vowel that should receive the tone mark.
fn find_vowel_index(chars: &[char]) -> Option<usize> {
    // Priority 1: 'a' or 'e'
    if let Some(i) = chars.iter().position(|&c| c == 'a' || c == 'e') {
        return Some(i);
    }
    // Priority 2: 'ou' → mark 'o'
    for i in 0..chars.len().saturating_sub(1) {
        if chars[i] == 'o' && chars[i + 1] == 'u' {
            return Some(i);
        }
    }
    // Priority 3: last standard vowel in the syllable
    if let Some(i) = chars.iter().rposition(|&c| is_vowel(c)) {
        return Some(i);
    }
    // Priority 4: 'ê'
    if let Some(i) = chars.iter().position(|&c| c == 'ê') {
        return Some(i);
    }
    // Priority 5: syllabic nasal 'm' or 'n'
    chars.iter().position(|&c| c == 'm' || c == 'n')
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ü')
}

/// Apply a tone mark to a single base char.
/// Returns a vec of chars (may be 1 char for precomposed, or 2 for base+combining).
fn apply_mark_to_char(base: char, tone: u8, uppercase: bool) -> Vec<char> {
    let base_lower = base.to_lowercase().next().unwrap_or(base);

    if let Some(marked_lower) = precomposed(base_lower, tone) {
        let marked = match_case(
            if uppercase {
                base.to_uppercase().next().unwrap_or(base)
            } else {
                base
            },
            marked_lower,
        );
        vec![marked]
    } else {
        // Fallback: base char + combining mark (e.g. m̀)
        let base_out = if uppercase {
            base.to_uppercase().next().unwrap_or(base)
        } else {
            base
        };
        if let Some(mark) = combining_mark(tone) {
            vec![base_out, mark]
        } else if tone == 4 {
            // m + grave (m̀) — no precomposed exists
            vec![base_out, COMBINING_GRAVE]
        } else {
            vec![base_out]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Group A: basic tone mark placement ──────────────────────────────────
    #[test]
    fn group_a_basic_placement() {
        let cases = [
            ("ni", 3, "nǐ"),
            ("hao", 3, "hǎo"),
            ("liu", 2, "liú"),
            ("gui", 4, "guì"),
            ("hou", 2, "hóu"),
            ("dou", 1, "dōu"),
            ("bei", 4, "bèi"),
            ("an", 1, "ān"),
            ("en", 2, "én"),
            ("yi", 4, "yì"),
            ("wu", 3, "wǔ"),
            ("yu", 2, "yú"),
            ("zhi", 1, "zhī"),
            ("chi", 2, "chí"),
            ("shi", 3, "shǐ"),
            ("ri", 4, "rì"),
            ("zi", 1, "zī"),
            ("ci", 2, "cí"),
            ("si", 3, "sǐ"),
        ];
        for (syl, tone, expected) in cases {
            assert_eq!(
                apply_tone(syl, tone),
                expected,
                "apply_tone({syl:?}, {tone}) failed"
            );
        }
    }

    // ── Group B: ü input variants ───────────────────────────────────────────
    #[test]
    fn group_b_u_umlaut() {
        let cases = [
            ("lv", 4, "lǜ"),
            ("lv", 2, "lǘ"),
            ("lv", 1, "lǖ"),
            ("lv", 3, "lǚ"),
            ("lve", 4, "lüè"),
            ("nv", 3, "nǚ"),
            ("nve", 4, "nüè"),
        ];
        for (syl, tone, expected) in cases {
            assert_eq!(
                apply_tone(syl, tone),
                expected,
                "apply_tone({syl:?}, {tone}) failed"
            );
        }
    }

    // ── j/q/x/y keep 'u' not 'ü' ────────────────────────────────────────────
    #[test]
    fn group_b_jqxy_keep_u() {
        // j/q/x/y+u: the u in the table is NOT ü, output stays u
        let cases = [
            ("ju", 4, "jù"),
            ("qu", 1, "qū"),
            ("xu", 2, "xú"),
            ("yu", 3, "yǔ"),
            ("xue", 2, "xué"),
            ("yuan", 2, "yuán"),
            ("jun", 4, "jùn"),
        ];
        for (syl, tone, expected) in cases {
            assert_eq!(
                apply_tone(syl, tone),
                expected,
                "apply_tone({syl:?}, {tone}) failed"
            );
        }
    }

    // ── Group E: syllabic nasals ─────────────────────────────────────────────
    #[test]
    fn group_e_syllabic_nasals() {
        assert_eq!(apply_tone("n", 2), "ń");
        assert_eq!(apply_tone("n", 4), "ǹ");
        assert_eq!(apply_tone("m", 2), "ḿ");
        // m+grave: combining mark (m̀), verify length = 2 Unicode scalars
        let m_grave = apply_tone("m", 4);
        assert!(m_grave.starts_with('m'), "m̀ should start with m");
        assert_eq!(m_grave.chars().count(), 2, "m̀ needs base+combining");
    }

    // ── Neutral / no-mark tones ──────────────────────────────────────────────
    #[test]
    fn neutral_tones() {
        assert_eq!(apply_tone("ma", 5), "ma");
        assert_eq!(apply_tone("ma", 0), "ma");
        assert_eq!(apply_tone("ni", 5), "ni");
    }

    // ── Case preservation ────────────────────────────────────────────────────
    #[test]
    fn case_preservation() {
        assert_eq!(apply_tone("Ni", 2), "Ní");
        assert_eq!(apply_tone("NI", 2), "NÍ");
        assert_eq!(apply_tone("HAO", 3), "HǍO");
        assert_eq!(apply_tone("Lv", 4), "Lǜ");
    }

    // ── Erhua base ───────────────────────────────────────────────────────────
    #[test]
    fn erhua_tone_on_base() {
        // "hua" gets tone 4 → "huà"; 'r' appended by engine separately
        assert_eq!(apply_tone("hua", 4), "huà");
        assert_eq!(apply_tone("er", 2), "ér");
    }
}
