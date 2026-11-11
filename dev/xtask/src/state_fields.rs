//! Role: Detect combinations of state fields that hide a state machine.
//!
//! Two bool/Option/counter fields, or an Option beside a State/Phase enum,
//! require an explicit state model. Role suffixes, records, value modules,
//! Element implementations and Api option builders describe data instead.
//! The rule closes the known gaps (SL-1 to SL-8) for every crate.
//! A projection field is visible to one module subtree below the crate root, and only the one writer its marker names changes it there; an independent reason is not shared with another file (SL-9).

mod audit;
mod check;
mod claims;
mod classify;
mod locals;
mod mode;
#[cfg(test)]
mod tests;

pub(crate) use audit::{AuditRow, open_count, write};
pub(crate) use check::check;
pub(crate) use claims::verify;
pub(crate) use mode::Mode;
