//! Role: Pair unhandled key presses with their releases for one embedded child.

use kivact_types::KeyCode;
use std::collections::HashSet;

/// What the App did with one key press.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::port) enum Verdict {
    /// A focused text input, focus movement, a handler or a default action used it.
    Handled,
    /// Nobody used it; an embedded editor with `forward_unhandled_keys` gives
    /// its native message back to the parent.
    Unhandled,
}

enum Probe {
    Idle,
    Open { code: KeyCode, verdict: Verdict },
}

/// The synchronous unhandled-key probe and the presses given to the parent.
///
/// The engine decides "unhandled" and reports it while the turn runs, so a
/// native press is delivered, a client wake runs the turn, and the verdict is
/// read before the native message returns (INPUT-01 exception).
pub(in crate::port) struct KeyLedger {
    probe: Probe,
    forwarded: HashSet<KeyCode>,
}

impl Default for KeyLedger {
    fn default() -> Self {
        Self {
            probe: Probe::Idle,
            forwarded: HashSet::new(),
        }
    }
}

impl KeyLedger {
    /// Start probing a native press; a nested probe is an invariant violation.
    pub(in crate::port) fn open(&mut self, code: KeyCode) {
        match self.probe {
            Probe::Idle => {
                self.probe = Probe::Open {
                    code,
                    verdict: Verdict::Handled,
                };
            }
            Probe::Open { .. } => {
                unreachable!("invariant: an embedded key probe is not nested")
            }
        }
    }

    /// The engine reported that nobody handled the press of `code`.
    pub(in crate::port) fn unhandled(&mut self, code: KeyCode) {
        if let Probe::Open {
            code: probed,
            verdict,
        } = &mut self.probe
            && *probed == code
        {
            *verdict = Verdict::Unhandled;
        }
    }

    /// End the probe and return its verdict.
    pub(in crate::port) fn close(&mut self) -> Verdict {
        match std::mem::replace(&mut self.probe, Probe::Idle) {
            Probe::Open { verdict, .. } => verdict,
            Probe::Idle => Verdict::Handled,
        }
    }

    /// Remember that the press of `code` went to the parent.
    pub(in crate::port) fn forward(&mut self, code: KeyCode) {
        self.forwarded.insert(code);
    }

    /// Whether the press of `code` went to the parent; forget it.
    pub(in crate::port) fn take_forwarded(&mut self, code: KeyCode) -> bool {
        self.forwarded.remove(&code)
    }
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the key probe and release pairing.
    use super::{KeyLedger, Verdict};
    use kivact_types::KeyCode;

    #[test]
    fn a_probe_reports_only_its_own_code() {
        let mut keys = KeyLedger::default();
        keys.open(KeyCode::Space);
        keys.unhandled(KeyCode::Enter);
        assert_eq!(keys.close(), Verdict::Handled);
        keys.open(KeyCode::Space);
        keys.unhandled(KeyCode::Space);
        assert_eq!(keys.close(), Verdict::Unhandled);
        assert_eq!(keys.close(), Verdict::Handled);
    }

    #[test]
    fn a_release_follows_its_forwarded_press() {
        let mut keys = KeyLedger::default();
        keys.forward(KeyCode::Space);
        assert!(keys.take_forwarded(KeyCode::Space));
        assert!(!keys.take_forwarded(KeyCode::Space));
    }
}
