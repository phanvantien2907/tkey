//! `tkey-core` – pure-Rust IME engine for Pinyin (and future Vietnamese/Telex).
//!
//! This crate has **no OS dependencies** and **no runtime I/O**.
//! It exposes a per-keystroke [`Engine`] API designed for use by platform shells
//! (Swift InputMethodKit on macOS, TSF on Windows) or for testing in isolation.
//!
//! # Quick start
//! ```
//! use tkey_core::{Engine, Config};
//! let mut e = Engine::new(Config::default());
//! let out = e.feed('n'); assert_eq!(out.preedit, "n");
//! let out = e.feed('i'); assert_eq!(out.preedit, "ni");
//! let out = e.feed('3'); assert_eq!(out.preedit, "nǐ");
//! let out = e.commit();  assert_eq!(out.commit,  "nǐ");
//! ```

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod config;
pub mod output;

mod engine;
mod sandhi;
mod syllable;
mod tone;
mod unicode;

pub use config::{Config, ToneSandhi};
pub use engine::Engine;
pub use output::Output;

/// Feed a string of characters into an engine, returning `(committed, preedit)`.
///
/// Builds committed text from both engine commits and unhandled characters (`!handled`).
///
/// # Examples
/// ```
/// use tkey_core::{Engine, Config, feed_str};
/// let mut e = Engine::new(Config::default());
/// let (committed, preedit) = feed_str(&mut e, "wo1   zhong1guo2");
/// assert_eq!(committed, "wō   ");
/// assert_eq!(preedit, "zhōngguó");
/// ```
pub fn feed_str(engine: &mut Engine, s: &str) -> (String, String) {
    engine.feed_str(s)
}

/// Feed a string of characters into an engine and commit any remaining composition.
///
/// Returns the full text committed.
///
/// # Examples
/// ```
/// use tkey_core::{Engine, Config, feed_and_commit};
/// let mut e = Engine::new(Config::default());
/// assert_eq!(feed_and_commit(&mut e, "wo1   zhong1guo2"), "wō   zhōngguó");
/// ```
pub fn feed_and_commit(engine: &mut Engine, s: &str) -> String {
    engine.feed_and_commit(s)
}

/// Segments a toneless lowercase Pinyin string into valid syllables using greedy longest match.
///
/// Apostrophes force an explicit syllable split (e.g. `"xi'an"` -> `["xi", "an"]`).
///
/// # Examples
/// ```
/// use tkey_core::segment_pinyin;
/// assert_eq!(segment_pinyin("nihao"), vec!["ni", "hao"]);
/// assert_eq!(segment_pinyin("xi'an"), vec!["xi", "an"]);
/// ```
pub fn segment_pinyin(s: &str) -> Vec<String> {
    syllable::segment::segment_buffer(s)
}

/// Applies a Pinyin tone mark (1–4, or 0/5 for neutral) to a single syllable.
///
/// # Examples
/// ```
/// use tkey_core::apply_pinyin_tone;
/// assert_eq!(apply_pinyin_tone("ni", 3), "nǐ");
/// assert_eq!(apply_pinyin_tone("hao", 3), "hǎo");
/// assert_eq!(apply_pinyin_tone("lv", 4), "lǜ");
/// ```
pub fn apply_pinyin_tone(syllable: &str, tone: u8) -> String {
    tone::apply_tone(syllable, tone)
}
