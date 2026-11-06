//! Role: Assemble state control modules.
mod data;
mod implementation;
mod machine;
pub(in crate::gradient) use data::GradientOwner;
pub(in crate::gradient) use implementation::{Model, State};
