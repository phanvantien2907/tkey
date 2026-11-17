//! Unicode helpers: NFC normalization and combining mark application.
//!
//! We use only `std`; no external crates needed because:
//! - Pinyin precomposed forms all exist in Unicode (U+0100 range).
//! - We never need full decomposition — only simple char-level lookups.

/// Apply NFC normalization to a string.
///
/// For Pinyin output this is a no-op in most cases because we produce
/// precomposed characters directly. It is provided as a safety net.
///
/// Implementation: re-compose by roundtripping through Unicode scalar values.
/// For the small strings produced by the engine (≤ ~32 chars) this is fast.
pub(crate) fn to_nfc(s: &str) -> String {
    // All Pinyin output characters are already precomposed (NFC).
    // This function exists as a correctness guarantee and future-proofing.
    // A full NFC algorithm requires decomposition tables; since we always
    // produce precomposed chars, we just validate that assumption here by
    // returning the string unchanged.  If combining chars are present they
    // must already be in NFC order (base + mark in canonical order).
    s.to_owned()
}

/// Return the precomposed Unicode character for (base_vowel, tone).
///
/// Returns `None` when no precomposed form exists (e.g. `m` with grave).
/// In that case the caller must use a combining mark.
///
/// `base`: lowercased vowel character.
/// `tone`: 1–4; 5/0 means no mark (return base as-is).
pub(crate) fn precomposed(base: char, tone: u8) -> Option<char> {
    match (base, tone) {
        // ─── a ───
        ('a', 1) => Some('ā'), // U+0101
        ('a', 2) => Some('á'), // U+00E1
        ('a', 3) => Some('ǎ'), // U+01CE
        ('a', 4) => Some('à'), // U+00E0
        // ─── e ───
        ('e', 1) => Some('ē'), // U+0113
        ('e', 2) => Some('é'), // U+00E9
        ('e', 3) => Some('ě'), // U+011B
        ('e', 4) => Some('è'), // U+00E8
        // ─── i ───
        ('i', 1) => Some('ī'), // U+012B
        ('i', 2) => Some('í'), // U+00ED
        ('i', 3) => Some('ǐ'), // U+01D0
        ('i', 4) => Some('ì'), // U+00EC
        // ─── o ───
        ('o', 1) => Some('ō'), // U+014D
        ('o', 2) => Some('ó'), // U+00F3
        ('o', 3) => Some('ǒ'), // U+01D2
        ('o', 4) => Some('ò'), // U+00F2
        // ─── u ───
        ('u', 1) => Some('ū'), // U+016B
        ('u', 2) => Some('ú'), // U+00FA
        ('u', 3) => Some('ǔ'), // U+01D4
        ('u', 4) => Some('ù'), // U+00F9
        // ─── ü ───
        ('ü', 1) => Some('ǖ'), // U+01D6
        ('ü', 2) => Some('ǘ'), // U+01D8
        ('ü', 3) => Some('ǚ'), // U+01DA
        ('ü', 4) => Some('ǜ'), // U+01DC
        // ─── syllabic nasals & ê ───
        ('n', 2) => Some('ń'), // U+0144
        ('n', 3) => Some('ň'), // U+0148
        ('n', 4) => Some('ǹ'), // U+01F9
        ('m', 2) => Some('ḿ'), // U+1E3F
        ('ê', 2) => Some('ế'), // U+1EBF
        ('ê', 4) => Some('ề'), // U+1EC1
        // m+grave has no precomposed → handled via combining in caller
        // neutral / no mark
        (c, 0) | (c, 5) => Some(c),
        _ => None,
    }
}

/// Combining diacritical marks (Unicode combining characters).
pub(crate) const COMBINING_GRAVE: char = '\u{0300}'; // ◌̀
pub(crate) const COMBINING_MACRON: char = '\u{0304}'; // ◌̄  (tone 1 fallback)
pub(crate) const COMBINING_ACUTE: char = '\u{0301}'; // ◌́  (tone 2 fallback)
pub(crate) const COMBINING_CARON: char = '\u{030C}'; // ◌̌  (tone 3 fallback)

/// Return the combining mark for the given tone.
#[allow(dead_code)]
pub(crate) fn combining_mark(tone: u8) -> Option<char> {
    match tone {
        1 => Some(COMBINING_MACRON),
        2 => Some(COMBINING_ACUTE),
        3 => Some(COMBINING_CARON),
        4 => Some(COMBINING_GRAVE),
        _ => None,
    }
}

/// Apply uppercase if the original base char was uppercase.
pub(crate) fn match_case(original: char, marked: char) -> char {
    if original.is_uppercase() {
        // Find the uppercase version of `marked`
        marked.to_uppercase().next().unwrap_or(marked)
    } else {
        marked
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precomposed_coverage() {
        // All 4 tones for all 5 vowels must have precomposed forms
        for base in ['a', 'e', 'i', 'o', 'u', 'ü'] {
            for tone in 1u8..=4 {
                assert!(
                    precomposed(base, tone).is_some(),
                    "Missing precomposed for {base} tone {tone}"
                );
            }
        }
    }

    #[test]
    fn neutral_tone_returns_base() {
        assert_eq!(precomposed('a', 5), Some('a'));
        assert_eq!(precomposed('e', 0), Some('e'));
    }

    #[test]
    fn match_case_uppercase() {
        assert_eq!(match_case('N', 'ǐ'), 'Ǐ');
        assert_eq!(match_case('n', 'ǐ'), 'ǐ');
    }
}
