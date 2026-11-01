//! Role: Path geometry and coordinate operations.
//!
//! Curve geometry of a [`Path`]: arcs become cubics once, and everything
//! else (transforming, measuring, bounding) works on the curves themselves.
//! kurbo stays behind this module; none of its types is public.
use super::Path;
use crate::geometry::{Point, Rect, Size};
use crate::transform::Transform;
use kivact_wire::{PathCmd, Point as WirePoint};
use kurbo::{Arc, BezPath, ParamCurve, ParamCurveExtrema, PathEl, Point as KPoint, SvgArc, Vec2};
use std::f64::consts::TAU;

/// One flattened contour, with closure represented separately from its points.
#[derive(Clone, Debug, PartialEq)]
pub struct PathContour {
    /// The contour's points, without a repeated start point for closure.
    pub points: Vec<Point>,
    /// Whether the last point connects back to the first.
    pub closed: bool,
}

/// The largest distance, in px, between an arc and its cubics (more for a
/// huge radius, so the count of cubics stays bounded).
const ARC_TOLERANCE: f64 = 0.01;
pub(super) fn kp(p: WirePoint) -> KPoint {
    KPoint::new(f64::from(p.x), f64::from(p.y))
}

pub(super) fn wire(p: KPoint) -> WirePoint {
    WirePoint {
        x: p.x as f32,
        y: p.y as f32,
    }
}

/// The cubics of `arc` after `current`, which they move on.
fn cubics(arc: Arc, out: &mut Vec<PathCmd>, current: &mut Option<KPoint>) {
    let tolerance = ARC_TOLERANCE.max(arc.radii.x.max(arc.radii.y) * 1e-6);
    for el in arc.append_iter(tolerance) {
        if let PathEl::CurveTo(a, b, c) = el {
            out.push(PathCmd::CubicTo(wire(a), wire(b), wire(c)));
            *current = Some(c);
        }
    }
}

impl Path {
    /// Approximate each contour with line segments in logical pixels.
    ///
    /// `tolerance` controls the flattening error; non-positive or non-finite
    /// values use 0.25. Closed contours omit the repeated start point and set
    /// [`PathContour::closed`]. Empty or invalid paths return no contours.
    pub fn flattened(&self, tolerance: f32) -> Vec<PathContour> {
        let Some(path) = self.bez() else {
            return Vec::new();
        };
        let tolerance = if tolerance.is_finite() && tolerance > 0.0 {
            f64::from(tolerance)
        } else {
            0.25
        };
        let mut contours = Vec::new();
        let mut points = Vec::new();
        let mut start = None;
        let flush = |points: &mut Vec<WirePoint>, closed, contours: &mut Vec<PathContour>| {
            if closed && points.len() > 1 && points.first() == points.last() {
                points.pop();
            }
            if !points.is_empty() {
                contours.push(PathContour {
                    points: std::mem::take(points),
                    closed,
                });
            }
        };
        kurbo::flatten(path, tolerance, |element| match element {
            PathEl::MoveTo(point) => {
                flush(&mut points, false, &mut contours);
                start = Some(wire(point));
                points.push(wire(point));
            }
            PathEl::LineTo(point) => {
                if points.is_empty()
                    && let Some(start) = start
                {
                    points.push(start);
                }
                points.push(wire(point));
            }
            PathEl::ClosePath => flush(&mut points, true, &mut contours),
            // kurbo::flatten emits only moves, lines and closes.
            PathEl::QuadTo(..) | PathEl::CurveTo(..) => {}
        });
        flush(&mut points, false, &mut contours);
        contours
    }

    /// The commands with every arc replaced by cubics, as the renderer
    /// lowers them: a segment without a current point is dropped, an arc
    /// begins a contour when none is open, and an SVG arc to its own start
    /// point draws nothing.
    pub(super) fn lowered(&self) -> Vec<PathCmd> {
        let mut out = Vec::with_capacity(self.cmds.len());
        let mut current: Option<KPoint> = None;
        let mut start = KPoint::ZERO;
        for cmd in &self.cmds {
            match *cmd {
                PathCmd::MoveTo(p) => {
                    start = kp(p);
                    current = Some(start);
                    out.push(*cmd);
                }
                PathCmd::LineTo(p) | PathCmd::QuadTo(_, p) | PathCmd::CubicTo(_, _, p)
                    if current.is_some() =>
                {
                    current = Some(kp(p));
                    out.push(*cmd);
                }
                PathCmd::Close if current.is_some() => {
                    current = Some(start);
                    out.push(*cmd);
                }
                PathCmd::Arc {
                    center,
                    rx,
                    ry,
                    x_rotation_deg: rotation,
                    start_deg: from_deg,
                    sweep_deg: sweep,
                } => {
                    if ![center.x, center.y, rx, ry, rotation, from_deg, sweep]
                        .iter()
                        .all(|v| v.is_finite())
                    {
                        continue;
                    }
                    let arc = Arc::new(
                        kp(center),
                        Vec2::new(f64::from(rx.abs()), f64::from(ry.abs())),
                        f64::from(from_deg).to_radians(),
                        f64::from(sweep).to_radians().clamp(-TAU, TAU),
                        f64::from(rotation).to_radians(),
                    );
                    let first = arc.eval(0.0);
                    if current.is_some() {
                        out.push(PathCmd::LineTo(wire(first)));
                    } else {
                        out.push(PathCmd::MoveTo(wire(first)));
                        start = first;
                    }
                    current = Some(first);
                    cubics(arc, &mut out, &mut current);
                }
                PathCmd::ArcTo {
                    rx,
                    ry,
                    x_rotation_deg: rotation,
                    large_arc,
                    sweep,
                    to,
                } => {
                    if ![rx, ry, rotation, to.x, to.y].iter().all(|v| v.is_finite()) {
                        continue;
                    }
                    let end = kp(to);
                    let Some(from) = current else {
                        out.push(PathCmd::MoveTo(to));
                        start = end;
                        current = Some(end);
                        continue;
                    };
                    if from == end {
                        continue;
                    }
                    let svg = SvgArc {
                        from,
                        to: end,
                        radii: Vec2::new(f64::from(rx), f64::from(ry)),
                        x_rotation: f64::from(rotation).to_radians(),
                        large_arc,
                        sweep,
                    };
                    match Arc::from_svg_arc(&svg) {
                        Some(arc) => cubics(arc, &mut out, &mut current),
                        None => out.push(PathCmd::LineTo(to)),
                    }
                    current = Some(end);
                }
                _ => {}
            }
        }
        out
    }

    /// The path as curves, `None` when it has no command or a point that is
    /// not finite.
    pub(super) fn bez(&self) -> Option<BezPath> {
        let mut path = BezPath::new();
        for cmd in self.lowered() {
            match cmd {
                PathCmd::MoveTo(p) => path.move_to(kp(p)),
                PathCmd::LineTo(p) => path.line_to(kp(p)),
                PathCmd::QuadTo(a, b) => path.quad_to(kp(a), kp(b)),
                PathCmd::CubicTo(a, b, c) => path.curve_to(kp(a), kp(b), kp(c)),
                PathCmd::Close => path.close_path(),
                // Lowered away.
                PathCmd::Arc { .. } | PathCmd::ArcTo { .. } => {}
            }
        }
        (!path.elements().is_empty() && path.elements().iter().all(PathEl::is_finite))
            .then_some(path)
    }

    /// A path of the curves `path`.
    pub(super) fn from_bez(path: &BezPath, even_odd: bool) -> Path {
        let cmds = path
            .elements()
            .iter()
            .map(|el| match *el {
                PathEl::MoveTo(p) => PathCmd::MoveTo(wire(p)),
                PathEl::LineTo(p) => PathCmd::LineTo(wire(p)),
                PathEl::QuadTo(a, b) => PathCmd::QuadTo(wire(a), wire(b)),
                PathEl::CurveTo(a, b, c) => PathCmd::CubicTo(wire(a), wire(b), wire(c)),
                PathEl::ClosePath => PathCmd::Close,
            })
            .collect();
        Path { cmds, even_odd }
    }

    /// The contours of `other` after this path's (HTML `Path2D.addPath`).
    /// `other` starts its own contour, whatever this path left open, and
    /// the result keeps this path's fill rule.
    ///
    /// ```
    /// use kivact::draw::{Path, Rect};
    /// let a = Path::new().rect(Rect::new(0.0, 0.0, 10.0, 10.0));
    /// let b = Path::new().rect(Rect::new(20.0, 0.0, 10.0, 10.0));
    /// assert_eq!(a.append(&b).bounds(), Some(Rect::new(0.0, 0.0, 30.0, 10.0)));
    /// ```
    pub fn append(&self, other: &Path) -> Path {
        let mut out = self.clone();
        if matches!(other.cmds.first(), Some(PathCmd::MoveTo(_))) {
            out.cmds.extend_from_slice(&other.cmds);
        } else {
            // A leading arc begins its own contour; a leading line has no
            // current point and draws nothing.
            out.cmds.extend(other.lowered());
        }
        out
    }

    /// This path with every point mapped by `t`. Curves stay curves (arcs
    /// and ellipses become cubics first), nothing is flattened, and the fill
    /// rule is kept. A percentage in `t`'s translation is of an empty box,
    /// so it is zero: use px.
    ///
    /// ```
    /// use kivact::draw::{Path, Rect};
    /// use kivact::draw::Transform;
    /// let square = Path::new().rect(Rect::new(0.0, 0.0, 10.0, 10.0));
    /// let moved = square.transformed(Transform::new().translate(5, 7).scale(2));
    /// assert_eq!(moved.bounds(), Some(Rect::new(5.0, 7.0, 20.0, 20.0)));
    /// ```
    pub fn transformed(&self, t: Transform) -> Path {
        let [a, b, c, d, e, f] = t.matrix_for(Size::default()).map(f64::from);
        let map = |p: WirePoint| {
            let (x, y) = (f64::from(p.x), f64::from(p.y));
            wire(KPoint::new(a * x + c * y + e, b * x + d * y + f))
        };
        let cmds = self
            .lowered()
            .into_iter()
            .map(|cmd| match cmd {
                PathCmd::MoveTo(p) => PathCmd::MoveTo(map(p)),
                PathCmd::LineTo(p) => PathCmd::LineTo(map(p)),
                PathCmd::QuadTo(p, q) => PathCmd::QuadTo(map(p), map(q)),
                PathCmd::CubicTo(p, q, r) => PathCmd::CubicTo(map(p), map(q), map(r)),
                other => other,
            })
            .collect();
        Path {
            cmds,
            even_odd: self.even_odd,
        }
    }

    /// The smallest rectangle holding the curves themselves (a circle's
    /// bounds are its diameter, not its control points'). `None` when the
    /// path has no segment: it is empty, only moves, or not finite.
    ///
    /// ```
    /// use kivact::draw::{Path, Point};
    /// let circle = Path::new().arc(Point::new(50.0, 50.0), 10, 0, 360, false);
    /// let b = circle.bounds().unwrap();
    /// assert!((b.x - 40.0).abs() < 0.01 && (b.width - 20.0).abs() < 0.01);
    /// assert_eq!(Path::new().bounds(), None);
    /// ```
    pub fn bounds(&self) -> Option<Rect> {
        let bez = self.bez()?;
        let r = bez
            .segments()
            .map(|segment| segment.bounding_box())
            .reduce(|a, b| a.union(b))?;
        let (x, y) = (r.x0 as f32, r.y0 as f32);
        Some(Rect::new(x, y, r.x1 as f32 - x, r.y1 as f32 - y))
    }
}
