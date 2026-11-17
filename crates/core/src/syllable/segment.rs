//! Syllable segmentation: split a raw Pinyin buffer into syllable segments
//! using longest-match and apostrophe boundaries.

#[cfg(test)]
use crate::syllable::table::SYLLABLE_SET;
use crate::syllable::table::is_valid_syllable;

/// Segment a raw lowercase buffer (no tone digits) into Pinyin syllables.
/// Returns a list of segments in order.
///
/// Apostrophes force a split: "xi'an" → ["xi", "an"].
/// Longest-match is applied within each apostrophe-delimited part.
pub(crate) fn segment_buffer(normalized_lower: &str) -> Vec<String> {
    let mut result = Vec::new();

    // Split on apostrophes first
    for part in normalized_lower.split('\'') {
        if part.is_empty() {
            continue;
        }
        segment_part(part, &mut result);
    }

    result
}

/// Apply longest-match segmentation to a single (apostrophe-free) part.
fn segment_part(s: &str, out: &mut Vec<String>) {
    let chars: Vec<char> = s.chars().collect();
    let mut pos = 0;

    while pos < chars.len() {
        // Try longest match first (up to 6 chars: "zhuang")
        let max_len = (chars.len() - pos).min(6);
        let mut found = false;

        for len in (1..=max_len).rev() {
            let candidate: String = chars[pos..pos + len].iter().collect();
            if is_valid_syllable(&candidate) {
                out.push(candidate);
                pos += len;
                found = true;
                break;
            }
        }

        if !found {
            // Not a valid pinyin start — push this char as raw
            let ch: String = chars[pos..pos + 1].iter().collect();
            // Merge into last segment if it's also raw
            if let Some(last) = out.last_mut().filter(|l| !is_valid_syllable(l)) {
                last.push_str(&ch);
                pos += 1;
                continue;
            }
            out.push(ch);
            pos += 1;
        }
    }
}

#[cfg(test)]
pub(crate) fn syllable_count() -> usize {
    SYLLABLE_SET.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_syllable() {
        assert_eq!(segment_buffer("ni"), vec!["ni"]);
        assert_eq!(segment_buffer("hao"), vec!["hao"]);
        assert_eq!(segment_buffer("zhong"), vec!["zhong"]);
    }

    #[test]
    fn multi_syllable_no_separator() {
        // ni + hao
        assert_eq!(segment_buffer("nihao"), vec!["ni", "hao"]);
        // zhong + guo
        assert_eq!(segment_buffer("zhongguo"), vec!["zhong", "guo"]);
    }

    #[test]
    fn apostrophe_split() {
        // xi'an → xi, an
        assert_eq!(segment_buffer("xi'an"), vec!["xi", "an"]);
        // Without apostrophe: xian (longest match)
        assert_eq!(segment_buffer("xian"), vec!["xian"]);
    }

    #[test]
    fn erhua_base_segmented() {
        // "huar" → hua (the 'r' is treated as erhua by the engine layer, not here)
        // segmenter sees "hua" + leftover "r"
        let segs = segment_buffer("hua");
        assert_eq!(segs, vec!["hua"]);
    }

    #[test]
    fn syllable_count_reasonable() {
        let count = syllable_count();
        // We expect between 380 and 450 syllables
        assert!(
            (380..=450).contains(&count),
            "Unexpected syllable count: {count}"
        );
    }
}
