//! Role: Shared drawing primitives and authored draw records.
pub mod primitives;
mod recording;

pub use primitives::{BorderStyle, FillRule, LineCap, LineJoin, Stroke};
pub use recording::{
    DrawingLayer, DrawingShape, Pixels, Region, Scene, SceneCmd, ShapePaint, TextRun,
};
