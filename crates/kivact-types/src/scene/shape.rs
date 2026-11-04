//! Role: Paths, rounded rectangles and shapes in logical pixels.
//! Plain data usable on any thread.
//!
//! Every shape has one outline ([`Shape::outline`]: moves, lines, quadratic
//! and cubic curves only), which the renderer fills and strokes and
//! [`Shape::contains`] hit-tests, so what is drawn and what is hit agree.

use crate::geometry::{Point, Rect};
use crate::scene::primitives::FillRule;

/// Portable path segment, usable on any thread.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathCmd {
    /// Begin a contour.
    MoveTo(Point),
    /// Straight segment.
    LineTo(Point),
    /// Quadratic curve with control and endpoint.
    QuadTo(Point, Point),
    /// Cubic curve with two controls and endpoint.
    CubicTo(Point, Point, Point),
    /// Elliptical arc from the current point to `to`, as SVG's `A`
    /// command: an ellipse of radii `rx` × `ry` rotated by `x_rotation_deg`
    /// degrees, the larger or smaller of its two arcs (`large_arc`),
    /// clockwise on screen (`sweep`) or not. Radii too small to reach `to`
    /// grow as SVG says; a zero radius draws a line.
    ArcTo {
        /// Horizontal radius before rotation.
        rx: f32,
        /// Vertical radius before rotation.
        ry: f32,
        /// Rotation of the ellipse's x axis, in degrees.
        x_rotation_deg: f32,
        /// Take the arc of more than 180°.
        large_arc: bool,
        /// Go clockwise on screen.
        sweep: bool,
        /// Endpoint.
        to: Point,
    },
    /// An arc of the ellipse centered at `center` with radii `rx` × `ry`
    /// rotated by `x_rotation_deg` degrees, from the angle `start_deg`
    /// sweeping `sweep_deg` degrees (positive: clockwise on screen; at most
    /// a full turn is drawn), as canvas `ellipse` and `arc`: a line joins
    /// the current point to the arc's start, or the arc begins a contour.
    Arc {
        /// Center.
        center: Point,
        /// Horizontal radius before rotation.
        rx: f32,
        /// Vertical radius before rotation.
        ry: f32,
        /// Rotation of the ellipse's x axis, in degrees.
        x_rotation_deg: f32,
        /// Start angle in degrees (0: along the rotated x axis).
        start_deg: f32,
        /// Swept angle in degrees.
        sweep_deg: f32,
    },
    /// Close the contour.
    Close,
}

/// One corner's elliptical radii.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Radius {
    /// Horizontal radius.
    pub x: f32,
    /// Vertical radius.
    pub y: f32,
}

impl Radius {
    /// A circular corner.
    pub fn circular(r: f32) -> Self {
        Radius { x: r, y: r }
    }
}

/// A rectangle with elliptical corners (CSS `border-radius`, each corner
/// with its own horizontal and vertical radius).
///
/// Radii are used as CSS uses them: negative radii count as zero, and when
/// two adjacent radii add up to more than their side, all radii shrink by
/// the same factor until they fit.
///
/// ```
/// use kivact_types::{Point, Rect};
/// use kivact_types::scene::{Radius, RoundedRect};
/// // `border-radius: 50%` on a 100 × 40 box: an ellipse.
/// let ellipse = RoundedRect::ellipse(Rect { x: 0.0, y: 0.0, width: 100.0, height: 40.0 });
/// assert_eq!(ellipse.radii[0], Radius { x: 50.0, y: 20.0 });
/// assert!(ellipse.contains(Point { x: 50.0, y: 20.0 }));
/// assert!(!ellipse.contains(Point { x: 2.0, y: 2.0 }));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RoundedRect {
    /// Bounds.
    pub rect: Rect,
    /// Top-left, top-right, bottom-right, bottom-left radii.
    pub radii: [Radius; 4],
}

impl RoundedRect {
    /// `rect` with the same circular radius `r` at every corner.
    pub fn new(rect: Rect, r: f32) -> Self {
        RoundedRect {
            rect,
            radii: [Radius::circular(r); 4],
        }
    }

    /// `rect` with circular radii: top-left, top-right, bottom-right,
    /// bottom-left.
    pub fn with_corners(rect: Rect, radii: [f32; 4]) -> Self {
        RoundedRect {
            rect,
            radii: radii.map(Radius::circular),
        }
    }

    /// The ellipse inscribed in `rect` (`border-radius: 50%`).
    pub fn ellipse(rect: Rect) -> Self {
        RoundedRect {
            rect,
            radii: [Radius {
                x: rect.width / 2.0,
                y: rect.height / 2.0,
            }; 4],
        }
    }

    /// Whether every corner is circular (horizontal and vertical radii
    /// equal once fitted).
    pub fn is_circular(&self) -> bool {
        self.fitted_radii().iter().all(|r| r.x == r.y)
    }

    /// The radii as drawn: non-negative, shrunk by one factor so adjacent
    /// radii fit their side (CSS Backgrounds 3, "corner overlap").
    pub fn fitted_radii(&self) -> [Radius; 4] {
        let radii = self.radii.map(|r| Radius {
            x: positive(r.x),
            y: positive(r.y),
        });
        let (w, h) = (positive(self.rect.width), positive(self.rect.height));
        let ratio = |side: f32, sum: f32| if sum > side { side / sum } else { 1.0 };
        let f = ratio(w, radii[0].x + radii[1].x)
            .min(ratio(w, radii[3].x + radii[2].x))
            .min(ratio(h, radii[0].y + radii[3].y))
            .min(ratio(h, radii[1].y + radii[2].y));
        if f < 1.0 {
            radii.map(|r| Radius {
                x: r.x * f,
                y: r.y * f,
            })
        } else {
            radii
        }
    }

    /// The outline, clockwise from the end of the top-left corner: lines
    /// and one cubic per rounded corner.
    pub fn outline(&self) -> Vec<PathCmd> {
        let r = self.fitted_radii();
        let Rect {
            x,
            y,
            width: w,
            height: h,
        } = self.rect;
        let (x1, y1) = (x + w, y + h);
        let p = |x, y| Point { x, y };
        let mut out = Vec::with_capacity(10);
        out.push(PathCmd::MoveTo(p(x + r[0].x, y)));
        out.push(PathCmd::LineTo(p(x1 - r[1].x, y)));
        corner(&mut out, p(x1 - r[1].x, y), p(x1, y + r[1].y), p(x1, y));
        out.push(PathCmd::LineTo(p(x1, y1 - r[2].y)));
        corner(&mut out, p(x1, y1 - r[2].y), p(x1 - r[2].x, y1), p(x1, y1));
        out.push(PathCmd::LineTo(p(x + r[3].x, y1)));
        corner(&mut out, p(x + r[3].x, y1), p(x, y1 - r[3].y), p(x, y1));
        out.push(PathCmd::LineTo(p(x, y + r[0].y)));
        corner(&mut out, p(x, y + r[0].y), p(x + r[0].x, y), p(x, y));
        out.push(PathCmd::Close);
        out
    }

    /// Whether `p` is inside (corners excluded along their ellipses).
    pub fn contains(&self, p: Point) -> bool {
        if !self.rect.contains(p) {
            return false;
        }
        let r = self.fitted_radii();
        let Rect {
            x,
            y,
            width: w,
            height: h,
        } = self.rect;
        // Each corner's ellipse center, and whether p is in its corner box.
        let corners = [
            (
                x + r[0].x,
                y + r[0].y,
                r[0],
                p.x < x + r[0].x && p.y < y + r[0].y,
            ),
            (
                x + w - r[1].x,
                y + r[1].y,
                r[1],
                p.x > x + w - r[1].x && p.y < y + r[1].y,
            ),
            (
                x + w - r[2].x,
                y + h - r[2].y,
                r[2],
                p.x > x + w - r[2].x && p.y > y + h - r[2].y,
            ),
            (
                x + r[3].x,
                y + h - r[3].y,
                r[3],
                p.x < x + r[3].x && p.y > y + h - r[3].y,
            ),
        ];
        corners.iter().all(|(cx, cy, r, inside)| {
            if !inside || r.x <= 0.0 || r.y <= 0.0 {
                return true;
            }
            let (dx, dy) = ((p.x - cx) / r.x, (p.y - cy) / r.y);
            dx * dx + dy * dy <= 1.0
        })
    }
}

/// A quarter-ellipse corner from `from` to `to` whose bounding corner is
/// `at` (a cubic with the usual 0.5523 handle ratio), or nothing when the
/// corner is square.
fn corner(out: &mut Vec<PathCmd>, from: Point, to: Point, at: Point) {
    if from == to || from == at || to == at {
        if from != to {
            out.push(PathCmd::LineTo(to));
        }
        return;
    }
    const K: f32 = 0.552_284_8;
    let lerp = |a: Point, b: Point| Point {
        x: a.x + (b.x - a.x) * K,
        y: a.y + (b.y - a.y) * K,
    };
    out.push(PathCmd::CubicTo(lerp(from, at), lerp(to, at), to));
}

fn positive(v: f32) -> f32 {
    if v.is_finite() { v.max(0.0) } else { 0.0 }
}

/// A closed area to fill, stroke or clip to.
#[derive(Clone, Debug, PartialEq)]
pub enum Shape {
    /// A rectangle.
    Rect(Rect),
    /// A rectangle with elliptical corners.
    RoundedRect(RoundedRect),
    /// A closed polygon through `points` (CSS `clip-path: polygon(..)`).
    Polygon {
        /// Vertices in order.
        points: Vec<Point>,
        /// Which points the polygon encloses.
        rule: FillRule,
    },
    /// Any path; open contours are closed for filling and clipping.
    Path {
        /// Segments.
        cmds: Vec<PathCmd>,
        /// Which points the path encloses.
        rule: FillRule,
    },
}

impl Shape {
    /// The outline as moves, lines, quadratic and cubic curves and closes
    /// only (arcs become cubics): what renderers fill and stroke.
    pub fn outline(&self) -> Vec<PathCmd> {
        match self {
            Shape::Rect(r) => RoundedRect::new(*r, 0.0).outline(),
            Shape::RoundedRect(r) => r.outline(),
            Shape::Polygon { points, .. } => {
                let mut out: Vec<PathCmd> = points
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        if i == 0 {
                            PathCmd::MoveTo(*p)
                        } else {
                            PathCmd::LineTo(*p)
                        }
                    })
                    .collect();
                if !out.is_empty() {
                    out.push(PathCmd::Close);
                }
                out
            }
            Shape::Path { cmds, .. } => lower_arcs(cmds),
        }
    }

    /// The fill rule its outline is filled with.
    pub fn rule(&self) -> FillRule {
        match self {
            Shape::Rect(_) | Shape::RoundedRect(_) => FillRule::NonZero,
            Shape::Polygon { rule, .. } | Shape::Path { rule, .. } => *rule,
        }
    }

    /// Bounds holding the whole shape (control points included, so curves
    /// may make them a little larger than the ink); zero at the origin when
    /// empty.
    pub fn bounds(&self) -> Rect {
        match self {
            Shape::Rect(r) => *r,
            Shape::RoundedRect(r) => r.rect,
            _ => points_bounds(self.outline().iter().flat_map(cmd_points)),
        }
    }

    /// Whether `p` is inside, by the shape's fill rule (curves are measured
    /// along 16 straight pieces each).
    pub fn contains(&self, p: Point) -> bool {
        match self {
            Shape::Rect(r) => r.contains(p),
            Shape::RoundedRect(r) => r.contains(p),
            _ => {
                let winding = winding(&self.outline(), p);
                match self.rule() {
                    FillRule::NonZero => winding != 0,
                    FillRule::EvenOdd => winding % 2 != 0,
                }
            }
        }
    }
}

/// The points a segment names.
fn cmd_points(cmd: &PathCmd) -> impl Iterator<Item = Point> {
    let points: [Option<Point>; 3] = match *cmd {
        PathCmd::MoveTo(p) | PathCmd::LineTo(p) => [Some(p), None, None],
        PathCmd::QuadTo(a, b) => [Some(a), Some(b), None],
        PathCmd::CubicTo(a, b, c) => [Some(a), Some(b), Some(c)],
        PathCmd::ArcTo { to, .. } => [Some(to), None, None],
        PathCmd::Arc { .. } | PathCmd::Close => [None, None, None],
    };
    points.into_iter().flatten()
}

pub(crate) fn points_bounds(points: impl Iterator<Item = Point>) -> Rect {
    let mut bounds: Option<(f32, f32, f32, f32)> = None;
    for p in points {
        bounds = Some(match bounds {
            None => (p.x, p.y, p.x, p.y),
            Some((x0, y0, x1, y1)) => (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y)),
        });
    }
    let (x0, y0, x1, y1) = bounds.unwrap_or_default();
    Rect {
        x: x0,
        y: y0,
        width: x1 - x0,
        height: y1 - y0,
    }
}

/// `cmds` with arcs replaced by cubics.
fn lower_arcs(cmds: &[PathCmd]) -> Vec<PathCmd> {
    if !cmds
        .iter()
        .any(|c| matches!(c, PathCmd::ArcTo { .. } | PathCmd::Arc { .. }))
    {
        return cmds.to_vec();
    }
    let mut out = Vec::with_capacity(cmds.len() + 8);
    let mut current: Option<Point> = None;
    let mut start = Point::default();
    for cmd in cmds {
        match *cmd {
            PathCmd::MoveTo(p) => {
                current = Some(p);
                start = p;
                out.push(*cmd);
            }
            PathCmd::LineTo(p) | PathCmd::QuadTo(_, p) | PathCmd::CubicTo(_, _, p) => {
                current = Some(p);
                out.push(*cmd);
            }
            PathCmd::Close => {
                current = current.map(|_| start);
                out.push(*cmd);
            }
            PathCmd::ArcTo {
                rx,
                ry,
                x_rotation_deg,
                large_arc,
                sweep,
                to,
            } => {
                let Some(from) = current else {
                    // No contour to continue: begin one at the endpoint.
                    out.push(PathCmd::MoveTo(to));
                    current = Some(to);
                    start = to;
                    continue;
                };
                svg_arc(&mut out, from, rx, ry, x_rotation_deg, large_arc, sweep, to);
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
                let arc = Ellipse::new(center, rx.abs(), ry.abs(), x_rotation_deg);
                let a0 = start_deg.to_radians();
                let sweep = sweep_deg
                    .to_radians()
                    .clamp(-std::f32::consts::TAU, std::f32::consts::TAU);
                let first = arc.at(a0);
                if current.is_some() {
                    out.push(PathCmd::LineTo(first));
                } else {
                    out.push(PathCmd::MoveTo(first));
                    start = first;
                }
                current = Some(arc.cubics(&mut out, a0, sweep));
            }
        }
    }
    out
}

/// An ellipse: center, radii and rotation (radians' sine and cosine).
struct Ellipse {
    center: Point,
    rx: f32,
    ry: f32,
    sin: f32,
    cos: f32,
}

impl Ellipse {
    fn new(center: Point, rx: f32, ry: f32, rotation_deg: f32) -> Self {
        let (sin, cos) = rotation_deg.to_radians().sin_cos();
        Ellipse {
            center,
            rx,
            ry,
            sin,
            cos,
        }
    }

    /// The point at parameter angle `t` (radians).
    fn at(&self, t: f32) -> Point {
        let (s, c) = t.sin_cos();
        let (x, y) = (self.rx * c, self.ry * s);
        Point {
            x: self.center.x + x * self.cos - y * self.sin,
            y: self.center.y + x * self.sin + y * self.cos,
        }
    }

    /// The derivative at `t` (radians).
    fn tangent(&self, t: f32) -> Point {
        let (s, c) = t.sin_cos();
        let (x, y) = (-self.rx * s, self.ry * c);
        Point {
            x: x * self.cos - y * self.sin,
            y: x * self.sin + y * self.cos,
        }
    }

    /// Cubics along the arc from `t0` sweeping `sweep` radians, at most a
    /// quarter turn each; returns the end point.
    fn cubics(&self, out: &mut Vec<PathCmd>, t0: f32, sweep: f32) -> Point {
        let pieces = (sweep.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0) as usize;
        let step = sweep / pieces as f32;
        let k = 4.0 / 3.0 * (step / 4.0).tan();
        let mut t = t0;
        let mut p0 = self.at(t);
        for _ in 0..pieces {
            let t1 = t + step;
            let p1 = self.at(t1);
            let (d0, d1) = (self.tangent(t), self.tangent(t1));
            out.push(PathCmd::CubicTo(
                Point {
                    x: p0.x + k * d0.x,
                    y: p0.y + k * d0.y,
                },
                Point {
                    x: p1.x - k * d1.x,
                    y: p1.y - k * d1.y,
                },
                p1,
            ));
            p0 = p1;
            t = t1;
        }
        p0
    }
}

/// An SVG arc from `from` to `to` as cubics (SVG 1.1, F.6.5 and F.6.6).
#[expect(clippy::too_many_arguments, reason = "SVG's arc parameters")]
fn svg_arc(
    out: &mut Vec<PathCmd>,
    from: Point,
    rx: f32,
    ry: f32,
    x_rotation_deg: f32,
    large_arc: bool,
    sweep: bool,
    to: Point,
) {
    if from == to {
        return;
    }
    let (mut rx, mut ry) = (rx.abs(), ry.abs());
    if rx == 0.0 || ry == 0.0 {
        out.push(PathCmd::LineTo(to));
        return;
    }
    let (sin, cos) = x_rotation_deg.to_radians().sin_cos();
    let (dx, dy) = ((from.x - to.x) / 2.0, (from.y - to.y) / 2.0);
    let x1 = cos * dx + sin * dy;
    let y1 = -sin * dx + cos * dy;
    let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx *= s;
        ry *= s;
    }
    let num = (rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1).max(0.0);
    let den = rx * rx * y1 * y1 + ry * ry * x1 * x1;
    let mut coef = if den > 0.0 { (num / den).sqrt() } else { 0.0 };
    if large_arc == sweep {
        coef = -coef;
    }
    let cx1 = coef * rx * y1 / ry;
    let cy1 = -coef * ry * x1 / rx;
    let center = Point {
        x: cos * cx1 - sin * cy1 + (from.x + to.x) / 2.0,
        y: sin * cx1 + cos * cy1 + (from.y + to.y) / 2.0,
    };
    let angle = |ux: f32, uy: f32, vx: f32, vy: f32| {
        let dot = ux * vx + uy * vy;
        let len = (ux * ux + uy * uy).sqrt() * (vx * vx + vy * vy).sqrt();
        let a = (dot / len).clamp(-1.0, 1.0).acos();
        if ux * vy - uy * vx < 0.0 { -a } else { a }
    };
    let (ux, uy) = ((x1 - cx1) / rx, (y1 - cy1) / ry);
    let (vx, vy) = ((-x1 - cx1) / rx, (-y1 - cy1) / ry);
    let t0 = angle(1.0, 0.0, ux, uy);
    let mut delta = angle(ux, uy, vx, vy);
    let tau = std::f32::consts::TAU;
    if !sweep && delta > 0.0 {
        delta -= tau;
    } else if sweep && delta < 0.0 {
        delta += tau;
    }
    let ellipse = Ellipse {
        center,
        rx,
        ry,
        sin,
        cos,
    };
    ellipse.cubics(out, t0, delta);
    // End exactly on the endpoint, as given.
    if let Some(PathCmd::CubicTo(_, _, end)) = out.last_mut() {
        *end = to;
    }
}

/// How many times `outline` winds around `p` (curves split into 16
/// lines each; open contours closed).
fn winding(outline: &[PathCmd], p: Point) -> i32 {
    let mut winding = 0;
    let mut start = Point::default();
    let mut current = Point::default();
    let mut edge = |a: Point, b: Point| {
        if a.y <= p.y {
            if b.y > p.y && cross(a, b, p) > 0.0 {
                winding += 1;
            }
        } else if b.y <= p.y && cross(a, b, p) < 0.0 {
            winding -= 1;
        }
    };
    const PIECES: usize = 16;
    for cmd in outline {
        match *cmd {
            PathCmd::MoveTo(q) => {
                edge(current, start);
                start = q;
                current = q;
            }
            PathCmd::LineTo(q) => {
                edge(current, q);
                current = q;
            }
            PathCmd::QuadTo(c, q) => {
                let mut prev = current;
                for i in 1..=PIECES {
                    let t = i as f32 / PIECES as f32;
                    let u = 1.0 - t;
                    let next = Point {
                        x: u * u * current.x + 2.0 * u * t * c.x + t * t * q.x,
                        y: u * u * current.y + 2.0 * u * t * c.y + t * t * q.y,
                    };
                    edge(prev, next);
                    prev = next;
                }
                current = q;
            }
            PathCmd::CubicTo(c1, c2, q) => {
                let mut prev = current;
                for i in 1..=PIECES {
                    let t = i as f32 / PIECES as f32;
                    let u = 1.0 - t;
                    let next = Point {
                        x: u * u * u * current.x
                            + 3.0 * u * u * t * c1.x
                            + 3.0 * u * t * t * c2.x
                            + t * t * t * q.x,
                        y: u * u * u * current.y
                            + 3.0 * u * u * t * c1.y
                            + 3.0 * u * t * t * c2.y
                            + t * t * t * q.y,
                    };
                    edge(prev, next);
                    prev = next;
                }
                current = q;
            }
            PathCmd::Close => {
                edge(current, start);
                current = start;
            }
            PathCmd::ArcTo { to, .. } => {
                edge(current, to);
                current = to;
            }
            PathCmd::Arc { .. } => {}
        }
    }
    edge(current, start);
    winding
}

/// Positive when `p` is left of `a → b` (y down).
fn cross(a: Point, b: Point, p: Point) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y)
}
