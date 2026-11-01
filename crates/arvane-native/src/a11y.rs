//! Role: Converts accepted shell semantics to the native AccessKit adapter.
//!
//! Accepted hierarchy, validation and action acceptance live in arvane-shell.
//! Only this backend names AccessKit types, creating native updates inside the
//! adapter callback and the shell conversion lock, in that order.

mod conversion;
pub(crate) mod native;
