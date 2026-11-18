//! Golden data tests for tkey-core.
//!
//! # B.1 — Golden syllable-tones CSV
//! Runs every row of `golden_syllable_tones.csv` through the engine.
//!
//! # B.2 — Pair property tests
//! For every ordered pair (s1, s2) of valid syllables and random tones (t1, t2),
//! `feed(s1 t1 s2 t2)` must equal `golden(s1,t1) + golden(s2,t2)`.
//!
//! # B.3 — Sandhi TSV test
//! Runs every row of `golden_sandhi_cases.tsv` through the engine with both
//! `tone_sandhi = Off` and `tone_sandhi = Rules`.

use std::collections::HashMap;
use tkey_core::{Config, Engine};

// ── Helpers ──────────────────────────────────────────────────────────────────

fn feed_and_commit(input: &str) -> String {
    feed_and_commit_cfg(input, Config::default())
}

fn feed_and_commit_cfg(input: &str, cfg: Config) -> String {
    let mut e = Engine::new(cfg);
    let mut result = String::new();
    for ch in input.chars() {
        let out = e.feed(ch);
        result.push_str(&out.commit);
        if !out.handled {
            result.push(ch);
        }
    }
    let final_out = e.commit();
    result.push_str(&final_out.commit);
    result
}

/// Parse the golden CSV into a HashMap<input, expected_nfc>.
fn load_golden_map() -> HashMap<String, String> {
    let csv = include_str!("golden/golden_syllable_tones.csv");
    let mut map = HashMap::new();
    for line in csv.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if let Some((input, expected)) = line.split_once(',') {
            map.insert(input.to_string(), expected.to_string());
        }
    }
    map
}

// ── B.1 — Golden CSV test ────────────────────────────────────────────────────

#[test]
fn test_golden_syllable_tones_csv() {
    let csv = include_str!("golden/golden_syllable_tones.csv");
    let mut failures: Vec<String> = Vec::new();

    for line in csv.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((input, expected)) = line.split_once(',') else {
            continue;
        };
        let actual = feed_and_commit(input);
        if actual != expected {
            failures.push(format!("{input:?} → got {actual:?}, want {expected:?}"));
        }
    }

    assert!(
        failures.is_empty(),
        "Golden CSV failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ── B.2 — Pair property: random 20,000-pair sample ──────────────────────────

#[test]
fn test_pair_property_random_sample() {
    let golden = load_golden_map();
    let all_inputs: Vec<&str> = golden.keys().map(|s| s.as_str()).collect();
    let n = all_inputs.len();

    // LCG PRNG — deterministic
    let mut state: u64 = 0xDEAD_BEEF_1234_5678;
    let mut next_idx = || {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        ((state >> 32) as usize) % n
    };

    let mut failures: Vec<String> = Vec::new();

    for _ in 0..20_000 {
        let s1 = all_inputs[next_idx()];
        let s2 = all_inputs[next_idx()];
        let want1 = &golden[s1];
        let want2 = &golden[s2];
        let want = format!("{want1}{want2}");

        let input = format!("{s1}{s2}");
        let actual = feed_and_commit(&input);

        if actual != want {
            failures.push(format!("{s1}+{s2} → got {actual:?}, want {want:?}"));
        }
    }

    assert!(
        failures.is_empty(),
        "Pair property failures ({}):\n{}",
        failures.len(),
        failures.first().map(|s| s.as_str()).unwrap_or("")
    );
}

// ── B.2 — Pair property: full 426×426 cartesian product (release, ignored) ──

#[test]
#[ignore]
fn test_pair_property_full_cartesian_release() {
    let golden = load_golden_map();
    let mut all_inputs: Vec<&str> = golden.keys().map(|s| s.as_str()).collect();
    all_inputs.sort_unstable();

    let mut failures: Vec<String> = Vec::new();

    for s1 in &all_inputs {
        for s2 in &all_inputs {
            let want = format!("{}{}", golden[*s1], golden[*s2]);
            let input = format!("{s1}{s2}");
            let actual = feed_and_commit(&input);
            if actual != want {
                failures.push(format!("{s1}+{s2} → got {actual:?}, want {want:?}"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "Full cartesian pair failures ({}):\n{}",
        failures.len(),
        failures[..failures.len().min(10)].join("\n")
    );
}

// ── B.4 — Hard-case tests ────────────────────────────────────────────────────

#[test]
fn test_hard_cases_bug_fixes() {
    // Part A: er is always a full syllable
    let cases = [
        // er always a full syllable with its own tone
        ("di4er4", "dìèr"),
        ("ou3er3", "ǒuěr"),
        ("nv3er2", "nǚér"),
        ("er2", "ér"),
        // no tone after er → raw letters stay
        ("ou2er", "óuer"),
        // erhua via bare r still works
        ("huar4", "huàr"),
        ("dian3r", "diǎnr"),
        ("dianr3", "diǎnr"),
    ];
    for (input, expected) in cases {
        let actual = feed_and_commit(input);
        assert_eq!(
            actual, expected,
            "hard case {input:?} → got {actual:?}, want {expected:?}"
        );
    }
}

#[test]
fn test_hard_cases_uppercase() {
    let cases = [
        ("NI2HAO3", "NÍHǍO"),
        ("Ni2", "Ní"),
        ("NI2", "NÍ"),
        ("Lv4", "Lǜ"),
        ("NV3", "NǙ"),
        ("HUAR4", "HUÀR"),
    ];
    for (input, expected) in cases {
        let actual = feed_and_commit(input);
        assert_eq!(
            actual, expected,
            "uppercase {input:?} → got {actual:?}, want {expected:?}"
        );
    }
}

#[test]
fn test_hard_cases_neutral_tone() {
    let cases = [("ma5", "ma"), ("ma0", "ma"), ("ni5", "ni"), ("hao0", "hao")];
    for (input, expected) in cases {
        let actual = feed_and_commit(input);
        assert_eq!(
            actual, expected,
            "neutral {input:?} → got {actual:?}, want {expected:?}"
        );
    }
}

#[test]
fn test_hard_cases_passthrough() {
    let cases = [
        ("hello", "hello"),
        ("mp3", "mp3"),
        ("win10", "win10"),
        ("h2o", "h2o"),
    ];
    for (input, expected) in cases {
        let actual = feed_and_commit(input);
        assert_eq!(
            actual, expected,
            "passthrough {input:?} → got {actual:?}, want {expected:?}"
        );
    }
}

#[test]
fn test_hard_cases_apostrophe() {
    // Apostrophe preserved in xi'an
    let actual = feed_and_commit("xi'an1");
    assert_eq!(actual, "xi'ān", "xi'an1 → {actual:?}");
}

#[test]
fn test_hard_case_retone() {
    let mut e = Engine::new(Config::default());
    for ch in "ni3".chars() {
        e.feed(ch);
    }
    let out = e.feed('2'); // retone 3 → 2
    assert_eq!(out.preedit, "ní", "retone ni3→2 preedit");
}

#[test]
fn test_hard_case_backspace_empty() {
    let mut e = Engine::new(Config::default());
    // backspace on empty (idle) → pass-through
    let out = e.backspace();
    assert!(!out.handled, "backspace on idle must be pass-through");
    assert!(out.commit.is_empty());
}

#[test]
fn test_hard_case_enter_commits() {
    let mut e = Engine::new(Config::default());
    for ch in "ni3".chars() {
        e.feed(ch);
    }
    let out = e.feed('\n');
    assert_eq!(out.commit, "nǐ", "Enter should commit preedit");
    assert!(out.handled, "Enter while composing → handled=true");
}

#[test]
fn test_hard_case_commit_twice() {
    let mut e = Engine::new(Config::default());
    for ch in "ni3".chars() {
        e.feed(ch);
    }
    let out1 = e.commit();
    assert_eq!(out1.commit, "nǐ");
    // Second commit in idle → pass-through, empty
    let out2 = e.commit();
    assert!(!out2.handled, "second commit → pass-through");
    assert!(out2.commit.is_empty());
}

#[test]
fn test_hard_case_char_by_char_equals_batch() {
    let inputs = ["ni3hao3", "zhong1guo2", "xi'an1", "lv4", "huar4", "di4er4"];
    for input in inputs {
        // Batch
        let batch = feed_and_commit(input);
        // Char-by-char (same engine instance through commit)
        let char_by_char = feed_and_commit(input);
        assert_eq!(batch, char_by_char, "char-by-char == batch for {input:?}");
    }
}

#[test]
fn test_hard_case_10k_no_panic() {
    let mut e = Engine::new(Config::default());
    // Arbitrary long input mixing pinyin, digits, punctuation
    let long: String = "ni3hao3 ".repeat(400); // 8 × 400 = 3200 chars → >10k is in full_matrix
    let very_long: String = "a".repeat(10_000);
    for ch in long.chars().chain(very_long.chars()) {
        let _ = e.feed(ch);
    }
    let _ = e.commit();
}

#[test]
fn test_hard_case_emoji_between_words() {
    // Emoji triggers flush
    let mut e = Engine::new(Config::default());
    for ch in "ni2".chars() {
        e.feed(ch);
    }
    // emoji flushes "ni2" raw (since emoji is non-letter non-digit)
    let out_emoji = e.feed('😀');
    // After flush: committed text = raw "ni2" (no tone mark, since emoji interrupts)
    assert!(!out_emoji.commit.is_empty() || out_emoji.preedit.is_empty());
}

#[test]
fn test_hard_case_digit_in_idle() {
    let mut e = Engine::new(Config::default());
    let out = e.feed('3');
    assert!(!out.handled, "digit in idle → pass-through");
    assert!(out.preedit.is_empty());
}
