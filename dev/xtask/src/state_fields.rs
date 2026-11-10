//! Role: Detect combinations of state fields that hide a state machine.
//!
//! Two bool/Option/counter fields, or an Option beside a State/Phase enum,
//! require an explicit state model. Role suffixes, records, value modules,
//! Element implementations and Api option builders describe data instead.
//! The rule closes the known gaps (SL-1 to SL-8) for every crate.

mod audit;
mod check;
mod classify;
mod locals;
mod mode;
#[cfg(test)]
mod tests;

pub(crate) use audit::{AuditRow, open_count, write};
pub(crate) use check::check;
pub(crate) use mode::Mode;
