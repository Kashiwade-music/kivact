//! Role: Convex polygons in 3D: projecting rectangles, planes and splitting.
//!
//! A polygon keeps, per vertex, whether the edge leaving it is a split made
//! by a plane (those edges are drawn without antialiasing, see §8.4).

use kivact_types::geometry::Rect;
use kivact_types::geometry3d::{Matrix4, W_EPS};

/// Distances within this tolerance count as on a plane (design §8.4).
const EPS: f64 = 1e-4;

/// A plane `n · p + d = 0` with a unit normal `n`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Plane {
    /// The unit normal.
    pub(crate) n: [f64; 3],
    /// The offset.
    pub(crate) d: f64,
}

impl Plane {
    /// The signed distance of `p` from the plane.
    pub(crate) fn dist(&self, p: [f64; 3]) -> f64 {
        dot(self.n, p) + self.d
    }
}

/// A convex polygon in 3D with its painting key.
#[derive(Clone, Debug)]
pub(crate) struct Poly {
    /// The vertices in order.
    pub(crate) pts: Vec<[f64; 3]>,
    /// Per vertex: whether the edge leaving it is a split.
    pub(crate) leave: Vec<bool>,
    /// The painting order among coplanar polygons (the item positions of the plane).
    pub(crate) key: Vec<u32>,
    /// The draw this polygon belongs to.
    pub(crate) draw: usize,
}

/// The pieces a polygon splits into along a plane.
pub(crate) struct Split {
    /// Pieces on the positive side.
    pub(crate) front: Vec<Poly>,
    /// Pieces on the negative side.
    pub(crate) back: Vec<Poly>,
    /// The polygon itself when it lies in the plane.
    pub(crate) coplanar: Vec<Poly>,
}

/// Project a layer rectangle through a 4×4 map: its corners clipped to
/// w > W_EPS, then divided by w. None when fewer than three vertices remain.
pub(crate) fn project_rect(m: &Matrix4, r: Rect) -> Option<Vec<[f64; 3]>> {
    let corners = [
        (r.x, r.y),
        (r.right(), r.y),
        (r.right(), r.bottom()),
        (r.x, r.bottom()),
    ]
    .map(|(x, y)| m.transform_point([x, y, 0.0, 1.0]));
    let mut clipped: Vec<[f64; 4]> = Vec::with_capacity(5);
    for i in 0..4 {
        let (a, b) = (corners[i], corners[(i + 1) % 4]);
        let (a_in, b_in) = (a[3] > W_EPS, b[3] > W_EPS);
        if a_in {
            clipped.push(a);
        }
        if a_in != b_in {
            let t = (W_EPS - a[3]) / (b[3] - a[3]);
            clipped.push(lerp4(a, b, t));
        }
    }
    if clipped.len() < 3 {
        return None;
    }
    Some(
        clipped
            .iter()
            .map(|h| [h[0] / h[3], h[1] / h[3], h[2] / h[3]])
            .collect(),
    )
}

/// The plane of a convex polygon (Newell's method), None when it has no area.
pub(crate) fn plane_of(pts: &[[f64; 3]]) -> Option<Plane> {
    let n = pts.len();
    let mut normal = [0.0; 3];
    let mut centroid = [0.0; 3];
    for i in 0..n {
        let (a, b) = (pts[i], pts[(i + 1) % n]);
        normal[0] += (a[1] - b[1]) * (a[2] + b[2]);
        normal[1] += (a[2] - b[2]) * (a[0] + b[0]);
        normal[2] += (a[0] - b[0]) * (a[1] + b[1]);
        for k in 0..3 {
            centroid[k] += a[k] / n as f64;
        }
    }
    let len = dot(normal, normal).sqrt();
    if !len.is_finite() || len < 1e-12 {
        return None;
    }
    let unit = normal.map(|v| v / len);
    Some(Plane {
        n: unit,
        d: -dot(unit, centroid),
    })
}

/// Split a polygon along a plane. Pieces that touch the plane keep their
/// edges along it marked as splits.
pub(crate) fn split(poly: Poly, plane: &Plane) -> Split {
    let sides: Vec<i8> = poly.pts.iter().map(|p| side(plane.dist(*p))).collect();
    let front_any = sides.contains(&1);
    let back_any = sides.contains(&-1);
    let mut out = Split {
        front: Vec::new(),
        back: Vec::new(),
        coplanar: Vec::new(),
    };
    match (front_any, back_any) {
        (false, false) => out.coplanar.push(poly),
        (true, false) => out.front.push(poly),
        (false, true) => out.back.push(poly),
        (true, true) => {
            let (front, back) = cut(&poly, plane, &sides);
            out.front = front;
            out.back = back;
        }
    }
    out
}

/// The two pieces of a polygon that a plane crosses.
fn cut(poly: &Poly, plane: &Plane, sides: &[i8]) -> (Vec<Poly>, Vec<Poly>) {
    let n = poly.pts.len();
    let mut front = Piece::default();
    let mut back = Piece::default();
    for i in 0..n {
        let j = (i + 1) % n;
        let (a, b) = (poly.pts[i], poly.pts[j]);
        let (sa, sb) = (sides[i], sides[j]);
        let flag = poly.leave[i];
        // The edge leaving `a` in each piece is the original edge while the
        // next vertex of that piece lies along it, and a split otherwise.
        if sa >= 0 {
            front.push(a, if sa > 0 || sb >= 0 { flag } else { true });
        }
        if sa <= 0 {
            back.push(a, if sa < 0 || sb <= 0 { flag } else { true });
        }
        if sa * sb < 0 {
            let (da, db) = (plane.dist(a), plane.dist(b));
            let x = lerp3(a, b, da / (da - db));
            if sa > 0 {
                front.push(x, true);
                back.push(x, flag);
            } else {
                back.push(x, true);
                front.push(x, flag);
            }
        }
    }
    (front.into_polys(poly), back.into_polys(poly))
}

/// The side of a distance: 1 in front, -1 behind, 0 on the plane.
fn side(d: f64) -> i8 {
    if d > EPS {
        1
    } else if d < -EPS {
        -1
    } else {
        0
    }
}

/// The vertices of one piece being built.
#[derive(Default)]
struct Piece {
    /// Vertices with the leaving-edge flag of each.
    points: Vec<([f64; 3], bool)>,
}

impl Piece {
    /// Append a vertex.
    fn push(&mut self, p: [f64; 3], leave: bool) {
        self.points.push((p, leave));
    }

    /// The piece as a polygon sharing the key and draw of `source`, if it has area.
    fn into_polys(self, source: &Poly) -> Vec<Poly> {
        if self.points.len() < 3 {
            return Vec::new();
        }
        let (pts, leave): (Vec<[f64; 3]>, Vec<bool>) = self.points.into_iter().unzip();
        vec![Poly {
            pts,
            leave,
            key: source.key.clone(),
            draw: source.draw,
        }]
    }
}

/// A dot product.
pub(crate) fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Linear interpolation of 3D points.
fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [0, 1, 2].map(|k| a[k] + (b[k] - a[k]) * t)
}

/// Linear interpolation of homogeneous points.
fn lerp4(a: [f64; 4], b: [f64; 4], t: f64) -> [f64; 4] {
    [0, 1, 2, 3].map(|k| a[k] + (b[k] - a[k]) * t)
}
