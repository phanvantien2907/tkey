//! Per-keystroke output returned to the host platform shell.

/// Result of a single engine operation.
///
/// The host platform shell inspects these fields to update the UI:
/// - Write `commit` into the document (remove from preedit).
/// - Display `preedit` as underlined/marked composition text.
/// - If `handled` is `false`, forward the original key event to the host app unchanged.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Output {
    /// Text to be inserted into the document. Empty when nothing is committed yet.
    pub commit: String,
    /// Current in-progress composition text (shown as preedit / marked text).
    pub preedit: String,
    /// Whether the engine consumed this event.
    ///
    /// `false` means the host should let the app handle the original key event.
    pub handled: bool,
}

impl Output {
    /// Construct a simple pass-through result.
    pub(crate) fn pass_through() -> Self {
        Self {
            commit: String::new(),
            preedit: String::new(),
            handled: false,
        }
    }

    /// Construct a handled result with updated preedit only.
    pub(crate) fn preedit_only(preedit: impl Into<String>) -> Self {
        Self {
            commit: String::new(),
            preedit: preedit.into(),
            handled: true,
        }
    }

    /// Construct a commit + empty preedit result.
    pub(crate) fn commit_all(commit: impl Into<String>) -> Self {
        Self {
            commit: commit.into(),
            preedit: String::new(),
            handled: true,
        }
    }
}
