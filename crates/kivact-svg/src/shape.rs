//! Role: Resolves drawn shapes to path commands and writes SVG path data.
//!
//! A shape becomes one list of path commands in the drawing's coordinates
//! (the same outlines the window fills). Elliptical arcs of a path become SVG
//! `A` commands; the arcs of a `PathCmd::Arc` are cut into pieces of at most a
//! quarter turn, so a full turn is exact.

use super::encode::number;
use crate::SvgError;
use kivact_types::scene::{PathCmd, RoundedRect, Shape};
use kivact_types::{ClipShape, Corners, DrawingShape, FillRule, Point, Rect};

/// Pieces of a `PathCmd::Arc` are at most this many degrees.
const ARC_PIECE_DEG: f64 = 90.0;

/// The path commands, fill rule and bounding box (from the outline's points,
/// control points included) of a drawn shape.
pub(super) fn outline_of(shape: &DrawingShape) -> (Vec<PathCmd>, FillRule, Rect) {
    let (cmds, rule) = match shape {
        DrawingShape::Path { path, rule } => (path.cmds.clone(), *rule),
        DrawingShape::Rounded { rect, corners } => {
            (rounded(*rect, corners).outline(), FillRule::NonZero)
        }
        DrawingShape::Basic { shape, rect } => basic(shape, *rect),
    };
    let bounds = bounds_of(&cmds);
    (cmds, rule, bounds)
}

/// A CSS basic shape in its box, as the window lowers it.
fn basic(shape: &ClipShape, rect: Rect) -> (Vec<PathCmd>, FillRule) {
    let rule_of = |even_odd: bool| {
        if even_odd {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        }
    };
    let at = |[x, y]: [kivact_types::Len; 2]| Point {
        x: rect.x + x.of(rect.width),
        y: rect.y + y.of(rect.height),
    };
    match shape {
        ClipShape::Path { cmds, even_odd } => {
            let origin = Point {
                x: rect.x,
                y: rect.y,
            };
            (
                cmds.iter().map(|cmd| shift(*cmd, origin)).collect(),
                rule_of(*even_odd),
            )
        }
        ClipShape::Inset { edges, radius } => {
            let left = edges.left.of(rect.width);
            let top = edges.top.of(rect.height);
            let inner = Rect {
                x: rect.x + left,
                y: rect.y + top,
                width: (rect.width - left - edges.right.of(rect.width)).max(0.0),
                height: (rect.height - top - edges.bottom.of(rect.height)).max(0.0),
            };
            (rounded(inner, radius).outline(), FillRule::NonZero)
        }
        ClipShape::Circle { radius, center } => {
            let reference = rect.width.hypot(rect.height) / std::f64::consts::SQRT_2;
            let r = radius.of(reference).max(0.0);
            let c = at(*center);
            ellipse(c, r, r)
        }
        ClipShape::Ellipse {
            radius_x,
            radius_y,
            center,
        } => {
            let rx = radius_x.of(rect.width).max(0.0);
            let ry = radius_y.of(rect.height).max(0.0);
            ellipse(at(*center), rx, ry)
        }
        ClipShape::Polygon { points, even_odd } => {
            let shape = Shape::Polygon {
                points: points.iter().map(|p| at(*p)).collect(),
                rule: rule_of(*even_odd),
            };
            (shape.outline(), shape.rule())
        }
    }
}

/// The outline of a rectangle with elliptical corners of `corners`.
fn rounded(rect: Rect, corners: &Corners) -> RoundedRect {
    let radii = corners.each().map(|cr| kivact_types::scene::Radius {
        x: cr.x.of(rect.width).max(0.0),
        y: cr.y.of(rect.height).max(0.0),
    });
    RoundedRect { rect, radii }
}

/// The ellipse of radii `rx` × `ry` centered at `c`.
fn ellipse(c: Point, rx: f64, ry: f64) -> (Vec<PathCmd>, FillRule) {
    let shape = RoundedRect::ellipse(Rect {
        x: c.x - rx,
        y: c.y - ry,
        width: 2.0 * rx,
        height: 2.0 * ry,
    });
    (shape.outline(), FillRule::NonZero)
}

/// `cmd` moved by `origin`.
fn shift(cmd: PathCmd, origin: Point) -> PathCmd {
    let at = |p: Point| Point {
        x: p.x + origin.x,
        y: p.y + origin.y,
    };
    match cmd {
        PathCmd::MoveTo(p) => PathCmd::MoveTo(at(p)),
        PathCmd::LineTo(p) => PathCmd::LineTo(at(p)),
        PathCmd::QuadTo(c, p) => PathCmd::QuadTo(at(c), at(p)),
        PathCmd::CubicTo(c1, c2, p) => PathCmd::CubicTo(at(c1), at(c2), at(p)),
        PathCmd::ArcTo {
            rx,
            ry,
            x_rotation_deg,
            large_arc,
            sweep,
            to,
        } => PathCmd::ArcTo {
            rx,
            ry,
            x_rotation_deg,
            large_arc,
            sweep,
            to: at(to),
        },
        PathCmd::Arc {
            center,
            rx,
            ry,
            x_rotation_deg,
            start_deg,
            sweep_deg,
        } => PathCmd::Arc {
            center: at(center),
            rx,
            ry,
            x_rotation_deg,
            start_deg,
            sweep_deg,
        },
        PathCmd::Close => PathCmd::Close,
    }
}

/// The bounding box of the points a command list visits.
fn bounds_of(cmds: &[PathCmd]) -> Rect {
    let mut points = Vec::new();
    for cmd in cmds {
        match cmd {
            PathCmd::MoveTo(p) | PathCmd::LineTo(p) => points.push(*p),
            PathCmd::QuadTo(c, p) => points.extend([*c, *p]),
            PathCmd::CubicTo(c1, c2, p) => points.extend([*c1, *c2, *p]),
            PathCmd::ArcTo { to, .. } => points.push(*to),
            PathCmd::Arc {
                center,
                rx,
                ry,
                x_rotation_deg,
                start_deg,
                sweep_deg,
            } => {
                let arc = Ellipse {
                    center: *center,
                    rx: *rx,
                    ry: *ry,
                    rotation_deg: *x_rotation_deg,
                };
                for piece in arc_pieces(*start_deg, *sweep_deg) {
                    points.push(arc.point(piece));
                }
            }
            PathCmd::Close => {}
        }
    }
    let Some(first) = points.first() else {
        return Rect {
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
        };
    };
    let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x, first.y);
    for p in &points {
        x0 = x0.min(p.x);
        y0 = y0.min(p.y);
        x1 = x1.max(p.x);
        y1 = y1.max(p.y);
    }
    Rect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    }
}

/// The angles (degrees) where an arc from `start` sweeping `sweep` is cut:
/// the start, then the end of each piece.
fn arc_pieces(start: f64, sweep: f64) -> Vec<f64> {
    let sweep = sweep.clamp(-360.0, 360.0);
    let count = (sweep.abs() / ARC_PIECE_DEG).ceil().max(1.0) as usize;
    let step = sweep / count as f64;
    (0..=count).map(|k| start + step * k as f64).collect()
}

/// An ellipse of radii `rx` × `ry` at `center`, turned by `rotation_deg`.
#[derive(Clone, Copy)]
struct Ellipse {
    center: Point,
    rx: f64,
    ry: f64,
    rotation_deg: f64,
}

impl Ellipse {
    /// The point of the ellipse at the parametric angle `deg`.
    fn point(self, deg: f64) -> Point {
        let (sin_a, cos_a) = deg.to_radians().sin_cos();
        let (sin_r, cos_r) = self.rotation_deg.to_radians().sin_cos();
        let x = self.rx * cos_a;
        let y = self.ry * sin_a;
        Point {
            x: self.center.x + x * cos_r - y * sin_r,
            y: self.center.y + x * sin_r + y * cos_r,
        }
    }
}

/// SVG path data for `cmds`. Fails only if a value is not finite.
pub(super) fn path_data(cmds: &[PathCmd]) -> Result<String, SvgError> {
    let mut out = String::new();
    let mut current: Option<Point> = None;
    let mut start: Option<Point> = None;
    for cmd in cmds {
        match *cmd {
            PathCmd::MoveTo(p) => {
                push_segment(&mut out, format!("M{}", xy(p)));
                current = Some(p);
                start = Some(p);
            }
            PathCmd::LineTo(p) => {
                push_segment(&mut out, format!("L{}", xy(p)));
                current = Some(p);
            }
            PathCmd::QuadTo(c, p) => {
                push_segment(&mut out, format!("Q{} {}", xy(c), xy(p)));
                current = Some(p);
            }
            PathCmd::CubicTo(c1, c2, p) => {
                push_segment(&mut out, format!("C{} {} {}", xy(c1), xy(c2), xy(p)));
                current = Some(p);
            }
            PathCmd::ArcTo {
                rx,
                ry,
                x_rotation_deg,
                large_arc,
                sweep,
                to,
            } => {
                let segment = format!(
                    "A{} {} {} {} {} {}",
                    number(rx),
                    number(ry),
                    number(x_rotation_deg),
                    flag(large_arc),
                    flag(sweep),
                    xy(to)
                );
                push_segment(&mut out, segment);
                current = Some(to);
            }
            PathCmd::Arc {
                center,
                rx,
                ry,
                x_rotation_deg,
                start_deg,
                sweep_deg,
            } => {
                let arc = Ellipse {
                    center,
                    rx,
                    ry,
                    rotation_deg: x_rotation_deg,
                };
                // A line joins the current point to the arc's start; the arc
                // begins a contour only when there is no current point.
                let joins = if current.is_some() { "L" } else { "M" };
                current = Some(push_arc(&mut out, joins, arc, start_deg, sweep_deg));
            }
            PathCmd::Close => {
                push_segment(&mut out, "Z".to_owned());
                current = start;
            }
        }
    }
    // Non-finite values format as `NaN` or `inf`, which no other segment contains.
    if out.contains("NaN") || out.contains("inf") {
        return Err(SvgError::unsupported(
            "a path with a value that is not finite",
        ));
    }
    Ok(out)
}

/// Writes an arc as a move or line (`joins`) to its start, then one `A`
/// segment per piece. Returns the end point of the arc.
fn push_arc(out: &mut String, joins: &str, arc: Ellipse, start_deg: f64, sweep_deg: f64) -> Point {
    let angles = arc_pieces(start_deg, sweep_deg);
    push_segment(out, format!("{joins}{}", xy(arc.point(angles[0]))));
    let sweep_flag = flag(sweep_deg > 0.0);
    for &angle in &angles[1..] {
        // Pieces of at most a quarter turn never need the large arc.
        let to = arc.point(angle);
        let segment = format!(
            "A{} {} {} 0 {sweep_flag} {}",
            number(arc.rx),
            number(arc.ry),
            number(arc.rotation_deg),
            xy(to)
        );
        push_segment(out, segment);
    }
    arc.point(angles[angles.len() - 1])
}

/// Append one path segment, separated from the previous one by a space.
fn push_segment(out: &mut String, text: String) {
    if !out.is_empty() {
        out.push(' ');
    }
    out.push_str(&text);
}

fn xy(p: Point) -> String {
    format!("{} {}", number(p.x), number(p.y))
}

fn flag(on: bool) -> u8 {
    u8::from(on)
}

#[cfg(test)]
mod tests {
    //! Role: Tests of the SVG path data of lines, closed contours and arcs.

    use super::path_data;
    use crate::SvgError;
    use kivact_types::Point;
    use kivact_types::scene::PathCmd;

    fn at(x: f64, y: f64) -> Point {
        Point { x, y }
    }

    #[test]
    fn lines_and_a_close_write_their_segments_in_order() {
        let data = path_data(&[
            PathCmd::MoveTo(at(0.0, 0.0)),
            PathCmd::LineTo(at(10.0, 0.0)),
            PathCmd::LineTo(at(10.0, 5.0)),
            PathCmd::Close,
        ])
        .expect("finite lines write");

        assert_eq!(data, "M0 0 L10 0 L10 5 Z");
    }

    #[test]
    fn a_half_turn_arc_is_cut_into_two_quarter_turns() {
        let data = path_data(&[PathCmd::Arc {
            center: at(0.0, 0.0),
            rx: 1.0,
            ry: 1.0,
            x_rotation_deg: 0.0,
            start_deg: 0.0,
            sweep_deg: 180.0,
        }])
        .expect("a finite arc writes");

        assert_eq!(data, "M1 0 A1 1 0 0 1 0 1 A1 1 0 0 1 -1 0");
    }

    #[test]
    fn a_value_that_is_not_finite_is_unsupported() {
        let error =
            path_data(&[PathCmd::MoveTo(at(f64::NAN, 0.0))]).expect_err("NaN cannot be written");

        assert!(matches!(error, SvgError::Unsupported(_)));
    }
}
