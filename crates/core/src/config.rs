//! Engine configuration.

/// Input method typing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    /// Standard Hanyu Pinyin with tone digits (1-5/0).
    #[default]
    Pinyin,
    // Telex and Vni will be added here without breaking changes.
}

/// Tone sandhi behavior when displaying Pinyin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToneSandhi {
    /// No sandhi applied. Standard written Pinyin. (Default)
    #[default]
    Off,
    /// Apply tone sandhi rules (3+3, 不, 一, etc.)
    Rules,
}

/// Configuration for one [`Engine`](crate::Engine) session.
///
/// Cheap to clone; create once and pass to `Engine::new`.
///
/// # Examples
/// ```
/// use tkey_core::Config;
/// let cfg = Config::default();
/// assert_eq!(cfg.syllable_separator, "");
/// ```
#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Typing mode.
    pub mode: Mode,
    /// String inserted between committed syllables (default `""`).
    /// Set to `" "` to produce `ní hǎo` instead of `níhǎo`.
    pub syllable_separator: String,
    /// Tone sandhi behavior.
    pub tone_sandhi: ToneSandhi,
    // Future: pub shortcuts: Arc<ShortcutTable>,
}
