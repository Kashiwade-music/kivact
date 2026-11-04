//! Role: Transforms and offsets shared geometry for painting.

use kivact_types::Point;

pub(crate) fn shifted(
    shape: kivact_types::scene::Shape,
    origin: Point,
) -> kivact_types::scene::Shape {
    let at = |p: Point| Point {
        x: p.x + origin.x,
        y: p.y + origin.y,
    };
    let rule = shape.rule();
    kivact_types::scene::Shape::Path {
        rule,
        cmds: shape
            .outline()
            .into_iter()
            .map(|cmd| match cmd {
                kivact_types::scene::PathCmd::MoveTo(p) => {
                    kivact_types::scene::PathCmd::MoveTo(at(p))
                }
                kivact_types::scene::PathCmd::LineTo(p) => {
                    kivact_types::scene::PathCmd::LineTo(at(p))
                }
                kivact_types::scene::PathCmd::QuadTo(p, q) => {
                    kivact_types::scene::PathCmd::QuadTo(at(p), at(q))
                }
                kivact_types::scene::PathCmd::CubicTo(p, q, r) => {
                    kivact_types::scene::PathCmd::CubicTo(at(p), at(q), at(r))
                }
                other => other,
            })
            .collect(),
    }
}
