//! Role: Transforms and offsets shared geometry for painting.

use kivact_wire::Point;

/// Build a neutral affine transform from `[a, b, c, d, e, f]` coefficients.
pub fn transform(m: [f32; 6]) -> kivact_types::Transform {
    let [a, b, c, d, e, f] = m;
    kivact_types::Transform { a, b, c, d, e, f }
}

pub(crate) fn shifted(shape: kivact_types::Shape, origin: Point) -> kivact_types::Shape {
    let at = |p: Point| Point {
        x: p.x + origin.x,
        y: p.y + origin.y,
    };
    let rule = shape.rule();
    kivact_types::Shape::Path {
        rule,
        cmds: shape
            .outline()
            .into_iter()
            .map(|cmd| match cmd {
                kivact_types::PathCmd::MoveTo(p) => kivact_types::PathCmd::MoveTo(at(p)),
                kivact_types::PathCmd::LineTo(p) => kivact_types::PathCmd::LineTo(at(p)),
                kivact_types::PathCmd::QuadTo(p, q) => kivact_types::PathCmd::QuadTo(at(p), at(q)),
                kivact_types::PathCmd::CubicTo(p, q, r) => {
                    kivact_types::PathCmd::CubicTo(at(p), at(q), at(r))
                }
                other => other,
            })
            .collect(),
    }
}
