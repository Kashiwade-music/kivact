//! Role: Owns validated window semantics and portable accessibility acceptance.
//!
//! A rejected update retains the accepted rooted hierarchy, focus and text runs.
//! Backends convert current accepted values under PlatformTree's lock and notify
//! application code only after releasing it. No native accessibility type enters
//! this module; native and injected requests use one portable acceptance rule.

mod action;
pub(crate) mod error;
mod handle;
mod runs;
pub(crate) mod shared;
pub(crate) mod state;
mod validation;

pub use handle::Accessibility;
