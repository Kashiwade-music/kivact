//! Role: Canvas geometry is lowered once for drawing, culling and region hits.
use kivact_wire as wire;

/// Resolve borrowed wire geometry into an owned neutral logical outline.
pub fn shape(shape: &wire::DrawingShape) -> kivact_types::Shape {
    match shape {
        wire::DrawingShape::Basic { shape, rect } => super::shape::clip_path(shape, *rect),
        wire::DrawingShape::Rounded { rect, corners } => {
            super::shape::BoxShape::of(*rect, corners).shape()
        }
        wire::DrawingShape::Path { cmds, even_odd } => kivact_types::Shape::Path {
            cmds: cmds.clone(),
            rule: if *even_odd {
                kivact_types::FillRule::EvenOdd
            } else {
                kivact_types::FillRule::NonZero
            },
        },
    }
}
