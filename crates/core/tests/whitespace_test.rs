//! Exhaustive test suite for whitespace handling, Unicode spaces, and feed_str helper.

use tkey_core::config::{Config, ToneSandhi};
use tkey_core::{Engine, feed_and_commit, feed_str};

fn manual_char_by_char(e: &mut Engine, s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        let out = match ch {
            '\x08' => e.backspace(),
            _ => e.feed(ch),
        };
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
fn test_feed_str_semantics_and_separation() {
    let mut e = Engine::new(Config::default());
    // 1. Without commit triggers, text remains in preedit
    let (committed, preedit) = e.feed_str("ni3hao3");
    assert_eq!(committed, "");
    assert_eq!(preedit, "nǐhǎo");

    // Commit explicitly
    let out = e.commit();
    assert_eq!(out.commit, "nǐhǎo");
    assert_eq!(e.preedit(), "");

    // 2. With whitespace between words, committed text has first word and spaces, preedit has second word
    let (committed2, preedit2) = e.feed_str("wo1   zhong1guo2");
    assert_eq!(committed2, "wō   ");
    assert_eq!(preedit2, "zhōngguó");

    let out2 = e.commit();
    assert_eq!(out2.commit, "zhōngguó");
}

#[test]
fn test_feed_and_commit_multiple_spaces() {
    let mut e = Engine::new(Config::default());
    assert_eq!(e.feed_and_commit("wo1   zhong1guo2"), "wō   zhōngguó");
    assert_eq!(e.feed_and_commit("  ni3hao3  "), "  nǐhǎo  ");
    assert_eq!(e.feed_and_commit("   "), "   ");
    assert_eq!(e.feed_and_commit(""), "");
}

#[test]
fn test_tabs_and_newlines() {
    let mut e = Engine::new(Config::default());
    assert_eq!(e.feed_and_commit("wo3\tni3"), "wǒ\tnǐ");
    assert_eq!(e.feed_and_commit("wo3\t\tni3"), "wǒ\t\tnǐ");

    // Enter (\n or \r) while composing commits and is consumed
    assert_eq!(e.feed_and_commit("wo3\n"), "wǒ");
    assert_eq!(e.feed_and_commit("wo3\r"), "wǒ");

    // In Idle, newlines and CRLF pass through untouched
    assert_eq!(e.feed_and_commit("\r\n"), "\r\n");
    assert_eq!(e.feed_and_commit("wo3\n\r\nni3"), "wǒ\r\nnǐ");
}

#[test]
fn test_unicode_spaces() {
    let mut e = Engine::new(Config::default());

    // NBSP (U+00A0)
    assert_eq!(e.feed_and_commit("wo3\u{00A0}ni3"), "wǒ\u{00A0}nǐ");

    // Ideographic CJK space (U+3000)
    assert_eq!(e.feed_and_commit("wo3\u{3000}ni3"), "wǒ\u{3000}nǐ");

    // Zero-width space (U+200B)
    assert_eq!(e.feed_and_commit("wo3\u{200B}ni3"), "wǒ\u{200B}nǐ");

    // Mixed spaces
    assert_eq!(
        e.feed_and_commit("wo3 \u{00A0}\t\u{3000}\u{200B}ni3"),
        "wǒ \u{00A0}\t\u{3000}\u{200B}nǐ"
    );
}

#[test]
fn test_whitespace_in_various_contexts() {
    let mut e = Engine::new(Config::default());

    // Space after tone digit
    assert_eq!(e.feed_and_commit("ni3 hao3"), "nǐ hǎo");

    // Space after pass-through word
    assert_eq!(e.feed_and_commit("hello world3"), "hello world3");
    assert_eq!(e.feed_and_commit("mp3 player"), "mp3 player");
    assert_eq!(e.feed_and_commit("win10 pro"), "win10 pro");

    // Space after apostrophe word
    assert_eq!(e.feed_and_commit("can't stop"), "can't stop");
    assert_eq!(e.feed_and_commit("xi'an1 city"), "xi'ān city");
    assert_eq!(e.feed_and_commit("xi1'an1 city"), "xī'ān city");

    // Space after backspace
    let mut e2 = Engine::new(Config::default());
    e2.feed_str("wo1\x08"); // wo1, backspace removes tone 1 -> "wo"
    let (c, p) = e2.feed_str(" "); // space commits "wo"
    assert_eq!(c, "wo ");
    assert_eq!(p, "");
}

#[test]
fn test_backspace_in_idle_after_space() {
    let mut e = Engine::new(Config::default());
    // Feed "wo1 "
    let (committed, preedit) = e.feed_str("wo1 ");
    assert_eq!(committed, "wō ");
    assert_eq!(preedit, "");

    // Engine is in Idle. Now press Backspace:
    let out = e.backspace();
    assert!(
        !out.handled,
        "Backspace in Idle must return handled=false so host deletes the space"
    );
    assert_eq!(out.commit, "");
    assert_eq!(out.preedit, "");

    // Engine must not undo or modify previous word
    let (committed2, preedit2) = e.feed_str("ni3");
    assert_eq!(committed2, "");
    assert_eq!(preedit2, "nǐ");
}

#[test]
fn test_sandhi_with_spaces() {
    let cfg = Config {
        tone_sandhi: ToneSandhi::Rules,
        ..Default::default()
    };
    let mut e = Engine::new(cfg);

    // Without space: sandhi fires (3+3 -> 2+3)
    assert_eq!(e.feed_and_commit("ni3hao3"), "níhǎo");

    // With single space: space is a phrase boundary -> no sandhi
    assert_eq!(e.feed_and_commit("ni3 hao3"), "nǐ hǎo");

    // With multiple spaces: no sandhi
    assert_eq!(e.feed_and_commit("ni3   hao3"), "nǐ   hǎo");

    // With tab: no sandhi
    assert_eq!(e.feed_and_commit("ni3\thao3"), "nǐ\thǎo");
}

#[test]
fn test_char_by_char_equals_batch() {
    let test_cases = [
        "wo1   zhong1guo2",
        "  ni3hao3  ",
        "   ",
        "",
        "wo3\tni3",
        "wo3\u{00A0}ni3",
        "wo3\u{3000}ni3",
        "wo3\u{200B}ni3",
        "hello world3",
        "can't stop",
        "xi'an1 city",
        "ni3  hao3",
    ];

    for s in test_cases {
        let mut e_batch = Engine::new(Config::default());
        let batch_result = e_batch.feed_and_commit(s);

        let mut e_char = Engine::new(Config::default());
        let char_result = manual_char_by_char(&mut e_char, s);

        assert_eq!(
            batch_result, char_result,
            "Mismatch between feed_and_commit and manual char-by-char for {:?}",
            s
        );
    }
}

#[test]
fn test_top_level_helpers() {
    let mut e = Engine::new(Config::default());
    let (c, p) = feed_str(&mut e, "wo1   ni3");
    assert_eq!(c, "wō   ");
    assert_eq!(p, "nǐ");

    let mut e2 = Engine::new(Config::default());
    let text = feed_and_commit(&mut e2, "wo1   ni3");
    assert_eq!(text, "wō   nǐ");
}
