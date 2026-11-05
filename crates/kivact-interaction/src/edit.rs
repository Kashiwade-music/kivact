//! Role: Edit session handles and their pure lifecycle transitions.
mod handle;
mod machine;
pub use handle::EditSession;
pub(crate) use machine::EditSessionState;
