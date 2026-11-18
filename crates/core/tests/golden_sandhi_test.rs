//! Test for golden_sandhi_cases.tsv.
//!
//! Verifies:
//! - When `tone_sandhi = Off`, output equals `sandhi_off`.
//! - When `tone_sandhi = Rules`, output equals `sandhi_on`.

use tkey_core::Engine;
use tkey_core::config::{Config, ToneSandhi};

fn feed_and_commit_sandhi(input: &str, sandhi: ToneSandhi) -> String {
    let cfg = Config {
        tone_sandhi: sandhi,
        ..Default::default()
    };
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

#[test]
fn test_golden_sandhi_off() {
    let tsv = include_str!("golden/golden_sandhi_cases.tsv");
    let mut failures = Vec::new();

    for (line_idx, line) in tsv.lines().enumerate().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 3 {
            continue;
        }
        let typed = cols[0];
        let expected_off = cols[1];
        let note = if cols.len() > 3 { cols[3] } else { "" };

        let actual = feed_and_commit_sandhi(typed, ToneSandhi::Off);
        if actual != expected_off {
            failures.push(format!(
                "Line {}: typed={:?} → got {:?}, want {:?} ({})",
                line_idx + 1,
                typed,
                actual,
                expected_off,
                note
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Golden sandhi (Off) failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn test_golden_sandhi_on() {
    let tsv = include_str!("golden/golden_sandhi_cases.tsv");
    let mut failures = Vec::new();

    for (line_idx, line) in tsv.lines().enumerate().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 3 {
            continue;
        }
        let typed = cols[0];
        let expected_on = cols[2];
        let note = if cols.len() > 3 { cols[3] } else { "" };

        let actual = feed_and_commit_sandhi(typed, ToneSandhi::Rules);
        if actual != expected_on {
            failures.push(format!(
                "Line {}: typed={:?} → got {:?}, want {:?} ({})",
                line_idx + 1,
                typed,
                actual,
                expected_on,
                note
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Golden sandhi (Rules) failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
