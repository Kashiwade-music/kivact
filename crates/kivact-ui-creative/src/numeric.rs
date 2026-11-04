//! Role: Assemble numeric control modules.
mod implementation;
mod machine;
pub use implementation::{
    Numeric, NumericKind, PropertyState, evaluate_expression, fader, knob, number, value_slider,
};
