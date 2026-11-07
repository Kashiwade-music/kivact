//! Role: 3D matrices, plane projections and polygons for 3D transforms.

use crate::geometry::{Point, Rect, Transform};

/// Tolerance for "is this entry the identity's entry" tests.
const NEAR: f64 = 1e-12;

/// A projective map sends points with `w'` at or below this value nowhere;
/// they are behind the viewer.
pub const W_EPS: f64 = 1e-6;

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() <= NEAR
}

fn det3(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// A 4×4 matrix on column vectors (x, y, z, w): p' = M · p, `m[row][col]`.
/// CSS `matrix3d(a1, …, a16)` lists the matrix column by column:
/// a1 = m[0][0], a2 = m[1][0], a3 = m[2][0], a4 = m[3][0], a5 = m[0][1], …
/// Positive z points toward the viewer; y points down (as in CSS).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix4 {
    /// The entries, `m[row][col]`.
    pub m: [[f64; 4]; 4],
}

impl Matrix4 {
    /// The matrix that changes nothing.
    pub const IDENTITY: Matrix4 = Matrix4 {
        m: [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ],
    };

    /// The identity with `rows` as its upper 3×3 block.
    fn from_linear(rows: [[f64; 3]; 3]) -> Matrix4 {
        let mut out = Self::IDENTITY;
        for (r, row) in rows.iter().enumerate() {
            out.m[r][..3].copy_from_slice(row);
        }
        out
    }

    /// A move by `(x, y, z)`.
    pub fn translate(x: f64, y: f64, z: f64) -> Matrix4 {
        let mut out = Self::IDENTITY;
        out.m[0][3] = x;
        out.m[1][3] = y;
        out.m[2][3] = z;
        out
    }

    /// A scale by `(x, y, z)` about the origin.
    pub fn scale(x: f64, y: f64, z: f64) -> Matrix4 {
        let mut out = Self::IDENTITY;
        out.m[0][0] = x;
        out.m[1][1] = y;
        out.m[2][2] = z;
        out
    }

    /// CSS rotateX: [[1,0,0],[0,c,-s],[0,s,c]] in the upper 3×3.
    pub fn rotate_x(deg: f64) -> Matrix4 {
        let (s, c) = deg.to_radians().sin_cos();
        Self::from_linear([[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]])
    }

    /// CSS rotateY: [[c,0,s],[0,1,0],[-s,0,c]].
    pub fn rotate_y(deg: f64) -> Matrix4 {
        let (s, c) = deg.to_radians().sin_cos();
        Self::from_linear([[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]])
    }

    /// CSS rotate / rotateZ: [[c,-s,0],[s,c,0],[0,0,1]] (clockwise on screen).
    pub fn rotate_z(deg: f64) -> Matrix4 {
        let (s, c) = deg.to_radians().sin_cos();
        Self::from_linear([[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]])
    }

    /// CSS rotate3d(x, y, z, a): Rodrigues' formula about the normalized
    /// axis; a zero or non-finite axis gives the identity.
    pub fn rotate_axis(x: f64, y: f64, z: f64, deg: f64) -> Matrix4 {
        let len = (x * x + y * y + z * z).sqrt();
        if !len.is_finite() || len == 0.0 || !deg.is_finite() {
            return Self::IDENTITY;
        }
        let (x, y, z) = (x / len, y / len, z / len);
        let (s, c) = deg.to_radians().sin_cos();
        let k = 1.0 - c;
        Self::from_linear([
            [c + x * x * k, x * y * k - z * s, x * z * k + y * s],
            [y * x * k + z * s, c + y * y * k, y * z * k - x * s],
            [z * x * k - y * s, z * y * k + x * s, c + z * z * k],
        ])
    }

    /// Upper 3×3 [[1, tan(xy), tan(xz)], [0, 1, tan(yz)], [0, 0, 1]].
    pub fn skew(xy_deg: f64, xz_deg: f64, yz_deg: f64) -> Matrix4 {
        Self::from_linear([
            [1.0, xy_deg.to_radians().tan(), xz_deg.to_radians().tan()],
            [0.0, 1.0, yz_deg.to_radians().tan()],
            [0.0, 0.0, 1.0],
        ])
    }

    /// CSS perspective(d): m[3][2] = -1/d. d <= 0 or non-finite gives the identity.
    pub fn perspective(distance: f64) -> Matrix4 {
        let mut out = Self::IDENTITY;
        if distance.is_finite() && distance > 0.0 {
            out.m[3][2] = -1.0 / distance;
        }
        out
    }

    /// The 2D affine map `t` acting on the plane z = 0 (z passes through).
    pub fn from_affine(t: Transform) -> Matrix4 {
        let mut out = Self::IDENTITY;
        out.m[0] = [t.a, t.c, 0.0, t.e];
        out.m[1] = [t.b, t.d, 0.0, t.f];
        out
    }

    /// The plane map `p` with z passed through:
    /// [[h00, h01, 0, h02], [h10, h11, 0, h12], [0, 0, 1, 0], [h20, h21, 0, h22]].
    pub fn from_projective(p: Projective) -> Matrix4 {
        let h = p.m;
        Matrix4 {
            m: [
                [h[0][0], h[0][1], 0.0, h[0][2]],
                [h[1][0], h[1][1], 0.0, h[1][2]],
                [0.0, 0.0, 1.0, 0.0],
                [h[2][0], h[2][1], 0.0, h[2][2]],
            ],
        }
    }

    /// `self` first, then `next` (= next · self), as `Transform::then`.
    pub fn then(self, next: Matrix4) -> Matrix4 {
        let mut out = [[0.0; 4]; 4];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..4).map(|k| next.m[i][k] * self.m[k][j]).sum();
            }
        }
        Matrix4 { m: out }
    }

    /// The determinant of the 3×3 block left after removing `row` and `col`.
    fn minor(&self, row: usize, col: usize) -> f64 {
        let mut block = [[0.0; 3]; 3];
        let rows = (0..4).filter(|&r| r != row);
        for (bi, r) in rows.enumerate() {
            let cols = (0..4).filter(|&c| c != col);
            for (bj, c) in cols.enumerate() {
                block[bi][bj] = self.m[r][c];
            }
        }
        det3(&block)
    }

    /// The determinant.
    pub fn determinant(&self) -> f64 {
        (0..4)
            .map(|j| {
                let sign = if j % 2 == 0 { 1.0 } else { -1.0 };
                sign * self.m[0][j] * self.minor(0, j)
            })
            .sum()
    }

    /// The matrix undoing `self`. None when |det| < 1e-12 or a result is
    /// not finite.
    pub fn invert(&self) -> Option<Matrix4> {
        let det = self.determinant();
        if !det.is_finite() || det.abs() < NEAR {
            return None;
        }
        let mut out = [[0.0; 4]; 4];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                let sign = if (i + j) % 2 == 0 { 1.0 } else { -1.0 };
                *cell = sign * self.minor(j, i) / det;
            }
        }
        let inv = Matrix4 { m: out };
        inv.is_finite().then_some(inv)
    }

    /// Where the homogeneous point `p` = (x, y, z, w) goes.
    pub fn transform_point(&self, p: [f64; 4]) -> [f64; 4] {
        let mut out = [0.0; 4];
        for (cell, row) in out.iter_mut().zip(&self.m) {
            *cell = row.iter().zip(p).map(|(a, b)| a * b).sum();
        }
        out
    }

    /// Some when row 2, row 3 and column 2 are those of the identity
    /// (within 1e-12): the matrix is a 2D affine map.
    pub fn as_affine(&self) -> Option<Transform> {
        let m = &self.m;
        let identity = Self::IDENTITY.m;
        let rows_ok = (2..4).all(|r| (0..4).all(|c| near(m[r][c], identity[r][c])));
        let col_ok = (0..4).all(|r| near(m[r][2], identity[r][2]));
        (rows_ok && col_ok).then_some(Transform {
            a: m[0][0],
            b: m[1][0],
            c: m[0][1],
            d: m[1][1],
            e: m[0][3],
            f: m[1][3],
        })
    }

    /// The map of the plane z = 0: rows {0, 1, 3} × columns {0, 1, 3}.
    pub fn flatten(&self) -> Projective {
        const IDX: [usize; 3] = [0, 1, 3];
        let mut h = [[0.0; 3]; 3];
        for (i, row) in h.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = self.m[IDX[i]][IDX[j]];
            }
        }
        Projective { m: h }
    }

    /// CSS flattening into a flat parent: row 2 and column 2 become those
    /// of the identity (m[2][*] = m[*][2] = 0 except m[2][2] = 1).
    pub fn flattened(&self) -> Matrix4 {
        let mut out = *self;
        for k in 0..4 {
            out.m[2][k] = 0.0;
            out.m[k][2] = 0.0;
        }
        out.m[2][2] = 1.0;
        out
    }

    /// Decompose a matrix whose bottom row is (0, 0, 0, 1) into
    /// translate · Rz · Ry · Rx · skew · scale. None for a singular upper
    /// 3×3 (|det| < 1e-12), another bottom row or a non-finite matrix.
    ///
    /// The upper 3×3 is split by Gram–Schmidt on its columns; a negative
    /// determinant flips `scale[0]` so the rotation stays proper.
    ///
    /// ```
    /// use kivact_types::Matrix4;
    /// let inputs = [
    ///     // Negative scale and shear.
    ///     Matrix4 {
    ///         m: [
    ///             [-1.0, 1.0, 0.0, 0.0],
    ///             [0.0, 1.0, 0.0, 0.0],
    ///             [0.0, 0.0, 1.0, 0.0],
    ///             [0.0, 0.0, 0.0, 1.0],
    ///         ],
    ///     },
    ///     // Everything at once.
    ///     Matrix4::scale(2.0, -1.0, 0.5)
    ///         .then(Matrix4::skew(10.0, 5.0, 3.0))
    ///         .then(Matrix4::rotate_x(30.0))
    ///         .then(Matrix4::rotate_y(40.0))
    ///         .then(Matrix4::rotate_z(50.0)),
    ///     // Nearly singular.
    ///     Matrix4::scale(1e-6, 1.0, 1.0),
    ///     // Gimbal lock.
    ///     Matrix4::rotate_y(90.0),
    ///     Matrix4::IDENTITY,
    /// ];
    /// for input in inputs {
    ///     let d = input.decompose().unwrap();
    ///     let rebuilt = Matrix4::scale(d.scale[0], d.scale[1], d.scale[2])
    ///         .then(Matrix4::skew(d.skew[0], d.skew[1], d.skew[2]))
    ///         .then(Matrix4::rotate_x(d.rotate[0]))
    ///         .then(Matrix4::rotate_y(d.rotate[1]))
    ///         .then(Matrix4::rotate_z(d.rotate[2]))
    ///         .then(Matrix4::translate(d.translate[0], d.translate[1], d.translate[2]));
    ///     for r in 0..4 {
    ///         for c in 0..4 {
    ///             assert!((rebuilt.m[r][c] - input.m[r][c]).abs() <= 1e-9);
    ///         }
    ///     }
    /// }
    /// ```
    pub fn decompose(&self) -> Option<Decomposed> {
        let m = &self.m;
        let bottom_ok =
            near(m[3][0], 0.0) && near(m[3][1], 0.0) && near(m[3][2], 0.0) && near(m[3][3], 1.0);
        let linear = [
            [m[0][0], m[0][1], m[0][2]],
            [m[1][0], m[1][1], m[1][2]],
            [m[2][0], m[2][1], m[2][2]],
        ];
        let det = det3(&linear);
        if !bottom_ok || !self.is_finite() || !(det.abs() >= NEAR) {
            return None;
        }
        let cols = [0, 1, 2].map(|j| [m[0][j], m[1][j], m[2][j]]);
        let gs = gram_schmidt(cols, det < 0.0);
        let rotate = euler_zyx(&gs.q).map(f64::to_degrees);
        let skew = [
            (gs.d01 / gs.s[1]).atan().to_degrees(),
            (gs.d02 / gs.s[2]).atan().to_degrees(),
            (gs.d12 / gs.s[2]).atan().to_degrees(),
        ];
        Some(Decomposed {
            translate: [m[0][3], m[1][3], m[2][3]],
            rotate,
            skew,
            scale: gs.s,
        })
    }

    /// Whether every entry is finite.
    pub fn is_finite(&self) -> bool {
        self.m.iter().flatten().all(|v| v.is_finite())
    }

    /// The back face is toward the viewer: invertible and the inverse has
    /// m[2][2] < 0 (CSS Transforms 2). Mirrors (scale(−1, 1, 1)) show the front.
    pub fn shows_back(&self) -> bool {
        self.invert().is_some_and(|inv| inv.m[2][2] < 0.0)
    }
}

/// Orthonormal columns, scales and shear terms of the Gram–Schmidt split.
struct GramSchmidt {
    /// The rotation's columns q0, q1, q2.
    q: [[f64; 3]; 3],
    /// Scale (sx, sy, sz).
    s: [f64; 3],
    d01: f64,
    d02: f64,
    d12: f64,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// `a - k·b`.
fn sub_scaled(a: [f64; 3], k: f64, b: [f64; 3]) -> [f64; 3] {
    [a[0] - k * b[0], a[1] - k * b[1], a[2] - k * b[2]]
}

fn gram_schmidt(c: [[f64; 3]; 3], flip: bool) -> GramSchmidt {
    let unit = |v: [f64; 3], len: f64| v.map(|x| x / len);
    let mut sx = dot(c[0], c[0]).sqrt();
    let mut q0 = unit(c[0], sx);
    let mut d01 = dot(q0, c[1]);
    let u1 = sub_scaled(c[1], d01, q0);
    let sy = dot(u1, u1).sqrt();
    let q1 = unit(u1, sy);
    let mut d02 = dot(q0, c[2]);
    let d12 = dot(q1, c[2]);
    let u2 = sub_scaled(sub_scaled(c[2], d02, q0), d12, q1);
    let sz = dot(u2, u2).sqrt();
    let q2 = unit(u2, sz);
    if flip {
        sx = -sx;
        q0 = q0.map(|x| -x);
        d01 = -d01;
        d02 = -d02;
    }
    GramSchmidt {
        q: [q0, q1, q2],
        s: [sx, sy, sz],
        d01,
        d02,
        d12,
    }
}

/// Radians (α, β, γ) with R = Rz(γ) · Ry(β) · Rx(α) for the rotation whose
/// columns are `q`.
fn euler_zyx(q: &[[f64; 3]; 3]) -> [f64; 3] {
    let r = |row: usize, col: usize| q[col][row];
    let beta = (-r(2, 0)).clamp(-1.0, 1.0).asin();
    if beta.cos().abs() > 1e-9 {
        [r(2, 1).atan2(r(2, 2)), beta, r(1, 0).atan2(r(0, 0))]
    } else {
        [(-r(1, 2)).atan2(r(1, 1)), beta, 0.0]
    }
}

/// translate · Rz · Ry · Rx · skew · scale, as `Matrix4::decompose` splits it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decomposed {
    /// Translation (x, y, z).
    pub translate: [f64; 3],
    /// Degrees about x, y and z; the rotation is Rz · Ry · Rx.
    pub rotate: [f64; 3],
    /// Degrees: xy (CSS skewX), xz, yz.
    pub skew: [f64; 3],
    /// Scale (x, y, z); x is negative when the matrix mirrors.
    pub scale: [f64; 3],
}

/// A projective map of the plane (a 3×3 homography on (x, y, 1)):
/// (x', y', w') = H · (x, y, 1), point (x'/w', y'/w'). Points with
/// w' <= W_EPS are behind the viewer and have no image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Projective {
    /// The homography, `m[row][col]`.
    pub m: [[f64; 3]; 3],
}

impl Projective {
    /// The map that changes nothing.
    pub const IDENTITY: Projective = Projective {
        m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };

    /// The 2D affine map `t` as a homography with bottom row [0, 0, 1].
    pub fn from_affine(t: Transform) -> Projective {
        Projective {
            m: [[t.a, t.c, t.e], [t.b, t.d, t.f], [0.0, 0.0, 1.0]],
        }
    }

    /// Some when m[2] == [0, 0, 1] within 1e-12.
    pub fn as_affine(&self) -> Option<Transform> {
        let m = &self.m;
        (near(m[2][0], 0.0) && near(m[2][1], 0.0) && near(m[2][2], 1.0)).then_some(Transform {
            a: m[0][0],
            b: m[1][0],
            c: m[0][1],
            d: m[1][1],
            e: m[0][2],
            f: m[1][2],
        })
    }

    /// `self` first, then `next` (= next · self).
    pub fn then(self, next: Projective) -> Projective {
        let mut out = [[0.0; 3]; 3];
        for (i, row) in out.iter_mut().enumerate() {
            for (j, cell) in row.iter_mut().enumerate() {
                *cell = (0..3).map(|k| next.m[i][k] * self.m[k][j]).sum();
            }
        }
        Projective { m: out }
    }

    /// The determinant.
    pub fn determinant(&self) -> f64 {
        det3(&self.m)
    }

    /// The map undoing `self`. None when |det| < 1e-12 or a result is not
    /// finite.
    pub fn invert(&self) -> Option<Projective> {
        let det = self.determinant();
        if !det.is_finite() || det.abs() < NEAR {
            return None;
        }
        let [[a, b, c], [d, e, f], [g, h, i]] = self.m;
        let adjugate = [
            [e * i - f * h, c * h - b * i, b * f - c * e],
            [f * g - d * i, a * i - c * g, c * d - a * f],
            [d * h - e * g, b * g - a * h, a * e - b * d],
        ];
        let inv = Projective {
            m: adjugate.map(|row| row.map(|v| v / det)),
        };
        inv.m.iter().flatten().all(|v| v.is_finite()).then_some(inv)
    }

    /// Homogeneous image (x', y', w') of the plane point (x, y).
    fn homogeneous(&self, x: f64, y: f64) -> [f64; 3] {
        self.m.map(|row| row[0] * x + row[1] * y + row[2])
    }

    /// Where `p` goes; None when it is behind the viewer (w' <= W_EPS) or
    /// the result is not finite.
    pub fn apply(&self, p: Point) -> Option<Point> {
        let [x, y, w] = self.homogeneous(p.x, p.y);
        if !(w > W_EPS) {
            return None;
        }
        let out = Point::new(x / w, y / w);
        (out.x.is_finite() && out.y.is_finite()).then_some(out)
    }

    /// The convex polygon `r` maps to, clipped to w' > W_EPS (§5.2).
    /// None when fewer than three vertices remain.
    pub fn map_rect(&self, r: Rect) -> Option<Polygon> {
        let corners = rect_corners(r);
        let hom = corners.map(|p| self.homogeneous(p.x, p.y));
        let mut clipped: Vec<[f64; 3]> = Vec::with_capacity(5);
        for i in 0..4 {
            let (a, b) = (hom[i], hom[(i + 1) % 4]);
            let (a_in, b_in) = (a[2] > W_EPS, b[2] > W_EPS);
            if a_in {
                clipped.push(a);
            }
            if a_in != b_in {
                clipped.push(lerp3(a, b, (W_EPS - a[2]) / (b[2] - a[2])));
            }
        }
        if clipped.len() < 3 {
            return None;
        }
        let points = clipped.iter().map(|h| Point::new(h[0] / h[2], h[1] / h[2]));
        Some(Polygon {
            points: points.collect(),
        })
    }

    /// Bounds of `map_rect`; None when nothing is in front of the viewer.
    pub fn bounds(&self, r: Rect) -> Option<Rect> {
        self.map_rect(r)?.bounds()
    }

    /// Largest ratio of a mapped edge length to its original length over
    /// the four edges of `r` and its two diagonals; 1.0 when `r` has no
    /// image or is empty. Parts of a segment behind the viewer are cut off.
    pub fn max_scale(&self, r: Rect) -> f64 {
        if !(r.width > 0.0 && r.height > 0.0) {
            return 1.0;
        }
        let c = rect_corners(r);
        let segments = [(0, 1), (1, 2), (2, 3), (3, 0), (0, 2), (1, 3)];
        let best = segments
            .iter()
            .filter_map(|&(i, j)| self.segment_scale(c[i], c[j]))
            .fold(f64::NEG_INFINITY, f64::max);
        if best.is_finite() { best } else { 1.0 }
    }

    /// Mapped length over original length of the visible part of `a`-`b`.
    fn segment_scale(&self, a: Point, b: Point) -> Option<f64> {
        let (ha, hb) = (self.homogeneous(a.x, a.y), self.homogeneous(b.x, b.y));
        let cut = (W_EPS - ha[2]) / (hb[2] - ha[2]);
        let (t0, t1) = match (ha[2] > W_EPS, hb[2] > W_EPS) {
            (true, true) => (0.0, 1.0),
            (true, false) => (0.0, cut),
            (false, true) => (cut, 1.0),
            (false, false) => return None,
        };
        let length = (b.x - a.x).hypot(b.y - a.y) * (t1 - t0);
        let (p0, p1) = (lerp3(ha, hb, t0), lerp3(ha, hb, t1));
        let mapped = (p1[0] / p1[2] - p0[0] / p0[2]).hypot(p1[1] / p1[2] - p0[1] / p0[2]);
        let ratio = mapped / length;
        (length > 0.0 && ratio.is_finite()).then_some(ratio)
    }
}

fn rect_corners(r: Rect) -> [Point; 4] {
    [
        Point::new(r.x, r.y),
        Point::new(r.x + r.width, r.y),
        Point::new(r.x + r.width, r.y + r.height),
        Point::new(r.x, r.y + r.height),
    ]
}

fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t)
}

/// A convex polygon of at most 16 vertices, in order.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Polygon {
    /// The vertices in order (either winding).
    pub points: Vec<Point>,
}

impl Polygon {
    /// The smallest rectangle holding every vertex; None for no vertices.
    pub fn bounds(&self) -> Option<Rect> {
        let first = self.points.first()?;
        let (mut x0, mut y0, mut x1, mut y1) = (first.x, first.y, first.x, first.y);
        for p in &self.points[1..] {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
        Some(Rect::new(x0, y0, x1 - x0, y1 - y0))
    }

    /// Whether `p` is inside or on the boundary. A polygon with fewer than
    /// three vertices or zero area contains nothing.
    pub fn contains(&self, p: Point) -> bool {
        let n = self.points.len();
        if n < 3 {
            return false;
        }
        let edges = || (0..n).map(|i| (self.points[i], self.points[(i + 1) % n]));
        let area2: f64 = edges().map(|(a, b)| a.x * b.y - b.x * a.y).sum();
        if area2 == 0.0 {
            return false;
        }
        let (mut positive, mut negative) = (false, false);
        for (a, b) in edges() {
            let cross = (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x);
            positive |= cross > 0.0;
            negative |= cross < 0.0;
        }
        !(positive && negative)
    }
}
