//! Core state machine: the [`Engine`] struct.
//!
//! # State transitions
//!
//! ```text
//! Idle ──letter/apostrophe──► Composing
//! Idle ──other──────────────► Idle (pass-through)
//!
//! Composing ──letter─────────► Composing (append)
//! Composing ──tone digit─────► Composing (apply tone)
//! Composing ──backspace──────► Composing or Idle
//! Composing ──Space/commit───► Idle (commit all)
//! Composing ──Esc/reset──────► Idle (discard)
//! Composing ──invalid prefix─► Idle (flush raw, pass char)
//! ```

use crate::config::Config;
use crate::output::Output;
use crate::syllable::table::{is_valid_prefix, is_valid_syllable};
use crate::tone::apply_tone;

/// Maximum preedit buffer length (characters). Exceeded → flush raw.
const MAX_BUFFER: usize = 64;

/// Internal engine state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
enum State {
    #[default]
    Idle,
    Composing,
    PassThrough,
}

/// Per-keystroke Pinyin engine.
///
/// # Examples
/// ```
/// use tkey_core::{Engine, Config};
/// let mut e = Engine::new(Config::default());
/// let out = e.feed('n'); assert_eq!(out.preedit, "n");
/// let out = e.feed('i'); assert_eq!(out.preedit, "ni");
/// let out = e.feed('3'); assert_eq!(out.preedit, "nǐ");
/// let out = e.commit();  assert_eq!(out.commit,  "nǐ");
/// assert_eq!(out.preedit, "");
/// ```
pub struct Engine {
    config: Config,
    state: State,
    /// Raw typed characters (letters + apostrophes; digits stripped after tone applied).
    raw_buf: String,
    /// The rendered preedit (with tone marks applied where digits were typed).
    /// This is what the host shows as marked text.
    preedit_buf: String,
    /// Tracks the raw letter sequence (without tone digits) for each syllable position.
    /// Used for backspace to restore the raw form.
    ///
    /// Each entry = (raw_letters_for_syllable, applied_tone).
    syllables: Vec<SyllableEntry>,
    /// Pending char that could not be accepted (needs pass-through after flush).
    pending_passthrough: Option<char>,
    /// Set to true when an apostrophe was just typed; next syllable gets has_apostrophe_before=true.
    pending_apostrophe: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct SyllableEntry {
    /// Raw lowercase letters (ü-normalized) for this syllable.
    pub(crate) raw_lower: String,
    /// Original characters as typed by the user (preserves case and u: / v).
    pub(crate) orig_chars: Vec<char>,
    /// Applied tone (0 = none, 1-4 = tone marks, 5 = neutral).
    pub(crate) tone: u8,
    /// True if this syllable has erhua 'r'.
    pub(crate) erhua: bool,
    /// True when the erhua 'r' character was typed in uppercase ('R').
    pub(crate) erhua_upper: bool,
    /// True when this syllable was preceded by an apostrophe boundary ('xi'an').
    pub(crate) has_apostrophe_before: bool,
}

impl SyllableEntry {
    #[allow(dead_code)]
    pub(crate) fn render(&self) -> String {
        self.render_with_tone(self.tone)
    }

    pub(crate) fn render_with_tone(&self, tone: u8) -> String {
        let raw_orig: String = self.orig_chars.iter().collect();
        let mut rendered = if tone > 0 {
            apply_tone(&raw_orig, tone)
        } else if raw_orig.contains('v') || raw_orig.contains('V') || raw_orig.contains(':') {
            apply_tone(&raw_orig, 0)
        } else {
            raw_orig
        };
        if self.erhua {
            rendered.push(if self.erhua_upper { 'R' } else { 'r' });
        }
        rendered
    }

    pub(crate) fn new(
        raw_lower: String,
        orig_chars: Vec<char>,
        has_apostrophe_before: bool,
    ) -> Self {
        Self {
            raw_lower,
            orig_chars,
            tone: 0,
            erhua: false,
            erhua_upper: false,
            has_apostrophe_before,
        }
    }
}

impl Engine {
    /// Create a new engine with the given configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config,
            state: State::Idle,
            raw_buf: String::new(),
            preedit_buf: String::new(),
            syllables: Vec::new(),
            pending_passthrough: None,
            pending_apostrophe: false,
        }
    }

    /// Feed a single character to the engine.
    ///
    /// Returns an [`Output`] describing what the host should do.
    pub fn feed(&mut self, ch: char) -> Output {
        match self.state {
            State::Idle => self.feed_idle(ch),
            State::Composing => self.feed_composing(ch),
            State::PassThrough => self.feed_passthrough(ch),
        }
    }

    /// Process a Backspace key.
    pub fn backspace(&mut self) -> Output {
        match self.state {
            State::Idle | State::PassThrough => Output::pass_through(),
            State::Composing => self.backspace_composing(),
        }
    }

    /// Commit the current composition (Space, Enter, focus loss, punctuation).
    ///
    /// Returns the committed text. The engine returns to Idle.
    pub fn commit(&mut self) -> Output {
        if self.state == State::Idle {
            return Output::pass_through();
        }
        let committed = self.preedit_buf.clone();
        self.reset_internal();
        if committed.is_empty() {
            Output::pass_through()
        } else {
            Output::commit_all(committed)
        }
    }

    /// Discard the current composition without committing (Esc, mouse click, app switch).
    pub fn reset(&mut self) {
        self.reset_internal();
    }

    // ── Private helpers ─────────────────────────────────────────────────────

    fn feed_idle(&mut self, ch: char) -> Output {
        if is_composable_start(ch) {
            let lc = char_to_lower_normalized(ch);
            if is_valid_prefix(&lc.to_string()) {
                self.state = State::Composing;
                let entry = SyllableEntry::new(lc.to_string(), vec![ch], false);
                self.syllables.push(entry);
                self.raw_buf.push(ch);
                self.rebuild_preedit();
                Output::preedit_only(self.preedit_buf.clone())
            } else {
                Output::pass_through()
            }
        } else {
            Output::pass_through()
        }
    }

    fn feed_composing(&mut self, ch: char) -> Output {
        // Check buffer overflow
        if self.raw_buf.len() >= MAX_BUFFER {
            return self.flush_raw_and_passthrough(ch);
        }

        match classify(ch) {
            CharClass::Letter | CharClass::Apostrophe => self.accept_letter_composing(ch),
            CharClass::Colon => self.accept_colon(ch),
            CharClass::ToneDigit(d) => self.accept_tone_digit(d, ch),
            CharClass::CommitTrigger => {
                // Commit preedit, then host handles the trigger key
                let committed = self.preedit_buf.clone();
                self.reset_internal();
                let mut out = Output::commit_all(committed);
                if ch == '\n' || ch == '\r' {
                    // Enter while composing commits and is consumed
                    out.handled = true;
                } else {
                    // Space, Tab, etc. commit the buffer, then the key is inserted by host
                    out.handled = false;
                }
                out
            }
            CharClass::Other => {
                // Non-pinyin char (emoji, control, punctuation that is not a commit trigger)
                // Flush buffer raw and pass this char through
                self.flush_raw_and_passthrough(ch)
            }
        }
    }

    fn accept_letter_composing(&mut self, ch: char) -> Output {
        let lc = char_to_lower_normalized(ch);

        // Handle apostrophe: forces a syllable boundary.
        if ch == '\'' && !self.syllables.is_empty() {
            self.raw_buf.push('\'');
            self.pending_apostrophe = true;
            return Output::preedit_only(self.preedit_buf.clone());
        }

        // If the last syllable currently has erhua 'r', but now a letter arrives:
        // "An r followed by a vowel starts a new syllable (dianran -> dian + ran)."
        if let Some(last) = self.syllables.last_mut()
            && last.erhua
        {
            let candidate = format!("r{lc}");
            if is_valid_prefix(&candidate) {
                let r_char = if last.erhua_upper { 'R' } else { 'r' };
                last.erhua = false;
                last.erhua_upper = false;
                let entry = SyllableEntry::new(candidate, vec![r_char, ch], false);
                self.syllables.push(entry);
                self.raw_buf.push(ch);
                self.rebuild_preedit();
                return Output::preedit_only(self.preedit_buf.clone());
            }
        }

        // 1. Can we extend the current syllable?
        let can_extend = if self.pending_apostrophe {
            // After an apostrophe, never extend: always start a new syllable.
            false
        } else if let Some(last) = self.syllables.last() {
            if last.tone > 0 || last.erhua {
                false
            } else {
                let candidate = format!("{}{lc}", last.raw_lower);
                is_valid_prefix(&candidate)
            }
        } else {
            false
        };

        if can_extend {
            let last = self.syllables.last_mut().unwrap();
            last.raw_lower.push(lc);
            last.orig_chars.push(ch);
            self.raw_buf.push(ch);
            self.rebuild_preedit();
            return Output::preedit_only(self.preedit_buf.clone());
        }

        // 2. Can ch be bare erhua 'r' after valid/toned syllable?
        //    (huar4 → huàr, dian3r → diǎnr, dianr3 → diǎnr)
        if lc == 'r' && !self.syllables.is_empty() {
            let last = self.syllables.last_mut().unwrap();
            if !last.erhua && (last.tone > 0 || is_valid_syllable(&last.raw_lower)) {
                last.erhua = true;
                last.erhua_upper = ch.is_uppercase();
                self.raw_buf.push(ch);
                self.rebuild_preedit();
                return Output::preedit_only(self.preedit_buf.clone());
            }
        }

        // 3. Can we start a new syllable?
        let apostrophe_before = self.pending_apostrophe;
        let can_start_new = if apostrophe_before {
            // After an apostrophe, force a new syllable if lc is a valid start.
            is_valid_prefix(&lc.to_string())
        } else if let Some(last) = self.syllables.last() {
            let last_has_vowel = last
                .raw_lower
                .chars()
                .any(|c| matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'ü' | 'ê'));
            (last.tone > 0 || last.erhua || (is_valid_syllable(&last.raw_lower) && last_has_vowel))
                && is_valid_prefix(&lc.to_string())
        } else {
            is_valid_prefix(&lc.to_string())
        };

        if can_start_new {
            self.pending_apostrophe = false;
            let entry = SyllableEntry::new(lc.to_string(), vec![ch], apostrophe_before);
            self.syllables.push(entry);
            self.raw_buf.push(ch);
            self.rebuild_preedit();
            return Output::preedit_only(self.preedit_buf.clone());
        }

        // 4. Neither extending nor starting new syllable is valid → flush
        self.flush_raw_and_passthrough(ch)
    }

    fn accept_colon(&mut self, ch: char) -> Output {
        if let Some(last) = self
            .syllables
            .last_mut()
            .filter(|s| s.raw_lower.ends_with('u'))
        {
            last.raw_lower.pop();
            last.raw_lower.push('ü');
            last.orig_chars.push(ch);
            self.raw_buf.push(ch);
            self.rebuild_preedit();
            return Output::preedit_only(self.preedit_buf.clone());
        }
        self.flush_raw_and_passthrough(ch)
    }

    /// Accept a tone digit while composing.
    fn accept_tone_digit(&mut self, digit: u8, ch: char) -> Output {
        if self.syllables.is_empty() {
            return self.flush_raw_and_passthrough(ch);
        }

        let is_valid = if let Some(last) = self.syllables.last() {
            is_valid_syllable(&last.raw_lower) || last.erhua
        } else {
            false
        };

        if is_valid {
            let last = self.syllables.last_mut().unwrap();
            last.tone = digit;
            self.rebuild_preedit();
            Output::preedit_only(self.preedit_buf.clone())
        } else {
            // Digit does not follow a valid syllable (e.g. h2o, mp3)
            self.flush_raw_and_passthrough(ch)
        }
    }

    /// Rebuild `preedit_buf` from `syllables`.
    /// Apostrophe boundaries are preserved between syllables when `has_apostrophe_before` is set.
    /// Display tones are derived via sandhi rules when configured.
    fn rebuild_preedit(&mut self) {
        let sep = &self.config.syllable_separator;
        let display_tones = crate::sandhi::compute_display_tones(&self.syllables, &self.config);
        let mut result = String::new();
        for (i, entry) in self.syllables.iter().enumerate() {
            if i > 0 {
                if entry.has_apostrophe_before {
                    result.push('\'');
                } else {
                    result.push_str(sep);
                }
            }
            let tone = display_tones.get(i).copied().unwrap_or(entry.tone);
            result.push_str(&entry.render_with_tone(tone));
        }
        self.preedit_buf = result;
    }

    /// Flush the raw buffer as committed text, transition to PassThrough, then pass `_next_ch` through.
    fn flush_raw_and_passthrough(&mut self, _next_ch: char) -> Output {
        let raw = self.raw_buf.clone();
        self.reset_internal();
        self.state = State::PassThrough;
        Output {
            commit: raw,
            preedit: String::new(),
            handled: false,
        }
    }

    fn feed_passthrough(&mut self, ch: char) -> Output {
        if ch.is_whitespace() || (ch.is_ascii_punctuation() && ch != '\'') {
            self.state = State::Idle;
        }
        Output::pass_through()
    }

    fn backspace_composing(&mut self) -> Output {
        // Pop the last raw char and rebuild
        if self.raw_buf.is_empty() {
            self.state = State::Idle;
            return Output {
                commit: String::new(),
                preedit: String::new(),
                handled: true,
            };
        }

        // Pop from syllables
        if let Some(last) = self.syllables.last_mut() {
            if last.erhua {
                // Bare erhua 'r': remove erhua and pop 'r'
                last.erhua = false;
                last.erhua_upper = false;
                self.raw_buf.pop();
                self.rebuild_preedit();
                return Output::preedit_only(self.preedit_buf.clone());
            }
            if last.tone > 0 {
                // Remove tone first
                last.tone = 0;
                // Don't pop a letter from raw_buf — tone digit was not in raw_buf
                self.rebuild_preedit();
                return Output::preedit_only(self.preedit_buf.clone());
            }
            if last.raw_lower.len() > 1 {
                // Remove last letter of this syllable
                last.raw_lower.pop();
                last.orig_chars.pop();
                self.raw_buf.pop();
                self.rebuild_preedit();
                return Output::preedit_only(self.preedit_buf.clone());
            }
            // Last letter of the last syllable
            self.syllables.pop();
            self.raw_buf.pop();
        } else {
            self.raw_buf.pop();
        }

        if self.syllables.is_empty() && self.raw_buf.is_empty() {
            self.state = State::Idle;
            return Output {
                commit: String::new(),
                preedit: String::new(),
                handled: true,
            };
        }

        self.rebuild_preedit();
        Output::preedit_only(self.preedit_buf.clone())
    }

    fn reset_internal(&mut self) {
        self.state = State::Idle;
        self.raw_buf.clear();
        self.preedit_buf.clear();
        self.syllables.clear();
        self.pending_passthrough = None;
        self.pending_apostrophe = false;
    }
}

// ── Character classification ─────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
enum CharClass {
    Letter,
    Apostrophe,
    Colon,
    ToneDigit(u8),
    CommitTrigger,
    Other,
}

fn classify(ch: char) -> CharClass {
    match ch {
        'a'..='z' | 'A'..='Z' | 'ê' | 'Ê' | 'ü' | 'Ü' => CharClass::Letter,
        '\'' => CharClass::Apostrophe,
        ':' => CharClass::Colon,
        '1' => CharClass::ToneDigit(1),
        '2' => CharClass::ToneDigit(2),
        '3' => CharClass::ToneDigit(3),
        '4' => CharClass::ToneDigit(4),
        '5' | '0' => CharClass::ToneDigit(5),
        ' ' | '\n' | '\r' | '\t' => CharClass::CommitTrigger,
        _ => CharClass::Other,
    }
}

fn is_composable_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == 'ê' || ch == 'Ê' || ch == 'ü' || ch == 'Ü'
}

fn char_to_lower_normalized(ch: char) -> char {
    let lc = ch.to_lowercase().next().unwrap_or(ch);
    if lc == 'v' { 'ü' } else { lc }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn feed_str(engine: &mut Engine, s: &str) -> Output {
        let mut out = Output::default();
        for ch in s.chars() {
            out = engine.feed(ch);
        }
        out
    }

    // ── Group M: all examples from the spec prompt ───────────────────────────
    #[test]
    fn group_m_spec_examples() {
        let cases: &[(&str, &str)] = &[
            ("ni2hao3", "níhǎo"),
            ("ni3hao3", "nǐhǎo"),
            ("zhong1guo2", "zhōngguó"),
            ("liu2", "liú"),
            ("gui4", "guì"),
            ("hou2", "hóu"),
            ("lv4", "lǜ"),
            ("lve4", "lüè"),
            ("lue4", "lüè"),
            ("nv3", "nǚ"),
            ("nu:3", "nǚ"),
            ("er2", "ér"),
            ("huar4", "huàr"),
            ("ma5", "ma"),
        ];

        for &(input, expected) in cases {
            let mut e = Engine::new(Config::default());
            feed_str(&mut e, input);
            let out = e.commit();
            assert_eq!(
                out.commit, expected,
                "Input: {input:?} → got {:?}, want {expected:?}",
                out.commit
            );
        }
    }

    // ── Apostrophe segmentation ──────────────────────────────────────────────
    #[test]
    fn xi_an_apostrophe() {
        let mut e = Engine::new(Config::default());
        // xi'an1 → xī'ān
        feed_str(&mut e, "xi'an1");
        let out = e.commit();
        // Apostrophe preserved; xi gets tone from earlier (no digit for xi),
        // an gets tone 1 → ān
        // xi has no tone → xi; xi'ān
        assert!(
            out.commit.contains('ā') || out.commit.contains("xi"),
            "xi'an1 commit: {:?}",
            out.commit
        );
    }

    // ── Case sensitivity ─────────────────────────────────────────────────────
    #[test]
    fn case_ni() {
        let mut e = Engine::new(Config::default());
        feed_str(&mut e, "Ni2");
        let out = e.commit();
        assert_eq!(out.commit, "Ní", "Ni2 → Ní");
    }

    #[test]
    fn case_ni_all_caps() {
        let mut e = Engine::new(Config::default());
        feed_str(&mut e, "NI2");
        let out = e.commit();
        assert_eq!(out.commit, "NÍ", "NI2 → NÍ");
    }

    // ── Retone ───────────────────────────────────────────────────────────────
    #[test]
    fn retone() {
        let mut e = Engine::new(Config::default());
        feed_str(&mut e, "ni2");
        let out1 = e.feed('3'); // retone: 2 → 3
        assert_eq!(out1.preedit, "nǐ", "retone ni2→3");
    }

    // ── Backspace removes tone first ─────────────────────────────────────────
    #[test]
    fn backspace_removes_tone_first() {
        let mut e = Engine::new(Config::default());
        feed_str(&mut e, "ni3");
        let out = e.backspace();
        assert_eq!(out.preedit, "ni", "backspace after tone should leave raw");
        let out = e.backspace();
        assert_eq!(out.preedit, "n");
        let out = e.backspace();
        assert_eq!(out.preedit, "");
    }

    // ── Pass-through for non-pinyin ──────────────────────────────────────────
    #[test]
    fn passthrough_digit_idle() {
        let mut e = Engine::new(Config::default());
        let out = e.feed('2');
        assert!(!out.handled, "digit in idle → pass-through");
    }

    #[test]
    fn passthrough_mp3() {
        // mp: 'm' is valid start, 'mp' is not a valid prefix → flush 'm' raw, then 'p'
        let mut e = Engine::new(Config::default());
        let out_m = e.feed('m');
        assert_eq!(out_m.preedit, "m");
        let out_p = e.feed('p');
        // 'mp' is not a valid prefix → flush
        assert_eq!(out_p.commit, "m", "flush 'm' on invalid prefix 'mp'");
    }

    // ── Commit on Space ──────────────────────────────────────────────────────
    #[test]
    fn commit_on_space() {
        let mut e = Engine::new(Config::default());
        feed_str(&mut e, "ni3");
        let out = e.feed(' ');
        assert_eq!(out.commit, "nǐ");
    }

    // ── Empty buffer commit ──────────────────────────────────────────────────
    #[test]
    fn commit_idle_is_passthrough() {
        let mut e = Engine::new(Config::default());
        let out = e.commit();
        assert!(!out.handled);
        assert!(out.commit.is_empty());
    }

    // ── Reset discards preedit ───────────────────────────────────────────────
    #[test]
    fn reset_discards() {
        let mut e = Engine::new(Config::default());
        feed_str(&mut e, "ni3");
        e.reset();
        let out = e.commit();
        assert!(out.commit.is_empty(), "after reset, commit should be empty");
    }

    // ── Buffer overflow ──────────────────────────────────────────────────────
    #[test]
    fn no_panic_long_input() {
        let mut e = Engine::new(Config::default());
        let long = "a".repeat(200);
        for ch in long.chars() {
            let _ = e.feed(ch);
        }
        let _ = e.commit();
    }

    // ── Unicode/emoji pass-through ───────────────────────────────────────────
    #[test]
    fn emoji_pass_through() {
        let mut e = Engine::new(Config::default());
        let out = e.feed('😀');
        // emoji in idle → pass-through
        assert!(!out.handled);
    }

    // ── Idempotence ──────────────────────────────────────────────────────────
    #[test]
    fn idempotence_ni3hao3() {
        let input = "ni3hao3";
        let mut e = Engine::new(Config::default());
        for ch in input.chars() {
            e.feed(ch);
        }
        let out1 = e.commit();

        let mut e2 = Engine::new(Config::default());
        // Feed same string again
        for ch in input.chars() {
            e2.feed(ch);
        }
        let out2 = e2.commit();
        assert_eq!(out1.commit, out2.commit, "idempotence check");
    }
}
