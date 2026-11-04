//! Role: Canvas geometry is lowered once for drawing, culling and region hits.
use kivact_types as types;

/// Resolve borrowed shared geometry into an owned neutral logical outline.
pub fn shape(shape: &types::DrawingShape) -> kivact_types::protocol::Shape {
    match shape {
        types::DrawingShape::Basic { shape, rect } => super::shape::clip_path(shape, *rect),
        types::DrawingShape::Rounded { rect, corners } => {
            super::shape::BoxShape::of(*rect, corners).shape()
        }
        types::DrawingShape::Path { cmds, even_odd } => kivact_types::protocol::Shape::Path {
            cmds: cmds.clone(),
            rule: if *even_odd {
                kivact_types::FillRule::EvenOdd
            } else {
                kivact_types::FillRule::NonZero
            },
        },
    }
}
