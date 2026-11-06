//! Role: Expose select behavior and keep its transition implementation private.
mod handle;
mod machine;
pub use handle::{SelectApi, use_select};
