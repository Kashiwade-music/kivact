//! Role: Canvas geometry is lowered once for drawing, culling and region hits.
use kivact_types as types;

/// Resolve borrowed shared geometry into an owned neutral logical outline.
pub(crate) fn shape(shape: &types::DrawingShape) -> kivact_types::scene::Shape {
    match shape {
        types::DrawingShape::Basic { shape, rect } => super::shape::clip_path(shape, *rect),
        types::DrawingShape::Rounded { rect, corners } => {
            super::shape::BoxShape::of(*rect, corners).shape()
        }
        types::DrawingShape::Path { path, rule } => kivact_types::scene::Shape::Path {
            cmds: path.cmds.clone(),
            rule: *rule,
        },
    }
}
