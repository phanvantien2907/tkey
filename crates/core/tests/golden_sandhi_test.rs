//! Test for golden_sandhi_cases.tsv.
//!
//! Verifies:
//! - When `tone_sandhi = Off`, output equals `sandhi_off`.
//! - When `tone_sandhi = Rules, sandhi_blocklist = false`, output equals `sandhi_on_rule_always`.
//! - When `tone_sandhi = Rules, sandhi_blocklist = true`, output equals `sandhi_on_with_blocklist`.

use tkey_core::Engine;
use tkey_core::config::{Config, ToneSandhi};

fn feed_and_commit_sandhi(input: &str, sandhi: ToneSandhi, blocklist: bool) -> String {
    let cfg = Config {
        tone_sandhi: sandhi,
        sandhi_blocklist: blocklist,
        ..Default::default()
    };
    let mut e = Engine::new(cfg);
    e.feed_and_commit(input)
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
        if cols.len() < 4 {
            continue;
        }
        let typed = cols[0];
        let expected_off = cols[1];
        let note = if cols.len() > 4 { cols[4] } else { "" };

        let actual = feed_and_commit_sandhi(typed, ToneSandhi::Off, false);
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
fn test_golden_sandhi_rule_always() {
    let tsv = include_str!("golden/golden_sandhi_cases.tsv");
    let mut failures = Vec::new();

    for (line_idx, line) in tsv.lines().enumerate().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            continue;
        }
        let typed = cols[0];
        let expected_rule_always = cols[2];
        let note = if cols.len() > 4 { cols[4] } else { "" };

        let actual = feed_and_commit_sandhi(typed, ToneSandhi::Rules, false);
        if actual != expected_rule_always {
            failures.push(format!(
                "Line {}: typed={:?} → got {:?}, want {:?} ({})",
                line_idx + 1,
                typed,
                actual,
                expected_rule_always,
                note
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Golden sandhi (Rules Always / Blocklist Off) failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn test_golden_sandhi_with_blocklist() {
    let tsv = include_str!("golden/golden_sandhi_cases.tsv");
    let mut failures = Vec::new();

    for (line_idx, line) in tsv.lines().enumerate().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 4 {
            continue;
        }
        let typed = cols[0];
        let expected_with_blocklist = cols[3];
        let note = if cols.len() > 4 { cols[4] } else { "" };

        let actual = feed_and_commit_sandhi(typed, ToneSandhi::Rules, true);
        if actual != expected_with_blocklist {
            failures.push(format!(
                "Line {}: typed={:?} → got {:?}, want {:?} ({})",
                line_idx + 1,
                typed,
                actual,
                expected_with_blocklist,
                note
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "Golden sandhi (Rules with Blocklist) failures ({}):\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn test_explicit_sandhi_idempotent() {
    for blocklist in [false, true] {
        for sandhi in [ToneSandhi::Off, ToneSandhi::Rules] {
            assert_eq!(
                feed_and_commit_sandhi("bu2dui4", sandhi, blocklist),
                "búduì",
                "bu2dui4 must stay búduì regardless of sandhi={:?}, blocklist={}",
                sandhi,
                blocklist
            );
            assert_eq!(
                feed_and_commit_sandhi("yi2ge4", sandhi, blocklist),
                "yígè",
                "yi2ge4 must stay yígè regardless of sandhi={:?}, blocklist={}",
                sandhi,
                blocklist
            );
            assert_eq!(
                feed_and_commit_sandhi("bu3dui4", sandhi, blocklist),
                "bǔduì",
                "bu3dui4 must stay bǔduì regardless of sandhi={:?}, blocklist={}",
                sandhi,
                blocklist
            );
            assert_eq!(
                feed_and_commit_sandhi("bu1dui4", sandhi, blocklist),
                "būduì",
                "bu1dui4 must stay būduì regardless of sandhi={:?}, blocklist={}",
                sandhi,
                blocklist
            );
        }
    }
}
