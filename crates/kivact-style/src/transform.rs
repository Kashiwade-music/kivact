//! Role: Geometric transforms and layout transforms.
//!
//! Transforms: CSS `transform` as a value ([`Transform`]), used by element
//! styles (`style().transform(..)`, with the `translate`, `rotate` and
//! `scale` shortcuts and `transform_origin`) and by a `Draw::layer`
//! (`Layer::transform`).
//!
//! On an element, the individual properties apply in CSS's order whatever
//! the call order: `translate`, then `rotate` (z, then y, then x), then
//! `scale`, then the `transform` list; a state style that sets only `scale`
//! keeps the base `rotate`. `translate` is one property per axis
//! (`translate_x`, `translate_y`, `translate_z`), merged per axis by style
//! merging and by state styles: a pressed `translate_y(1)` keeps a base
//! `translate_x(pct(-50))` centering (Tailwind's `translate-x-*` and
//! `translate-y-*`), and a state that sets the same axis replaces it, as in
//! CSS.
//!
//! A [`Transform`] is a `Copy` value in the canonical form `translate ·
//! rotate (z, y, x) · skew · scale` (a px translation plus fractions of the
//! box, rotations and skews in degrees, scale factors per axis, and an
//! optional `perspective()`). Functions in that order add to the parts as
//! given (`rotate(360)` stays 360); one out of order (a `rotate` after a
//! `scale`, `skew_y`, `matrix`, `matrix_3d`) composes the matrix and splits
//! it into the parts again (`Matrix4::decompose`). The translation is always
//! outermost, so it never breaks the form. The engine's `ElementTransform`
//! is the canonical form and an origin (`Len` pairs and a z), which the
//! engine turns into a matrix by the box size; `Layer::transform` turns the
//! same value into a matrix by the canvas size about the canvas origin
//! (Arvane's `PushTransform` for a 2D one, a layer matrix for a 3D one).
//! `Animated<Transform>` and a paint transition interpolate the parts and
//! the origin component-wise (`rotate(0)` to `rotate(360)` turns once), close
//! to CSS's per-function interpolation of lists of the same shape and its
//! decomposed matrices otherwise.
//!
//! The z-index orders painting and hits among siblings only (stable, every
//! container isolating as CSS `isolation: isolate`); the Tab order is the
//! tree order.

use kivact_types::{ElementTransform, Len, Matrix4};

use super::length::Length;
use crate::geometry::Size;

/// Where a transform's `perspective()` function sits among its functions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PerspectivePlace {
    /// No perspective function.
    Absent,
    /// The perspective is the first function (it is valid there).
    First,
    /// The perspective follows other functions (counted as unsupported).
    Late,
}

/// A 3D transform written like CSS `transform`: the functions in call
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
/// It keeps the form `translate · rotate · skew · scale`, so animating it
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
    /// Move in px (x, y, z).
    translate: [f64; 3],
    /// Move by fractions of the box: x by `[0]·w + [1]·h`, y by
    /// `[2]·w + [3]·h`.
    per_size: [f64; 4],
    /// Degrees about x, y and z.
    rotate: [f64; 3],
    /// Degrees: xy (skewX), xz, yz.
    skew: [f64; 3],
    /// Scale factors along x, y and z.
    scale: [f64; 3],
    /// 1/d of the `perspective(d)` function; 0 is none.
    perspective_inverse: f64,
    /// The canonical stage of the last function that set a part: a
    /// function of an earlier stage is composed by decomposition.
    stage: Stage,
    /// At least one function was given (`perspective()` is valid only as
    /// the first one).
    started: bool,
    /// Where the perspective function sits, so a merge counts it once.
    perspective_place: PerspectivePlace,
    /// Functions CSS allows that this form cannot keep, counted as
    /// unsupported by the style or drawing that uses the transform.
    unsupported: u32,
}

impl PartialEq for Transform {
    fn eq(&self, o: &Transform) -> bool {
        self.translate == o.translate
            && self.per_size == o.per_size
            && self.rotate == o.rotate
            && self.skew == o.skew
            && self.scale == o.scale
            && self.perspective_inverse == o.perspective_inverse
    }
}

impl Default for Transform {
    fn default() -> Self {
        Transform::new()
    }
}

/// Canonical stages, in the order of the canonical form: a function of a later
/// stage follows the earlier ones.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Stage {
    /// No function set a part yet.
    None,
    RotateZ,
    RotateY,
    RotateX,
    Skew,
    Scale,
}

/// `tan` of `deg` degrees (exact at 0).
fn tan_deg(deg: f64) -> f64 {
    if deg == 0.0 {
        0.0
    } else {
        deg.to_radians().tan()
    }
}

/// The matrix of `a` followed by `b` as CSS lists them (`A B` is A · B).
fn cat(a: Matrix4, b: Matrix4) -> Matrix4 {
    b.then(a)
}

impl Transform {
    /// No transform (CSS `none`).
    pub const fn new() -> Self {
        Transform {
            translate: [0.0; 3],
            per_size: [0.0; 4],
            rotate: [0.0; 3],
            skew: [0.0; 3],
            scale: [1.0; 3],
            perspective_inverse: 0.0,
            stage: Stage::None,
            started: false,
            perspective_place: PerspectivePlace::Absent,
            unsupported: 0,
        }
    }

    /// The linear part (rotation, skew, scale) as a matrix: the point goes
    /// through the scale, then the skew, then the rotations x, y, z.
    fn linear(&self) -> Matrix4 {
        Matrix4::scale(self.scale[0], self.scale[1], self.scale[2])
            .then(Matrix4::skew(self.skew[0], self.skew[1], self.skew[2]))
            .then(Matrix4::rotate_x(self.rotate[0]))
            .then(Matrix4::rotate_y(self.rotate[1]))
            .then(Matrix4::rotate_z(self.rotate[2]))
    }

    /// Replace the linear part by `m`, split into the canonical parts. A
    /// singular `m` (which collapses the box) keeps only its diagonal scale.
    fn set_linear(&mut self, m: Matrix4) {
        match m.decompose() {
            Some(d) => {
                self.rotate = d.rotate;
                self.skew = d.skew;
                self.scale = d.scale;
            }
            None => {
                self.rotate = [0.0; 3];
                self.skew = [0.0; 3];
                self.scale = [m.m[0][0], m.m[1][1], m.m[2][2]];
            }
        }
        self.stage = Stage::Scale;
    }

    /// Whether a function of `stage` extends the canonical form directly
    /// (it is not before the stage already set).
    fn in_order(&self, stage: Stage) -> bool {
        stage >= self.stage
    }

    /// Follow the linear matrix `next` (CSS: this function after the ones so far).
    fn then_matrix(mut self, next: Matrix4) -> Self {
        let cur = self.linear();
        self.set_linear(cat(cur, next));
        self
    }

    /// Record that a function was given; false for the first one.
    fn begin(&mut self) -> bool {
        let first = !self.started;
        self.started = true;
        first
    }

    /// Move by `px` (x, y, z) and the fractions `per` of the box (`[x per w,
    /// x per h, y per w, y per h]`), mapped by the linear part so far.
    fn moved(mut self, px: [f64; 3], per: [f64; 4]) -> Self {
        let l = self.linear().m;
        for (i, t) in self.translate.iter_mut().enumerate() {
            *t += (0..3).map(|k| l[i][k] * px[k]).sum::<f64>();
        }
        // The percentage part maps through the x and y rows only (its z
        // contribution has no box to be a percentage of).
        let (a, b, c, d) = (l[0][0], l[0][1], l[1][0], l[1][1]);
        let p = &mut self.per_size;
        let (xw, xh, yw, yh) = (per[0], per[1], per[2], per[3]);
        p[0] += a * xw + b * yw;
        p[1] += a * xh + b * yh;
        p[2] += c * xw + d * yw;
        p[3] += c * xh + d * yh;
        self
    }

    /// CSS `translate(x, y)`: px, or a percentage of the box
    /// (`pct(-50)`), or both (`pct(50) - 4`).
    pub fn translate(self, x: impl Into<Length>, y: impl Into<Length>) -> Self {
        self.translate_3d(x, y, 0.0)
    }

    /// CSS `translateX(x)`.
    pub fn translate_x(self, x: impl Into<Length>) -> Self {
        self.translate(x, 0)
    }

    /// CSS `translateY(y)`.
    pub fn translate_y(self, y: impl Into<Length>) -> Self {
        self.translate(0, y)
    }

    /// CSS `translateZ(px)`.
    pub fn translate_z(self, z: impl Into<f64>) -> Self {
        self.translate_3d(0, 0, z)
    }

    /// CSS `translate3d(x, y, z)`: x and y as [`translate`](Self::translate),
    /// z in px.
    pub fn translate_3d(
        mut self,
        x: impl Into<Length>,
        y: impl Into<Length>,
        z: impl Into<f64>,
    ) -> Self {
        let (x, y) = (x.into().to_len(), y.into().to_len());
        self.begin();
        self.moved(
            [x.px, y.px, z.into()],
            [x.pct / 100.0, 0.0, 0.0, y.pct / 100.0],
        )
    }

    /// CSS `rotate(deg)`: clockwise, in degrees (the z axis).
    pub fn rotate(self, deg: impl Into<f64>) -> Self {
        self.rotate_z(deg)
    }

    /// CSS `rotateZ(deg)`.
    pub fn rotate_z(mut self, deg: impl Into<f64>) -> Self {
        let deg = deg.into();
        self.begin();
        if self.in_order(Stage::RotateZ) {
            self.rotate[2] += deg;
            self.stage = Stage::RotateZ;
            return self;
        }
        self.then_matrix(Matrix4::rotate_z(deg))
    }

    /// CSS `rotateY(deg)`.
    pub fn rotate_y(mut self, deg: impl Into<f64>) -> Self {
        let deg = deg.into();
        self.begin();
        if self.in_order(Stage::RotateY) {
            self.rotate[1] += deg;
            self.stage = Stage::RotateY;
            return self;
        }
        self.then_matrix(Matrix4::rotate_y(deg))
    }

    /// CSS `rotateX(deg)`.
    pub fn rotate_x(mut self, deg: impl Into<f64>) -> Self {
        let deg = deg.into();
        self.begin();
        if self.in_order(Stage::RotateX) {
            self.rotate[0] += deg;
            self.stage = Stage::RotateX;
            return self;
        }
        self.then_matrix(Matrix4::rotate_x(deg))
    }

    /// CSS `rotate3d(x, y, z, deg)`.
    pub fn rotate_3d(
        mut self,
        x: impl Into<f64>,
        y: impl Into<f64>,
        z: impl Into<f64>,
        deg: impl Into<f64>,
    ) -> Self {
        self.begin();
        let m = Matrix4::rotate_axis(x.into(), y.into(), z.into(), deg.into());
        self.then_matrix(m)
    }

    /// CSS `scale(s)`.
    pub fn scale(self, s: impl Into<f64>) -> Self {
        let s = s.into();
        self.scale_3d(s, s, 1.0)
    }

    /// CSS `scale(x, y)`.
    pub fn scale_xy(self, x: impl Into<f64>, y: impl Into<f64>) -> Self {
        self.scale_3d(x, y, 1.0)
    }

    /// CSS `scaleX(x)`.
    pub fn scale_x(self, x: impl Into<f64>) -> Self {
        self.scale_3d(x, 1.0, 1.0)
    }

    /// CSS `scaleY(y)`.
    pub fn scale_y(self, y: impl Into<f64>) -> Self {
        self.scale_3d(1.0, y, 1.0)
    }

    /// CSS `scaleZ(z)`.
    pub fn scale_z(self, z: impl Into<f64>) -> Self {
        self.scale_3d(1.0, 1.0, z)
    }

    /// CSS `scale3d(x, y, z)`.
    pub fn scale_3d(mut self, x: impl Into<f64>, y: impl Into<f64>, z: impl Into<f64>) -> Self {
        let (x, y, z) = (x.into(), y.into(), z.into());
        self.begin();
        if self.in_order(Stage::Scale) {
            self.scale = [self.scale[0] * x, self.scale[1] * y, self.scale[2] * z];
            self.stage = Stage::Scale;
            return self;
        }
        self.then_matrix(Matrix4::scale(x, y, z))
    }

    /// CSS `skewX(deg)`.
    pub fn skew_x(mut self, deg: impl Into<f64>) -> Self {
        let deg = deg.into();
        self.begin();
        if self.in_order(Stage::Skew) {
            self.skew[0] = (tan_deg(self.skew[0]) + tan_deg(deg)).atan().to_degrees();
            self.stage = Stage::Skew;
            return self;
        }
        self.then_matrix(Matrix4::skew(deg, 0.0, 0.0))
    }

    /// CSS `skewY(deg)`.
    pub fn skew_y(mut self, deg: impl Into<f64>) -> Self {
        let t = tan_deg(deg.into());
        self.begin();
        if t == 0.0 {
            return self;
        }
        // The shear of y by x: a linear map of the plane, composed exactly.
        let mut m = Matrix4::IDENTITY;
        m.m[1][0] = t;
        self.then_matrix(m)
    }

    /// CSS `skew(x, y)`.
    pub fn skew(self, x: impl Into<f64>, y: impl Into<f64>) -> Self {
        self.skew_x(x).skew_y(y)
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
        let mut lin = Matrix4::IDENTITY;
        lin.m[0][0] = a.into();
        lin.m[1][0] = b.into();
        lin.m[0][1] = c.into();
        lin.m[1][1] = d.into();
        let mut this = self.moved([e.into(), f.into(), 0.0], [0.0; 4]);
        this.begin();
        this.then_matrix(lin)
    }

    /// CSS `matrix3d(..)`, sixteen entries by column. A bottom row other
    /// than (0, 0, 0, 1) is counted as unsupported and set to (0, 0, 0, 1).
    pub fn matrix_3d(mut self, entries: [f64; 16]) -> Self {
        self.begin();
        let mut mat = Matrix4::IDENTITY;
        for (col, chunk) in entries.as_chunks::<4>().0.iter().enumerate() {
            for (row, v) in chunk.iter().enumerate() {
                mat.m[row][col] = *v;
            }
        }
        if mat.m[3][0] != 0.0 || mat.m[3][1] != 0.0 || mat.m[3][2] != 0.0 || mat.m[3][3] != 1.0 {
            self.unsupported += 1;
            mat.m[3] = [0.0, 0.0, 0.0, 1.0];
        }
        let t = [mat.m[0][3], mat.m[1][3], mat.m[2][3]];
        let mut lin = mat;
        for row in lin.m.iter_mut().take(3) {
            row[3] = 0.0;
        }
        self = self.moved(t, [0.0; 4]);
        self.then_matrix(lin)
    }

    /// CSS `perspective(d)`: a distance in px, meant as the first function
    /// of a transform. Given later, it counts as unsupported and applies as
    /// if it were first.
    pub fn perspective(mut self, d: impl Into<f64>) -> Self {
        let d = d.into();
        let first = self.begin();
        self.perspective_place = if first {
            PerspectivePlace::First
        } else {
            self.unsupported += 1;
            PerspectivePlace::Late
        };
        if d.is_finite() && d > 0.0 {
            self.perspective_inverse = 1.0 / d;
        }
        self
    }

    /// `self` followed by the functions of `next` (CSS lists `self next`).
    /// A perspective of `next` is counted once: `next` already counted it
    /// when it was not first there.
    pub(crate) fn then(self, next: &Transform) -> Self {
        let had_functions = self.started;
        let mut out = self.moved(next.translate, next.per_size);
        out.begin();
        out = out.then_matrix(next.linear());
        out.unsupported += next.unsupported;
        if next.perspective_inverse > 0.0 {
            let first_in_next = next.perspective_place == PerspectivePlace::First;
            if first_in_next && had_functions {
                out.unsupported += 1;
            }
            out.perspective_place = if first_in_next && !had_functions {
                PerspectivePlace::First
            } else {
                PerspectivePlace::Late
            };
            out.perspective_inverse = next.perspective_inverse;
        }
        out
    }

    /// The functions this transform could not keep, to report.
    pub(crate) fn unsupported(&self) -> u32 {
        self.unsupported
    }

    /// The 2D matrix `[a, b, c, d, e, f]` for a box of `size`, about its
    /// top-left corner: the x and y parts of the 3D matrix.
    pub(crate) fn matrix_for(&self, size: Size) -> [f64; 6] {
        let m = self.to_wire([Len::ZERO, Len::ZERO], 0.0).matrix(size);
        [
            m.m[0][0], m.m[1][0], m.m[0][1], m.m[1][1], m.m[0][3], m.m[1][3],
        ]
    }

    /// The element form about `origin` (x, y) and `origin_z` (px).
    pub(crate) fn to_wire(self, origin: [Len; 2], origin_z: f64) -> ElementTransform {
        ElementTransform {
            translate: self.translate,
            per_size: self.per_size,
            rotate: self.rotate,
            skew: self.skew,
            scale: self.scale,
            perspective_inverse: self.perspective_inverse,
            origin,
            origin_z,
        }
    }

    /// Its parts in a fixed order (animation): translate 3, per_size 4,
    /// rotate 3, skew 3, scale 3 and the perspective, 17 in all.
    pub(crate) fn components(&self) -> [f64; 17] {
        let (t, p, r, k, s) = (
            self.translate,
            self.per_size,
            self.rotate,
            self.skew,
            self.scale,
        );
        [
            t[0],
            t[1],
            t[2],
            p[0],
            p[1],
            p[2],
            p[3],
            r[0],
            r[1],
            r[2],
            k[0],
            k[1],
            k[2],
            s[0],
            s[1],
            s[2],
            self.perspective_inverse,
        ]
    }

    /// The transform of the parts `c` ([`Transform::components`]).
    pub(crate) fn from_components(c: [f64; 17]) -> Transform {
        Transform {
            translate: [c[0], c[1], c[2]],
            per_size: [c[3], c[4], c[5], c[6]],
            rotate: [c[7], c[8], c[9]],
            skew: [c[10], c[11], c[12]],
            scale: [c[13], c[14], c[15]],
            perspective_inverse: c[16],
            stage: Stage::Scale,
            started: true,
            perspective_place: if c[16] > 0.0 {
                PerspectivePlace::First
            } else {
                PerspectivePlace::Absent
            },
            unsupported: 0,
        }
    }
}

impl kivact_runtime::Animatable for Transform {
    const COMPONENTS: usize = 17;
    fn to_components(&self, out: &mut Vec<f64>) {
        out.extend_from_slice(&self.components());
    }
    fn from_components(c: &[f64]) -> Self {
        let g = |i: usize| c.get(i).copied().unwrap_or_default();
        Transform::from_components(std::array::from_fn(g))
    }
}
