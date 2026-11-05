//! Role: Assemble numeric control modules.
mod implementation;
mod machine;
mod paint;
mod text;
pub use implementation::{Numeric, NumericKind, PropertyState, fader, knob, number, value_slider};
pub use text::evaluate_expression;
