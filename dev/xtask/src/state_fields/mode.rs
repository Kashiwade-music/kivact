//! Role: The rule set and reporting mode of one state-fields run.

/// Which rule set to apply: the gaps the rule always had, or the widened
/// checks that every crate must satisfy before `state-fields` replaces them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Strictness {
    Current,
    Strict,
}

impl Strictness {
    /// The check name reported by this rule set.
    pub(crate) fn rule(self) -> &'static str {
        match self {
            Self::Current => "state-fields",
            Self::Strict => "state-fields-strict",
        }
    }
}

/// How one run reports: the rule set, and whether local guesses are notes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Mode {
    pub(crate) strictness: Strictness,
    pub(crate) report: bool,
}
