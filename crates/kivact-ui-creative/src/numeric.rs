//! Role: Assemble numeric control modules.

mod builder;
mod input;
mod machine;
mod paint;
mod scrub;
mod text;
mod view;

pub use builder::{
    Fader, Knob, Number, NumericKind, PropertyState, ValueSlider, fader, knob, number, value_slider,
};
pub use text::evaluate_expression;
