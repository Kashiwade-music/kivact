//! Role: Cubic easing control-point values and their finite normalization.
use kivact::draw::Point;
/// A cubic easing curve with fixed endpoints (0, 0) and (1, 1).
/// X controls are clamped to 0..=1; finite Y controls may overshoot that range.
/// Built with [`CubicBezier::new`].
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub struct CubicBezier {
    /// First control point horizontal coordinate, normalized to 0..=1.
    pub x1: f64,
    /// First control point progress; finite overshoot is permitted.
    pub y1: f64,
    /// Second control point horizontal coordinate, normalized to 0..=1.
    pub x2: f64,
    /// Second control point progress; finite overshoot is permitted.
    pub y2: f64,
}
impl Default for CubicBezier {
    fn default() -> Self {
        Self {
            x1: 0.25,
            y1: 0.1,
            x2: 0.25,
            y2: 1.0,
        }
    }
}
impl CubicBezier {
    /// Progress at normalized time, retaining finite control-point overshoot.
    /// Time is clamped to 0..=1; NaN and negative infinity use 0, positive
    /// infinity uses 1. Invalid controls use the same defaults as [`Self::new`].
    pub fn progress(&self, time: f64) -> f64 {
        let time = if time.is_nan() {
            0.0
        } else {
            time.clamp(0.0, 1.0)
        };
        if time == 0.0 || time == 1.0 {
            return time;
        }
        let curve = self.clean();
        // Bernstein weights stay bounded even for very large finite controls.
        let coordinate = |t: f64, a: f64, b: f64| {
            let u = 1.0 - t;
            (3.0 * u * u * t) * a + (3.0 * u * t * t) * b + t * t * t
        };
        // The clean X curve is monotonic, so bisection finds the parameter whose
        // X equals `time` to well within 1e-6.
        let (mut low, mut high) = (0.0, 1.0);
        let mut t = 0.5;
        for _ in 0..60 {
            t = (low + high) / 2.0;
            let x = coordinate(t, curve.x1, curve.x2);
            if x == time {
                break;
            }
            if x < time {
                low = t;
            } else {
                high = t;
            }
        }
        coordinate(t, curve.y1, curve.y2)
    }

    /// Sample 2..=513 evenly spaced times, including both exact endpoints.
    /// Progress is clamped to the finite `f32` range when creating points.
    pub fn samples(&self, count: usize) -> Vec<Point> {
        let count = count.clamp(2, 513);
        (0..count)
            .map(|i| {
                let time = i as f64 / (count - 1) as f64;
                Point::new(
                    time as f32,
                    self.progress(time)
                        .clamp(-f64::from(f32::MAX), f64::from(f32::MAX))
                        as f32,
                )
            })
            .collect()
    }

    /// Construct a curve. NaN X and non-finite Y use the corresponding default;
    /// infinite X clamps to an endpoint. Finite Y is retained, including overshoot.
    pub fn new(x1: f64, y1: f64, x2: f64, y2: f64) -> Self {
        let default = Self::default();
        let x = |v: f64, fallback| {
            if v.is_nan() {
                fallback
            } else {
                v.clamp(0.0, 1.0)
            }
        };
        let y = |v: f64, fallback| if v.is_finite() { v } else { fallback };
        Self {
            x1: x(x1, default.x1),
            y1: y(y1, default.y1),
            x2: x(x2, default.x2),
            y2: y(y2, default.y2),
        }
    }
    pub(super) fn clean(self) -> Self {
        Self::new(self.x1, self.y1, self.x2, self.y2)
    }
    pub(super) fn point(self, index: usize) -> (f64, f64) {
        if index == 0 {
            (self.x1, self.y1)
        } else {
            (self.x2, self.y2)
        }
    }
    pub(super) fn with_point(self, index: usize, p: (f64, f64)) -> Self {
        if index == 0 {
            Self::new(p.0, p.1, self.x2, self.y2)
        } else {
            Self::new(self.x1, self.y1, p.0, p.1)
        }
    }
}
