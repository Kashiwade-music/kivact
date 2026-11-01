//! Role: Transforms and offsets shared geometry for painting.

use kivact_wire::Point;

/// Build a neutral affine transform from `[a, b, c, d, e, f]` coefficients.
pub fn transform(m: [f32; 6]) -> ui_types::Transform {
    let [a, b, c, d, e, f] = m;
    ui_types::Transform { a, b, c, d, e, f }
}

pub(crate) fn shifted(shape: ui_types::Shape, origin: Point) -> ui_types::Shape {
    let at = |p: Point| Point {
        x: p.x + origin.x,
        y: p.y + origin.y,
    };
    let rule = shape.rule();
    ui_types::Shape::Path {
        rule,
        cmds: shape
            .outline()
            .into_iter()
            .map(|cmd| match cmd {
                ui_types::PathCmd::MoveTo(p) => ui_types::PathCmd::MoveTo(at(p)),
                ui_types::PathCmd::LineTo(p) => ui_types::PathCmd::LineTo(at(p)),
                ui_types::PathCmd::QuadTo(p, q) => ui_types::PathCmd::QuadTo(at(p), at(q)),
                ui_types::PathCmd::CubicTo(p, q, r) => {
                    ui_types::PathCmd::CubicTo(at(p), at(q), at(r))
                }
                other => other,
            })
            .collect(),
    }
}
