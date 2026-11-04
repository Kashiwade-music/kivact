//! Role: Graph curves, interpolation, view ranges and edit requests.
use kivact::style::{Color, rgb};
/// A handle offset in the same time and value units as its curve key.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CurveTangent {
    pub time: f64,
    pub value: f64,
}
impl CurveTangent {
    pub fn new(time: f64, value: f64) -> Self {
        Self { time, value }
    }
}

/// A curve key with relative incoming and outgoing handles.
#[derive(Clone, Debug, PartialEq)]
pub struct CurveKey {
    pub id: u64,
    pub time: f64,
    pub value: f64,
    pub interpolation: Interpolation,
    pub in_tangent: CurveTangent,
    pub out_tangent: CurveTangent,
}
impl CurveKey {
    pub fn new(id: u64, time: f64, value: f64) -> Self {
        Self {
            id,
            time,
            value,
            interpolation: Interpolation::Bezier,
            in_tangent: CurveTangent::new(-1.0, 0.0),
            out_tangent: CurveTangent::new(1.0, 0.0),
        }
    }
}

/// A named channel with keys whose IDs are unique across the graph.
#[derive(Clone, Debug, PartialEq)]
pub struct Curve {
    pub id: u64,
    pub label: String,
    pub color: Color,
    pub keys: Vec<CurveKey>,
}
impl Curve {
    pub fn new(id: u64, label: impl Into<String>, keys: Vec<CurveKey>) -> Self {
        Self {
            id,
            label: label.into(),
            color: rgb(0x8b7cff),
            keys,
        }
    }
    pub fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }
}

/// The visible logical time and value intervals of a graph.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphView {
    pub time: (f64, f64),
    pub value: (f64, f64),
}
impl Default for GraphView {
    fn default() -> Self {
        Self {
            time: (0.0, 100.0),
            value: (-2.0, 3.0),
        }
    }
}
kivact::state::constant_value!(GraphView);
/// An absolute proposed position and tangents for one graph key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyMove {
    pub id: u64,
    pub time: f64,
    pub value: f64,
    pub in_tangent: CurveTangent,
    pub out_tangent: CurveTangent,
}
/// A graph edit for the application to accept, constrain or ignore.
#[derive(Clone, Debug, PartialEq)]
pub enum GraphRequest {
    /// Absolute positions and relative tangents for all keys in the edit.
    Move(Vec<KeyMove>),
    /// Delete or Backspace on the selected keys.
    Delete(Vec<u64>),
    /// Set the interpolation of the selected keys.
    Interpolate(Vec<u64>, Interpolation),
}
/// The interpolation used from one graph point to its next point.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Interpolation {
    Constant,
    Linear,
    #[default]
    Bezier,
}
