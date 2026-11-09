//! Role: Detect combinations of state fields that hide a state machine.
//!
//! Two bool/Option/counter fields, or an Option beside a State/Phase enum,
//! require an explicit state model. Role suffixes, records, value modules,
//! NodeBuilder implementations and Api option builders describe data instead.
//! `Strictness::Strict` closes the known gaps of that rule (SL-1 to SL-8).

mod check;
mod classify;
mod locals;
#[cfg(test)]
mod tests;

pub(crate) use check::check;

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
