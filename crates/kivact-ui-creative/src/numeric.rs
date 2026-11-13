//! Role: Assemble numeric control modules.

mod builder;
mod input;
mod machine;
mod paint;
mod scrub;
mod text;
mod view;

pub use builder::{Numeric, NumericKind, PropertyState, fader, knob, number, value_slider};
pub use text::evaluate_expression;
