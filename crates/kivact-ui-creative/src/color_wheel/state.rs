//! Role: Assemble state control modules.
pub(in crate::color_wheel) mod controls_machine;
mod data;
mod implementation;
mod machine;
pub(in crate::color_wheel) use data::{ColorOwner, SwatchSelection};
pub(in crate::color_wheel) use implementation::{Model, State};
