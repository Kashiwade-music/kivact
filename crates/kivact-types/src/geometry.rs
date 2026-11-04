//! Role: Logical-pixel geometry.
//! Half-open rectangles keep adjacent hit regions disjoint.

/// Position in logical pixels; usable on any thread.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Point {
    /// Horizontal position.
    pub x: f64,
    /// Vertical position.
    pub y: f64,
}

/// Extent in logical pixels; usable on any thread.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Size {
    /// Horizontal extent.
    pub width: f64,
    /// Vertical extent.
    pub height: f64,
}

/// Half-open axis-aligned bounds in logical pixels; usable on any thread.
/// Operations using a rectangle specify its coordinate origin; image
/// source rectangles explicitly interpret its units as source pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    /// Left edge.
    pub x: f64,
    /// Top edge.
    pub y: f64,
    /// Horizontal extent.
    pub width: f64,
    /// Vertical extent.
    pub height: f64,
}

impl Rect {
    /// Whether the point is inside these half-open bounds.
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.y >= self.y && p.x < self.x + self.width && p.y < self.y + self.height
    }

    /// The right edge.
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    /// The bottom edge.
    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }

    /// The bounds grown by `dx` on the left and right and `dy` on the top
    /// and bottom (shrunk for negative values, never below zero size).
    pub fn inflate(&self, dx: f64, dy: f64) -> Rect {
        let width = (self.width + 2.0 * dx).max(0.0);
        let height = (self.height + 2.0 * dy).max(0.0);
        Rect {
            x: self.x + (self.width - width) / 2.0,
            y: self.y + (self.height - height) / 2.0,
            width,
            height,
        }
    }

    /// The smallest bounds holding both.
    pub fn union(&self, other: Rect) -> Rect {
        let (x0, y0) = (self.x.min(other.x), self.y.min(other.y));
        let (x1, y1) = (
            self.right().max(other.right()),
            self.bottom().max(other.bottom()),
        );
        Rect {
            x: x0,
            y: y0,
            width: x1 - x0,
            height: y1 - y0,
        }
    }

    /// Whether the two bounds share an area or an edge.
    pub fn intersects(&self, other: Rect) -> bool {
        self.x <= other.right()
            && other.x <= self.right()
            && self.y <= other.bottom()
            && other.y <= self.bottom()
    }
}

impl Point {
    pub const fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}

impl Size {
    pub const fn new(width: f64, height: f64) -> Self {
        Size { width, height }
    }

    /// The rect at the origin with this size.
    pub fn to_rect(self) -> Rect {
        Rect::new(0.0, 0.0, self.width, self.height)
    }
}

impl Rect {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// The rectangle between two corners, regardless of their order.
    /// Equal points produce a rectangle with zero width and height.
    pub fn from_points(a: Point, b: Point) -> Rect {
        Rect::new(
            a.x.min(b.x),
            a.y.min(b.y),
            (a.x - b.x).abs(),
            (a.y - b.y).abs(),
        )
    }

    pub fn center(&self) -> Point {
        Point::new(self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

impl Rect {
    pub fn intersect(&self, other: &Rect) -> Rect {
        let x0 = self.x.max(other.x);
        let y0 = self.y.max(other.y);
        let x1 = (self.x + self.width).min(other.x + other.width);
        let y1 = (self.y + self.height).min(other.y + other.height);
        Rect {
            x: x0,
            y: y0,
            width: (x1 - x0).max(0.0),
            height: (y1 - y0).max(0.0),
        }
    }
}

/// Distances from a window's edges, in logical pixels.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Insets {
    /// From the top edge.
    pub top: f64,
    /// From the right edge.
    pub right: f64,
    /// From the bottom edge.
    pub bottom: f64,
    /// From the left edge.
    pub left: f64,
}

/// Values for the physical top, right, bottom and left edges.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Edges<T> {
    /// Top edge.
    pub top: T,
    /// Right edge.
    pub right: T,
    /// Bottom edge.
    pub bottom: T,
    /// Left edge.
    pub left: T,
}

impl<T: Copy> Edges<T> {
    /// Use the same value on every edge.
    pub fn all(v: T) -> Self {
        Edges {
            top: v,
            right: v,
            bottom: v,
            left: v,
        }
    }
}

/// A 2D affine transform, CSS `matrix(a, b, c, d, e, f)`:
/// `x' = a·x + c·y + e`, `y' = b·x + d·y + f` (y grows downwards, so a
/// positive rotation turns clockwise on screen).
///
/// [`crate::render::RenderCmd::PushTransform`] applies one to every command until
/// the matching [`crate::render::RenderCmd::PopTransform`].
///
/// ```
/// use kivact_types::Point;
/// use kivact_types::geometry::Transform;
/// // Rotate 90° about (10, 10): the point right of the origin goes below it.
/// let t = Transform::rotate_deg(90.0).about(Point { x: 10.0, y: 10.0 });
/// let p = t.apply(Point { x: 20.0, y: 10.0 });
/// assert!((p.x - 10.0).abs() < 1e-4 && (p.y - 20.0).abs() < 1e-4);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transform {
    /// Horizontal scale (x' per x).
    pub a: f64,
    /// Vertical skew (y' per x).
    pub b: f64,
    /// Horizontal skew (x' per y).
    pub c: f64,
    /// Vertical scale (y' per y).
    pub d: f64,
    /// Horizontal translation.
    pub e: f64,
    /// Vertical translation.
    pub f: f64,
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
    pub fn translate(x: f64, y: f64) -> Self {
        Transform {
            e: x,
            f: y,
            ..Self::IDENTITY
        }
    }

    /// A uniform scale about the origin.
    pub fn scale(s: f64) -> Self {
        Self::scale_xy(s, s)
    }

    /// A scale by `sx` horizontally and `sy` vertically about the origin.
    pub fn scale_xy(sx: f64, sy: f64) -> Self {
        Transform {
            a: sx,
            d: sy,
            ..Self::IDENTITY
        }
    }

    /// A rotation by `deg` degrees about the origin, clockwise on screen
    /// (CSS `rotate`).
    pub fn rotate_deg(deg: f64) -> Self {
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
    pub fn skew_deg(x_deg: f64, y_deg: f64) -> Self {
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
    pub fn determinant(&self) -> f64 {
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
    pub fn uniform_scale(&self) -> Option<f64> {
        let eps = self.a.abs() * 1e-5;
        (self.a > 0.0
            && self.b.abs() <= eps
            && self.c.abs() <= eps
            && (self.a - self.d).abs() <= eps)
            .then_some(self.a)
    }

    /// The largest factor by which the transform stretches a length (the
    /// larger singular value of the linear part).
    pub fn max_scale(&self) -> f64 {
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

impl Insets {
    /// Replace negative and non-finite safe-area distances with zero.
    pub fn sanitize(self) -> Self {
        let side = |v: f64| if v.is_finite() { v.max(0.0) } else { 0.0 };
        Self {
            top: side(self.top),
            right: side(self.right),
            bottom: side(self.bottom),
            left: side(self.left),
        }
    }
}
