//! Role: 2D affine transforms in logical pixels.
//! Plain data usable on any thread.

use crate::{Point, Rect};

/// A 2D affine transform, CSS `matrix(a, b, c, d, e, f)`:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f` (y grows downwards, so a
/// positive rotation turns clockwise on screen).
///
/// [`crate::DrawCommand::PushTransform`] applies one to every command until
/// the matching [`crate::DrawCommand::PopTransform`].
///
/// ```
/// use kivact_types::{Point, Transform};
/// // Rotate 90° about (10, 10): the point right of the origin goes below it.
/// let t = Transform::rotate_deg(90.0).about(Point { x: 10.0, y: 10.0 });
/// let p = t.apply(Point { x: 20.0, y: 10.0 });
/// assert!((p.x - 10.0).abs() < 1e-4 && (p.y - 20.0).abs() < 1e-4);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Horizontal scale (x' per x).
    pub a: f32,
    /// Vertical skew (y' per x).
    pub b: f32,
    /// Horizontal skew (x' per y).
    pub c: f32,
    /// Vertical scale (y' per y).
    pub d: f32,
    /// Horizontal translation.
    pub e: f32,
    /// Vertical translation.
    pub f: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self::IDENTITY
    }
}

impl Transform {
    /// The transform that changes nothing.
    pub const IDENTITY: Transform = Transform {
        a: 1.0,
        b: 0.0,
        c: 0.0,
        d: 1.0,
        e: 0.0,
        f: 0.0,
    };

    /// A move by `(x, y)`.
    pub fn translate(x: f32, y: f32) -> Self {
        Transform {
            e: x,
            f: y,
            ..Self::IDENTITY
        }
    }

    /// A uniform scale about the origin.
    pub fn scale(s: f32) -> Self {
        Self::scale_xy(s, s)
    }

    /// A scale by `sx` horizontally and `sy` vertically about the origin.
    pub fn scale_xy(sx: f32, sy: f32) -> Self {
        Transform {
            a: sx,
            d: sy,
            ..Self::IDENTITY
        }
    }

    /// A rotation by `deg` degrees about the origin, clockwise on screen
    /// (CSS `rotate`).
    pub fn rotate_deg(deg: f32) -> Self {
        let (sin, cos) = deg.to_radians().sin_cos();
        Transform {
            a: cos,
            b: sin,
            c: -sin,
            d: cos,
            e: 0.0,
            f: 0.0,
        }
    }

    /// A skew by `x_deg` degrees along x and `y_deg` along y (CSS
    /// `skew(x_deg, y_deg)`).
    pub fn skew_deg(x_deg: f32, y_deg: f32) -> Self {
        Transform {
            b: y_deg.to_radians().tan(),
            c: x_deg.to_radians().tan(),
            ..Self::IDENTITY
        }
    }

    /// `self`, then `next`: the transform mapping a point by `self` and the
    /// result by `next`.
    pub fn then(self, next: Transform) -> Transform {
        Transform {
            a: next.a * self.a + next.c * self.b,
            b: next.b * self.a + next.d * self.b,
            c: next.a * self.c + next.c * self.d,
            d: next.b * self.c + next.d * self.d,
            e: next.a * self.e + next.c * self.f + next.e,
            f: next.b * self.e + next.d * self.f + next.f,
        }
    }

    /// `self` applied about `origin` instead of `(0, 0)` (CSS
    /// `transform-origin`).
    pub fn about(self, origin: Point) -> Transform {
        Transform::translate(-origin.x, -origin.y)
            .then(self)
            .then(Transform::translate(origin.x, origin.y))
    }

    /// Where `p` goes.
    pub fn apply(&self, p: Point) -> Point {
        Point {
            x: self.a * p.x + self.c * p.y + self.e,
            y: self.b * p.x + self.d * p.y + self.f,
        }
    }

    /// The determinant of the linear part (zero: the transform flattens
    /// everything to a line or a point, and nothing it applies to is drawn).
    pub fn determinant(&self) -> f32 {
        self.a * self.d - self.b * self.c
    }

    /// The transform undoing `self`, if it has one.
    pub fn invert(&self) -> Option<Transform> {
        let det = self.determinant();
        if det == 0.0 || !det.is_finite() {
            return None;
        }
        let inv = 1.0 / det;
        let (a, b, c, d) = (self.d * inv, -self.b * inv, -self.c * inv, self.a * inv);
        Some(Transform {
            a,
            b,
            c,
            d,
            e: -(a * self.e + c * self.f),
            f: -(b * self.e + d * self.f),
        })
    }

    /// Whether it is exactly the identity.
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }

    /// `Some(s)` when the transform is a translation and a uniform scale
    /// `s > 0` only (no rotation, skew or mirroring), within float rounding
    /// (a relative 1e-5: transforms that cancel, such as a skew and the
    /// counter-skew of a child, count as the translation they leave).
    pub fn uniform_scale(&self) -> Option<f32> {
        let eps = self.a.abs() * 1e-5;
        (self.a > 0.0
            && self.b.abs() <= eps
            && self.c.abs() <= eps
            && (self.a - self.d).abs() <= eps)
            .then_some(self.a)
    }

    /// The largest factor by which the transform stretches a length (the
    /// larger singular value of the linear part).
    pub fn max_scale(&self) -> f32 {
        let (a, b, c, d) = (self.a, self.b, self.c, self.d);
        let s1 = a * a + b * b + c * c + d * d;
        let s2 = ((a * a + b * b - c * c - d * d).powi(2) + 4.0 * (a * c + b * d).powi(2)).sqrt();
        ((s1 + s2) / 2.0).sqrt()
    }

    /// The axis-aligned bounds of `rect` once transformed.
    pub fn bounds(&self, rect: Rect) -> Rect {
        let corners = [
            Point {
                x: rect.x,
                y: rect.y,
            },
            Point {
                x: rect.x + rect.width,
                y: rect.y,
            },
            Point {
                x: rect.x,
                y: rect.y + rect.height,
            },
            Point {
                x: rect.x + rect.width,
                y: rect.y + rect.height,
            },
        ]
        .map(|p| self.apply(p));
        let (mut x0, mut y0, mut x1, mut y1) =
            (corners[0].x, corners[0].y, corners[0].x, corners[0].y);
        for p in &corners[1..] {
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
}
