//! Role: Resolves drawn shapes to path commands and writes SVG path data.
//!
//! A shape becomes one list of path commands in the drawing's coordinates
//! (the same outlines the window fills). Elliptical arcs of a path become SVG
//! `A` commands; the arcs of a `PathCmd::Arc` are cut into pieces of at most a
//! quarter turn, so a full turn is exact.

use super::encode::number;
use crate::SvgError;
use kivact_types::scene::{PathCmd, Shape};
use kivact_types::{DrawingShape, FillRule, Point};

/// Pieces of a `PathCmd::Arc` are at most this many degrees.
const ARC_PIECE_DEG: f64 = 90.0;

/// The path commands and fill rule of a drawn shape, the outline the
/// window fills (`DrawingShape::to_shape`).
pub(super) fn outline_of(shape: &DrawingShape) -> (Vec<PathCmd>, FillRule) {
    let outline = shape.to_shape();
    let rule = outline.rule();
    match outline {
        Shape::Path { cmds, .. } => (cmds, rule),
        other => (other.outline(), rule),
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
