//! Tone sandhi derivation (3rd tone, 不, 一, and lexicon exceptions).

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use crate::config::{Config, ToneSandhi};
use crate::engine::SyllableEntry;

static POSITIVE_LEXICON: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
static BLOCKLIST: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_lexicon() -> &'static HashMap<&'static str, &'static str> {
    POSITIVE_LEXICON.get_or_init(|| {
        let text = include_str!("../data/sandhi_lexicon.tsv");
        let mut map = HashMap::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((cit, rest)) = line.split_once('\t') {
                let pron = rest.split('\t').next().unwrap_or(rest);
                map.insert(cit, pron);
            }
        }
        map
    })
}

fn get_blocklist() -> &'static HashSet<&'static str> {
    BLOCKLIST.get_or_init(|| {
        let text = include_str!("../data/sandhi_blocklist.tsv");
        let mut set = HashSet::new();
        for line in text.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let cit = line.split('\t').next().unwrap_or(line);
            set.insert(cit);
        }
        set
    })
}

/// Compute the display tones for a slice of syllables based on config.
pub(crate) fn compute_display_tones(syllables: &[SyllableEntry], config: &Config) -> Vec<u8> {
    if config.tone_sandhi == ToneSandhi::Off || syllables.is_empty() {
        return syllables.iter().map(|s| s.tone).collect();
    }

    let mut display_tones: Vec<u8> = syllables.iter().map(|s| s.tone).collect();
    let n = syllables.len();

    // 1. Lexicon and word-level exception matching (forward maximum match)
    let lex = get_lexicon();
    let blk = get_blocklist();

    let mut i = 0;
    while i < n {
        // Try match window of length 4, 3, 2
        let max_len = (n - i).min(4);
        let mut matched = false;

        for len in (2..=max_len).rev() {
            let key: String = (i..i + len)
                .map(|idx| format!("{}{}", syllables[idx].raw_lower, syllables[idx].tone))
                .collect();

            // Check blocklist first
            if blk.contains(key.as_str()) {
                // Blocked from sandhi: keep citation tones
                i += len;
                matched = true;
                break;
            }

            // Check positive lexicon
            if let Some(&pron) = lex.get(key.as_str())
                && let Some(tones) = extract_tones_from_pronounced_key(pron, len)
            {
                for (offset, t) in tones.into_iter().enumerate() {
                    display_tones[i + offset] = t;
                }
                i += len;
                matched = true;
                break;
            }
        }

        if !matched {
            i += 1;
        }
    }

    // 2. Generic 不 and 一 rules for syllables not blocked
    for idx in 0..n {
        let is_bu = syllables[idx].raw_lower == "bu" && syllables[idx].tone == 4;
        let is_yi = syllables[idx].raw_lower == "yi" && syllables[idx].tone == 1;

        if is_bu && idx + 1 < n {
            let pair_key = format!(
                "bu4{}{}",
                syllables[idx + 1].raw_lower,
                syllables[idx + 1].tone
            );
            if !blk.contains(pair_key.as_str()) {
                let is_x_bu_x = idx > 0
                    && idx + 1 < n
                    && syllables[idx - 1].raw_lower == syllables[idx + 1].raw_lower;
                if is_x_bu_x {
                    display_tones[idx] = 5; // neutral
                } else if display_tones[idx + 1] == 4 {
                    display_tones[idx] = 2; // bu4 -> bu2 before tone 4
                }
            }
        }

        if is_yi {
            let is_v_yi_v = idx > 0
                && idx + 1 < n
                && syllables[idx - 1].raw_lower == syllables[idx + 1].raw_lower;
            if is_v_yi_v {
                display_tones[idx] = 5;
            } else if idx + 1 < n {
                let pair_key = format!(
                    "yi1{}{}",
                    syllables[idx + 1].raw_lower,
                    syllables[idx + 1].tone
                );
                if !blk.contains(pair_key.as_str()) {
                    let is_ordinal = idx > 0
                        && syllables[idx - 1].raw_lower == "di"
                        && syllables[idx - 1].tone == 4;
                    if !is_ordinal {
                        let next_tone = display_tones[idx + 1];
                        if next_tone == 4 {
                            display_tones[idx] = 2;
                        } else if matches!(next_tone, 1..=3) {
                            display_tones[idx] = 4;
                        }
                    }
                }
            }
        }
    }

    // 3. Third-tone sandhi (3+3 -> 2+3; all but last in a run of 3s)
    let mut run_start = 0;
    while run_start < n {
        if display_tones[run_start] == 3 {
            let mut run_end = run_start;
            while run_end < n && display_tones[run_end] == 3 {
                run_end += 1;
            }
            // All but last become 2nd tone
            for tone in display_tones[run_start..run_end.saturating_sub(1)].iter_mut() {
                *tone = 2;
            }
            run_start = run_end;
        } else {
            run_start += 1;
        }
    }

    display_tones
}

fn extract_tones_from_pronounced_key(key: &str, expected_count: usize) -> Option<Vec<u8>> {
    let mut tones = Vec::with_capacity(expected_count);
    for ch in key.chars() {
        if let Some(d) = ch.to_digit(10) {
            tones.push(d as u8);
        }
    }
    if tones.len() == expected_count {
        Some(tones)
    } else {
        None
    }
}
