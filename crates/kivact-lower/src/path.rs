//! Role: Transforms and offsets shared geometry for painting.

use kivact_wire::Point;

/// Build a neutral affine transform from `[a, b, c, d, e, f]` coefficients.
pub fn transform(m: [f32; 6]) -> kivact_types::protocol::Transform {
    let [a, b, c, d, e, f] = m;
    kivact_types::protocol::Transform { a, b, c, d, e, f }
}

pub(crate) fn shifted(
    shape: kivact_types::protocol::Shape,
    origin: Point,
) -> kivact_types::protocol::Shape {
    let at = |p: Point| Point::new(p.x + origin.x, p.y + origin.y);
    let rule = shape.rule();
    kivact_types::protocol::Shape::Path {
        rule,
        cmds: shape
            .outline()
            .into_iter()
            .map(|cmd| match cmd {
                kivact_types::protocol::PathCmd::MoveTo(p) => {
                    kivact_types::protocol::PathCmd::MoveTo(at(p))
                }
                kivact_types::protocol::PathCmd::LineTo(p) => {
                    kivact_types::protocol::PathCmd::LineTo(at(p))
                }
                kivact_types::protocol::PathCmd::QuadTo(p, q) => {
                    kivact_types::protocol::PathCmd::QuadTo(at(p), at(q))
                }
                kivact_types::protocol::PathCmd::CubicTo(p, q, r) => {
                    kivact_types::protocol::PathCmd::CubicTo(at(p), at(q), at(r))
                }
                other => other,
            })
            .collect(),
    }
}
