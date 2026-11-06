//! Role: Shared drawing primitives and authored draw records.
mod path;
pub mod primitives;
mod recording;
mod shape;

pub use primitives::{BorderStyle, FillRule, LineCap, LineJoin, Stroke};
pub use recording::{
    DrawCmd, DrawList, DrawingLayer, DrawingShape, Pixels, Region, ShapePaint, TextRun,
};

pub use path::PathData;
pub use shape::{PathCmd, Radius, RoundedRect, Shape};
