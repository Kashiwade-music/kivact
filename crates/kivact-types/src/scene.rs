//! Role: Shared drawing primitives and authored draw records.
mod path;
pub mod primitives;
mod recording;
mod resolve;
mod shape;

pub use primitives::{BorderStyle, FillRule, LineCap, LineJoin, Stroke};
pub use recording::{DrawCmd, DrawList, DrawingLayer, DrawingShape, Region, ShapePaint, TextRun};

pub use path::PathData;
pub use shape::{Outline, PathCmd, Radius, RoundedRect};
