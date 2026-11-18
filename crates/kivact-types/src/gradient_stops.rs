//! Role: Expand a CSS gradient's authored stops into sRGB stops for engine and SVG.
use crate::color::Rgba;
use crate::color_space::{ColorInterpolation, SpaceColor, interpolate};
use crate::geometry::{Point, Rect};
use crate::paint::{Gradient, GradientKind, Len};

/// The deepest bisection of a gradient interval (at most 4096 pieces).
const MAX_DEPTH: u32 = 12;

/// A position `l` as a fraction of a line `length` px long (a conic
/// gradient's: 360°); a non-finite one is zero.
fn fraction(l: Len, length: f64) -> f64 {
    let t = if length > 0.0 {
        l.px / length + l.pct / 100.0
    } else {
        l.pct / 100.0
    };
    if t.is_finite() { t } else { 0.0 }
}

impl Gradient {
    /// The stops of this gradient as fractions of its line (CSS color stop
    /// fixup), with its color hints expanded (see [`expand_gradient`]):
    /// `line_length` is the line's length in px (a conic gradient's: 360°).
    pub fn srgb_stops(&self, line_length: f64) -> Vec<(f64, Rgba)> {
        let n = self.stops.len();
        let mut at: Vec<Option<f64>> = self
            .stops
            .iter()
            .map(|s| s.at.map(|l| fraction(l, line_length)))
            .collect();
        if n == 0 {
            return Vec::new();
        }
        if at[0].is_none() {
            at[0] = Some(0.0);
        }
        if at[n - 1].is_none() {
            at[n - 1] = Some(1.0);
        }
        // No position before an earlier one.
        let mut max = f64::NEG_INFINITY;
        for t in at.iter_mut().flatten() {
            max = max.max(*t);
            *t = max;
        }
        // Missing positions spread evenly between their neighbours.
        let mut i = 0;
        while i < n {
            if at[i].is_some() {
                i += 1;
                continue;
            }
            let start = i - 1;
            let end = (i..n).find(|j| at[*j].is_some()).unwrap_or(n - 1);
            let (a, b) = (at[start].unwrap_or(0.0), at[end].unwrap_or(1.0));
            let span = (end - start) as f64;
            for (k, slot) in at.iter_mut().enumerate().take(end).skip(i) {
                *slot = Some(a + (b - a) * (k - start) as f64 / span);
            }
            i = end;
        }
        let resolved: Vec<(f64, SpaceColor)> = self
            .stops
            .iter()
            .zip(at)
            .map(|(s, t)| (t.unwrap_or(0.0), s.color))
            .collect();
        let hints: Vec<Option<f64>> = self
            .hints
            .iter()
            .map(|h| h.map(|l| fraction(l, line_length)))
            .collect();
        expand_gradient(&resolved, &hints, self.interpolation)
    }

    /// The line of a linear gradient over `rect` (CSS: 0 points up, 90
    /// right; long enough for the corners to take the end colors) as
    /// its start and end; `None` for another kind. A non-finite angle
    /// is 180 (downwards).
    pub fn line(&self, rect: Rect) -> Option<(Point, Point)> {
        let GradientKind::Linear { angle_deg } = self.kind else {
            return None;
        };
        let angle = if angle_deg.is_finite() {
            angle_deg
        } else {
            180.0
        };
        let (dy, dx) = (angle - 90.0).to_radians().sin_cos();
        let span = rect.width * dx.abs() + rect.height * dy.abs();
        let (cx, cy) = (rect.x + rect.width / 2.0, rect.y + rect.height / 2.0);
        Some((
            Point {
                x: cx - dx * span / 2.0,
                y: cy - dy * span / 2.0,
            },
            Point {
                x: cx + dx * span / 2.0,
                y: cy + dy * span / 2.0,
            },
        ))
    }
}

/// The sRGB stops of a gradient whose authored stops are `stops` (offsets
/// in order) with color `hints` between them (`hints[i]` between stop i and
/// i + 1, an absolute offset; a missing hint is the midpoint). The colors
/// are interpolated in `how` and mapped to sRGB; where that ramp drifts from
/// the sRGB line between its stops, stops are added. Two stops at one
/// offset make a hard edge.
pub fn expand_gradient(
    stops: &[(f64, SpaceColor)],
    hints: &[Option<f64>],
    how: ColorInterpolation,
) -> Vec<(f64, Rgba)> {
    let Some((p, c)) = stops.first().copied() else {
        return Vec::new();
    };
    let mut out = vec![(p, c.to_srgb().sanitize())];
    for (i, pair) in stops.windows(2).enumerate() {
        let hint = hints.get(i).copied().flatten();
        expand_interval(&mut out, pair[0], pair[1], hint, how);
    }
    out
}

/// Append the stops of the interval from `a` to `b` (both authored), ending
/// at `b`; `hint` is its color hint's absolute offset.
fn expand_interval(
    out: &mut Vec<(f64, Rgba)>,
    a: (f64, SpaceColor),
    b: (f64, SpaceColor),
    hint: Option<f64>,
    how: ColorInterpolation,
) {
    let ((p0, c0), (p1, c1)) = (a, b);
    let end = c1.to_srgb().sanitize();
    let span = p1 - p0;
    if span.is_nan() || span <= 0.0 {
        out.push((p1, end));
        return;
    }
    let h = hint.map_or(0.5, |h| (h - p0) / span);
    if h == 0.0 {
        // The hint is at the start: the second color at once.
        out.push((p0, end));
        out.push((p1, end));
        return;
    }
    if h == 1.0 {
        // The hint is at the end: the first color to the end.
        out.push((p1, c0.to_srgb().sanitize()));
        out.push((p1, end));
        return;
    }
    let h = if h.is_nan() {
        0.5
    } else {
        h.clamp(0.001, 0.999)
    };
    // CSS's hint: the weight of the second color is t^(ln 0.5 / ln h), so
    // it is one half at the hint.
    let exponent = 0.5f64.ln() / h.ln();
    let ramp = |t: f64| {
        interpolate(c0, c1, t.powf(exponent), how)
            .to_srgb()
            .sanitize()
    };
    let (f_hint, f_end) = (ramp(h), ramp(1.0));
    let place = (p0, span);
    refine(out, &ramp, place, (0.0, ramp(0.0)), (h, f_hint), 0);
    refine(out, &ramp, place, (h, f_hint), (1.0, f_end), 0);
}

/// Bisect the ramp between `a` and `b` (offsets within the interval) until
/// its midpoint is within half a sRGB step of the premultiplied mix of the
/// ends, pushing each piece's end at its place `(start, length)` on the line.
fn refine(
    out: &mut Vec<(f64, Rgba)>,
    ramp: &impl Fn(f64) -> Rgba,
    place: (f64, f64),
    a: (f64, Rgba),
    b: (f64, Rgba),
    depth: u32,
) {
    let m = (a.0 + b.0) / 2.0;
    let mid = ramp(m);
    let linear = color_at(&[(0.0, a.1), (1.0, b.1)], 0.5);
    if depth >= MAX_DEPTH || within_step(mid, linear) {
        out.push((place.0 + b.0 * place.1, b.1));
        return;
    }
    refine(out, ramp, place, a, (m, mid), depth + 1);
    refine(out, ramp, place, (m, mid), b, depth + 1);
}

/// Whether every channel of `a` and `b` differs by at most half a step of
/// an 8-bit channel.
fn within_step(a: Rgba, b: Rgba) -> bool {
    const TOLERANCE: f64 = 0.5 / 255.0;
    [(a.r, b.r), (a.g, b.g), (a.b, b.b), (a.a, b.a)]
        .iter()
        .all(|(x, y)| (x - y).abs() <= TOLERANCE)
}

/// Stops spanning `[lo, hi]` mapped onto `[0, 1]`.
pub fn normalized_stops(stops: &[(f64, Rgba)], lo: f64, hi: f64) -> Vec<(f64, Rgba)> {
    let span = (hi - lo).max(f64::MIN_POSITIVE);
    stops
        .iter()
        .map(|(t, c)| (((t - lo) / span).clamp(0.0, 1.0), *c))
        .collect()
}

/// The color of the stops at `t` (premultiplied interpolation).
pub fn color_at(stops: &[(f64, Rgba)], t: f64) -> Rgba {
    let Some(first) = stops.first() else {
        return Rgba::default();
    };
    if t <= first.0 {
        return first.1;
    }
    for pair in stops.windows(2) {
        let ((t0, c0), (t1, c1)) = (pair[0], pair[1]);
        if t <= t1 {
            let k = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
            let a = c0.a + (c1.a - c0.a) * k;
            let mix = |x: f64, y: f64| {
                if a > 0.0 {
                    (x * c0.a + (y * c1.a - x * c0.a) * k) / a
                } else {
                    x + (y - x) * k
                }
            };
            return Rgba {
                r: mix(c0.r, c1.r),
                g: mix(c0.g, c1.g),
                b: mix(c0.b, c1.b),
                a,
            };
        }
    }
    stops.last().map_or(first.1, |l| l.1)
}
