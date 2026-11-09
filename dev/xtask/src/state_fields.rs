//! Role: Detect combinations of state fields that hide a state machine.
//!
//! Two bool/Option/counter fields, or an Option beside a State/Phase enum,
//! require an explicit state model. Role suffixes, records, value modules,
//! NodeBuilder implementations and Api option builders describe data instead.

mod check;
mod classify;
mod locals;
#[cfg(test)]
mod tests;

pub(crate) use check::check;
