//! Role: The numeric control's kinds, property states and resolved setup.

use super::text::{Formatter, TextParser};
use kivact::Prop;
use kivact::interaction::{ScalarMapping, ScalarRange, ScalarStep};
use kivact::state::{Callback, Signal, Value};
use kivact::style::Color;
use kivact::view::EditPhase;

/// Numeric geometry, with identical binding and keyboard behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NumericKind {
    Number,
    Slider,
    Knob,
    Fader,
}

/// Animation-state tint used by the source inspector.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Prop)]
pub enum PropertyState {
    #[default]
    Plain,
    Keyed,
    Animated,
    Driven,
}

/// The resolved state a numeric control mounts with: every default applied.
pub(super) struct Setup {
    pub(super) label: Value<String>,
    pub(super) state: Signal<f64>,
    pub(super) kind: NumericKind,
    pub(super) range: Value<ScalarRange>,
    pub(super) step: Value<ScalarStep>,
    pub(super) reset: Value<f64>,
    pub(super) precision: Value<usize>,
    pub(super) unit: Value<String>,
    pub(super) tint: Value<PropertyState>,
    pub(super) mapping: ScalarMapping,
    pub(super) format: Option<Formatter>,
    pub(super) drag_distance: Option<Value<f64>>,
    pub(super) text_entry: bool,
    pub(super) timecode: Option<Value<u32>>,
    pub(super) label_visible: Value<bool>,
    pub(super) accent: Option<Value<Color>>,
    pub(super) bipolar: Value<bool>,
    pub(super) horizontal: Value<bool>,
    pub(super) edit: Option<Callback<EditPhase>>,
    pub(super) text_parser: Option<TextParser>,
    pub(super) mixed: Value<bool>,
}
