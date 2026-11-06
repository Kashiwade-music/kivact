//! Role: Geometric transforms and layout transforms.
//!
//! Transforms: CSS `transform` as a value ([`Transform`]), used by element
//! styles (`style().transform(..)`, with the `translate`, `rotate` and
//! `scale` shortcuts and `transform_origin`) and by a `Draw::layer`
//! (`Layer::transform`).
//!
//! On an element, the individual properties apply in CSS's order whatever
//! the call order: `translate`, then `rotate`, then `scale`, then the
//! `transform` list; a state style that sets only `scale` keeps the base
//! `rotate`. `translate` is one property per axis (`translate_x`,
//! `translate_y`), merged per axis by style merging and by state styles: a
//! pressed `translate_y(1)` keeps a base `translate_x(pct(-50))` centering
//! (Tailwind's `translate-x-*` and `translate-y-*`), and a state that sets
//! the same axis replaces it, as in CSS.
//!
//! A [`Transform`] is a `Copy` value in the canonical form `translate ·
//! rotate · skewX · scale` (a px translation plus a 2×2 factor of the box's
//! width and height). Functions in that order add to the parts as given
//! (`rotate(360)` stays 360); one out of order (a `rotate` after a `scale`,
//! `skew_y`, `matrix`) composes the matrix and splits it into the four
//! parts again (QR decomposition, angles within ±180°). The translation is
//! always outermost, so it never breaks the form. The engine's
//! `ElementTransform` is the canonical form and an origin (`Len` pairs),
//! which the engine turns into a matrix by the box size; `Layer::transform`
//! turns the same value into a matrix by the canvas size about the canvas
//! origin (Arvane's `PushTransform`). `Animated<Transform>` and a paint
//! transition interpolate the parts and the origin component-wise (`rotate(0)`
//! to `rotate(360)` turns once), close to CSS's per-function interpolation
//! of lists of the same shape and its decomposed matrices otherwise.
//!
//! The z-index orders painting and hits among siblings only (stable, every
//! container isolating as CSS `isolation: isolate`); the Tab order is the
//! tree order.

use kivact_types::{ElementTransform, Len, Point as WPoint};

use super::length::Length;
use crate::geometry::Size;

/// A 2D transform written like CSS `transform`: the functions in call
/// order, `Transform::new().skew_x(-12).translate_y(2)` being CSS
/// `transform: skewX(-12deg) translateY(2px)`. Angles are in degrees
/// (clockwise on screen, y grows downwards); translations take px or a
/// percentage of the element's own box (`translate_x(pct(-50))`).
///
/// On an element (`style().transform(..)`) it maps the element's box and
/// its whole subtree about the transform origin (the box's centre unless
/// `transform_origin` says otherwise), and the element is hit, pointed
/// at, reported (`NodeRef::rect`) and exposed to assistive technology
/// where it is drawn. In a `Draw` layer (`Layer::transform`) it applies
/// about the canvas origin, and percentages are of the canvas.
///
/// It keeps the form `translate · rotate · skewX · scale`, so animating it
/// (`Animated<Transform>`, a paint transition) interpolates those parts:
/// `rotate(0)` to `rotate(360)` turns once. Functions given out of that
/// order (a `rotate` after a `scale`, `skew_y`, `matrix`) are composed
/// exactly, and their parts are decomposed (angles then within ±180°).
///
/// ```
/// use kivact::prelude::*;
/// use kivact::style::{pct};
/// use kivact::draw::Transform;
/// let tilt = Transform::new().translate(0, -1).rotate(-45);
/// let card = style().transform(tilt).transform_origin(0, 0);
/// let centered = style().translate(pct(-50), pct(-50));
/// # let _ = (card, centered);
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Transform {
    /// Move in px.
    translate: [f64; 2],
    /// Move by fractions of the box: x by `[0]·w + [1]·h`, y by
    /// `[2]·w + [3]·h`.
    per_size: [f64; 4],
    /// Degrees.
    rotate: f64,
    /// Degrees along x.
    skew: f64,
    scale: [f64; 2],
    /// Which of rotate (1), skew (2) and scale (3) a function set last: a
    /// later function of an earlier part is composed by decomposition.
    stage: u8,
}

impl PartialEq for Transform {
    fn eq(&self, o: &Transform) -> bool {
        self.translate == o.translate
            && self.per_size == o.per_size
            && self.rotate == o.rotate
            && self.skew == o.skew
            && self.scale == o.scale
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::new()
    }
}

/// `tan` of `deg` degrees (exact at 0).
fn tan_deg(deg: f64) -> f64 {
    if deg == 0.0 {
        0.0
    } else {
        deg.to_radians().tan()
    }
}

impl Transform {
    /// No transform (CSS `none`).
    pub const fn new() -> Self {
        Transform {
            translate: [0.0; 2],
            per_size: [0.0; 4],
            rotate: 0.0,
            skew: 0.0,
            scale: [1.0; 2],
            stage: 0,
        }
    }

    /// The linear part `[a, b, c, d]` (`x' = a·x + c·y`, `y' = b·x + d·y`).
    fn linear(&self) -> [f64; 4] {
        let (sin, cos) = self.rotate.to_radians().sin_cos();
        let k = tan_deg(self.skew);
        let [sx, sy] = self.scale;
        [
            cos * sx,
            sin * sx,
            (cos * k - sin) * sy,
            (sin * k + cos) * sy,
        ]
    }

    /// Replace the linear part by `[a, b, c, d]`, decomposed into rotate,
    /// skew and scale.
    fn set_linear(&mut self, [a, b, c, d]: [f64; 4]) {
        let sx = a.hypot(b);
        if sx > 1e-12 {
            let (cos, sin) = (a / sx, b / sx);
            let sy = cos * d - sin * c;
            let k = if sy.abs() > 1e-12 {
                (cos * c + sin * d) / sy
            } else {
                0.0
            };
            self.rotate = b.atan2(a).to_degrees();
            self.skew = k.atan().to_degrees();
            self.scale = [sx, sy];
        } else {
            // The x axis collapses: no rotation to read off.
            self.rotate = 0.0;
            self.skew = if d.abs() > 1e-12 {
                (c / d).atan().to_degrees()
            } else {
                0.0
            };
            self.scale = [0.0, d];
        }
        self.stage = 3;
    }

    /// Follow the linear part by `[a, b, c, d]` (CSS: this function after
    /// the ones so far).
    fn then_linear(mut self, [a2, b2, c2, d2]: [f64; 4]) -> Self {
        let [a, b, c, d] = self.linear();
        self.set_linear([
            a * a2 + c * b2,
            b * a2 + d * b2,
            a * c2 + c * d2,
            b * c2 + d * d2,
        ]);
        self
    }

    /// Move by `px` and the fractions `per` of the box (`[x per w, x per h,
    /// y per w, y per h]`), mapped by the linear part so far.
    fn moved(mut self, px: [f64; 2], per: [f64; 4]) -> Self {
        let [a, b, c, d] = self.linear();
        self.translate[0] += a * px[0] + c * px[1];
        self.translate[1] += b * px[0] + d * px[1];
        let p = &mut self.per_size;
        p[0] += a * per[0] + c * per[2];
        p[1] += a * per[1] + c * per[3];
        p[2] += b * per[0] + d * per[2];
        p[3] += b * per[1] + d * per[3];
        self
    }

    /// CSS `translate(x, y)`: px, or a percentage of the box
    /// (`pct(-50)`), or both (`pct(50) - 4`).
    pub fn translate(self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        let (x, y) = (x.into().to_len(), y.into().to_len());
        self.moved([x.px, y.px], [x.pct / 100.0, 0.0, 0.0, y.pct / 100.0])
    }

    /// CSS `translateX(x)`.
    pub fn translate_x(self, x: impl Into<Length>) -> Self {
        self.translate(x, 0)
    }

    /// CSS `translateY(y)`.
    pub fn translate_y(self, y: impl Into<Length>) -> Self {
        self.translate(0, y)
    }

    /// CSS `rotate(deg)`: clockwise, in degrees.
    pub fn rotate(mut self, deg: impl Into<f64>) -> Self {
        let deg = deg.into();
        if self.stage <= 1 {
            self.rotate += deg;
            self.stage = 1;
            return self;
        }
        let (sin, cos) = deg.to_radians().sin_cos();
        self.then_linear([cos, sin, -sin, cos])
    }

    /// CSS `scale(s)`.
    pub fn scale(self, s: impl Into<f64>) -> Self {
        let s = s.into();
        self.scale_xy(s, s)
    }

    /// CSS `scale(x, y)`.
    pub fn scale_xy(mut self, x: impl Into<f64>, y: impl Into<f64>) -> Self {
        let (x, y) = (x.into(), y.into());
        self.scale = [self.scale[0] * x, self.scale[1] * y];
        self.stage = 3;
        self
    }

    /// CSS `scaleX(x)`.
    pub fn scale_x(self, x: impl Into<f64>) -> Self {
        self.scale_xy(x, 1)
    }

    /// CSS `scaleY(y)`.
    pub fn scale_y(self, y: impl Into<f64>) -> Self {
        self.scale_xy(1, y)
    }

    /// CSS `skewX(deg)`.
    pub fn skew_x(mut self, deg: impl Into<f64>) -> Self {
        let deg = deg.into();
        if self.stage <= 2 {
            self.skew = (tan_deg(self.skew) + tan_deg(deg)).atan().to_degrees();
            self.stage = 2;
            return self;
        }
        self.then_linear([1.0, 0.0, tan_deg(deg), 1.0])
    }

    /// CSS `skewY(deg)`.
    pub fn skew_y(self, deg: impl Into<f64>) -> Self {
        let t = tan_deg(deg.into());
        if t == 0.0 {
            return self;
        }
        self.then_linear([1.0, t, 0.0, 1.0])
    }

    /// CSS `skew(x, y)`.
    pub fn skew(self, x: impl Into<f64>, y: impl Into<f64>) -> Self {
        let (x, y) = (tan_deg(x.into()), tan_deg(y.into()));
        if y == 0.0 {
            return self.skew_x(x.atan().to_degrees());
        }
        self.then_linear([1.0, y, x, 1.0])
    }

    /// CSS `matrix(a, b, c, d, e, f)`: `x' = a·x + c·y + e`,
    /// `y' = b·x + d·y + f`.
    pub fn matrix(
        self,
        a: impl Into<f64>,
        b: impl Into<f64>,
        c: impl Into<f64>,
        d: impl Into<f64>,
        e: impl Into<f64>,
        f: impl Into<f64>,
    ) -> Self {
        let (a, b, c, d) = (a.into(), b.into(), c.into(), d.into());
        self.moved([e.into(), f.into()], [0.0; 4])
            .then_linear([a, b, c, d])
    }

    /// `self` followed by the functions of `next` (CSS lists `self next`).
    pub(crate) fn then(self, next: &Transform) -> Self {
        let mut out = self.moved(next.translate, next.per_size);
        if next.rotate != 0.0 {
            out = out.rotate(next.rotate);
        }
        if next.skew != 0.0 {
            out = out.skew_x(next.skew);
        }
        if next.scale != [1.0, 1.0] {
            out = out.scale_xy(next.scale[0], next.scale[1]);
        }
        out
    }

    /// The matrix `[a, b, c, d, e, f]` for a box of `size`, about its
    /// top-left corner.
    pub(crate) fn matrix_for(&self, size: Size) -> [f64; 6] {
        let [a, b, c, d] = self.linear();
        let p = self.per_size;
        [
            a,
            b,
            c,
            d,
            self.translate[0] + p[0] * size.width + p[1] * size.height,
            self.translate[1] + p[2] * size.width + p[3] * size.height,
        ]
    }

    /// The element form about `origin` (x, y).
    pub(crate) fn to_wire(self, origin: [Len; 2]) -> ElementTransform {
        ElementTransform {
            translate: WPoint {
                x: self.translate[0],
                y: self.translate[1],
            },
            per_size: self.per_size,
            rotate: self.rotate,
            skew_x: self.skew,
            scale: WPoint {
                x: self.scale[0],
                y: self.scale[1],
            },
            origin,
        }
    }

    /// Its parts in a fixed order (animation).
    pub(crate) fn components(&self) -> [f64; 10] {
        let [t0, t1] = self.translate;
        let [p0, p1, p2, p3] = self.per_size;
        let [sx, sy] = self.scale;
        [t0, t1, p0, p1, p2, p3, self.rotate, self.skew, sx, sy]
    }

    /// The transform of the parts `c` ([`Transform::components`]).
    pub(crate) fn from_components(c: [f64; 10]) -> Transform {
        Transform {
            translate: [c[0], c[1]],
            per_size: [c[2], c[3], c[4], c[5]],
            rotate: c[6],
            skew: c[7],
            scale: [c[8], c[9]],
            stage: 3,
        }
    }
}

impl kivact_runtime::Animatable for Transform {
    const COMPONENTS: usize = 10;
    fn to_components(&self, out: &mut Vec<f64>) {
        out.extend_from_slice(&self.components());
    }
    fn from_components(c: &[f64]) -> Self {
        let g = |i: usize| c.get(i).copied().unwrap_or_default();
        Transform::from_components(std::array::from_fn(g))
    }
}
