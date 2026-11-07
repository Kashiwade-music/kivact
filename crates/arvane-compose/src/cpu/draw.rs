//! Role: Executes one DrawOp: samples, applies coverage and blends into its target.

use std::collections::HashMap;

use kivact_types::geometry::{Point, Rect};
use kivact_types::geometry3d::Projective;
use kivact_types::gpu::ExternalFrame;
use kivact_types::pixel::Pixels;
use kivact_types::render::PixelRect;

use super::blend::combine;
use super::surface::{Pixmap, Premul, Texels, sample_bilinear, sample_nearest};
use crate::ComposeError;
use crate::cache::ChunkKey;
use crate::plan::{DrawOp, Sampling, Scissor, Source};

/// The chunks of the frame (this frame's and the kept ones), by key.
pub(crate) type Chunks = HashMap<ChunkKey, Pixmap>;

/// Straight-alpha pixels read as premultiplied texels, within an optional source rectangle.
pub(crate) struct Straight<'a> {
    /// The pixels.
    pixels: &'a Pixels,
    /// The source rectangle in image px; None for all of the image.
    source: Option<Rect>,
}

impl Texels for Straight<'_> {
    fn texel(&self, x: i64, y: i64) -> Premul {
        let inside_image = x >= 0
            && y >= 0
            && x < i64::from(self.pixels.width)
            && y < i64::from(self.pixels.height);
        let inside_source = self.source.is_none_or(|r| {
            (x as f64) >= r.x
                && (y as f64) >= r.y
                && (x as f64) < r.right()
                && (y as f64) < r.bottom()
        });
        if !inside_image || !inside_source {
            return [0.0; 4];
        }
        let i = (y as usize * self.pixels.width as usize + x as usize) * 4;
        let p = &self.pixels.rgba[i..i + 4];
        let a = f32::from(p[3]) / 255.0;
        [
            f32::from(p[0]) / 255.0 * a,
            f32::from(p[1]) / 255.0 * a,
            f32::from(p[2]) / 255.0 * a,
            a,
        ]
    }
}

/// The pixels a draw reads.
enum Texture<'a> {
    /// A premultiplied pixmap (a chunk or a target).
    Premul(&'a Pixmap),
    /// Straight-alpha pixels.
    Straight(Straight<'a>),
}

impl Texture<'_> {
    /// The premultiplied color at `p` in the texture's space, sampled nearest or bilinear.
    fn sample(&self, p: Point, nearest: bool) -> Premul {
        match self {
            Texture::Premul(pixmap) => sample_with(*pixmap, p, nearest),
            Texture::Straight(straight) => sample_with(straight, p, nearest),
        }
    }
}

/// Samples `t` at `p`, nearest or bilinear.
fn sample_with<T: Texels + ?Sized>(t: &T, p: Point, nearest: bool) -> Premul {
    if nearest {
        sample_nearest(t, p)
    } else {
        sample_bilinear(t, p)
    }
}

/// A convex polygon's coverage of pixel centers (design §8.7).
pub(crate) struct Shape<'a> {
    /// The vertices in the plan's order.
    points: &'a [Point],
    /// +1 when the vertices run counter-clockwise (interior on the left), −1 otherwise.
    sign: f64,
    /// Whether each edge is a BSP split edge (top-left rule instead of antialiasing).
    split: Vec<bool>,
}

impl<'a> Shape<'a> {
    /// The shape of `points` with the `split_edges` (edge i joins point i and point i + 1).
    /// None for a polygon with fewer than three vertices or no area.
    pub(crate) fn new(points: &'a [Point], split_edges: &[usize]) -> Option<Shape<'a>> {
        let n = points.len();
        if n < 3 {
            return None;
        }
        let area2: f64 = (0..n)
            .map(|i| {
                let (a, b) = (points[i], points[(i + 1) % n]);
                a.x * b.y - b.x * a.y
            })
            .sum();
        if area2 == 0.0 {
            return None;
        }
        let mut split = vec![false; n];
        for &e in split_edges {
            if let Some(s) = split.get_mut(e) {
                *s = true;
            }
        }
        Some(Shape {
            points,
            sign: if area2 > 0.0 { 1.0 } else { -1.0 },
            split,
        })
    }

    /// The pixel rectangle `[x0, x1) × [y0, y1)` that can hold coverage, clipped to `w` × `h`.
    pub(crate) fn pixel_bounds(&self, w: u32, h: u32) -> (u32, u32, u32, u32) {
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for p in self.points {
            x0 = x0.min(p.x);
            y0 = y0.min(p.y);
            x1 = x1.max(p.x);
            y1 = y1.max(p.y);
        }
        let clamp = |v: f64, max: u32| (v.floor().max(0.0) as u32).min(max);
        let ceil = |v: f64, max: u32| (v.ceil().max(0.0) as u32).min(max);
        (clamp(x0, w), clamp(y0, h), ceil(x1, w), ceil(y1, h))
    }

    /// The coverage of the pixel center `p`: the clamped minimum inward distance plus 1/2, or the top-left rule on split edges.
    pub(crate) fn coverage(&self, p: Point) -> f64 {
        let n = self.points.len();
        let mut min_d = f64::INFINITY;
        for i in 0..n {
            let (a, b) = (self.points[i], self.points[(i + 1) % n]);
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let len = dx.hypot(dy);
            if len == 0.0 {
                continue;
            }
            let cross = dx * (p.y - a.y) - dy * (p.x - a.x);
            let d = self.sign * cross / len;
            if !self.split[i] {
                min_d = min_d.min(d);
                continue;
            }
            // Inward normal: (−dy, dx) scaled by the winding sign.
            let normal = (-dy * self.sign, dx * self.sign);
            let top = dy == 0.0 && normal.1 > 0.0;
            let left = dx == 0.0 && normal.0 > 0.0;
            if d < 0.0 || (d == 0.0 && !(top || left)) {
                return 0.0;
            }
        }
        if min_d.is_infinite() {
            return 1.0;
        }
        (0.5 + min_d).clamp(0.0, 1.0)
    }
}

/// Executes `op`: writes into `targets[op.target]` from its source, reading the other targets.
pub(crate) fn draw(
    op: &DrawOp<'_>,
    targets: &mut [Pixmap],
    chunks: &Chunks,
    missed: &mut Vec<ExternalFrame>,
) -> Result<(), ComposeError> {
    let Some(shape) = Shape::new(&op.polygon.points, &op.split_edges) else {
        return Ok(());
    };
    let target = op.target as usize;
    if let Source::Target(s) = op.source
        && s as usize == target
    {
        return Err(ComposeError::InvalidComposition(
            "a draw reads the target it writes".into(),
        ));
    }
    let cover = match op.coverage {
        Some(key) => Some(chunks.get(&key).ok_or_else(|| missing_chunk(key))?),
        None => None,
    };
    let mut dst = std::mem::take(&mut targets[target]);
    let result = match resolve(&op.source, targets, chunks, missed)? {
        Some(texture) => match texel_map(op, &texture) {
            Some(map) => paint(&mut dst, op, &map, &shape, &texture, cover),
            None => Ok(()),
        },
        None => Ok(()),
    };
    targets[target] = dst;
    result
}

/// The map from target px to the texels `texture` is sampled at.
///
/// Every source but an external frame is already in its own px, so `op.map` is used as it is.
/// An external frame's map ends at the item rect space (design §8.3): it is taken to the
/// frame's px here, by the size of the CPU pixels and the source rectangle, as the GPU does.
/// None when the item rect has no area in target px.
fn texel_map(op: &DrawOp<'_>, texture: &Texture<'_>) -> Option<Projective> {
    let (Source::External { source, .. }, Texture::Straight(straight)) = (&op.source, texture)
    else {
        return Some(op.map);
    };
    let (w, h) = (
        f64::from(straight.pixels.width),
        f64::from(straight.pixels.height),
    );
    let (mut x0, mut y0) = (f64::INFINITY, f64::INFINITY);
    let (mut x1, mut y1) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for p in &op.polygon.points {
        let q = op.map.apply(*p)?;
        x0 = x0.min(q.x);
        y0 = y0.min(q.y);
        x1 = x1.max(q.x);
        y1 = y1.max(q.y);
    }
    let (rw, rh) = (x1 - x0, y1 - y0);
    if rw <= 0.0 || rh <= 0.0 {
        return None;
    }
    let src = source.unwrap_or(Rect::new(0.0, 0.0, w, h));
    let (sx, sy) = (src.width / rw, src.height / rh);
    let to_texel = Projective {
        m: [
            [sx, 0.0, src.x - x0 * sx],
            [0.0, sy, src.y - y0 * sy],
            [0.0, 0.0, 1.0],
        ],
    };
    Some(op.map.then(to_texel))
}

/// The error for a chunk the plan named but the executor does not hold.
pub(crate) fn missing_chunk(key: ChunkKey) -> ComposeError {
    ComposeError::InvalidComposition(format!("chunk {key:?} is not rastered"))
}

/// The texture of `source`; None when an external frame has no CPU pixels (it is reported missed).
fn resolve<'a>(
    source: &Source<'a>,
    targets: &'a [Pixmap],
    chunks: &'a Chunks,
    missed: &mut Vec<ExternalFrame>,
) -> Result<Option<Texture<'a>>, ComposeError> {
    Ok(match source {
        Source::Chunk(key) => Some(Texture::Premul(
            chunks.get(key).ok_or_else(|| missing_chunk(*key))?,
        )),
        Source::Target(t) => Some(Texture::Premul(&targets[*t as usize])),
        Source::External { frame, cpu, source } => match cpu {
            Some(pixels) => Some(Texture::Straight(Straight {
                pixels,
                source: *source,
            })),
            None => {
                missed.push(*frame);
                None
            }
        },
    })
}

/// Blends every covered pixel of `dst` with the texture sampled through the op's map.
fn paint(
    dst: &mut Pixmap,
    op: &DrawOp<'_>,
    map: &Projective,
    shape: &Shape<'_>,
    texture: &Texture<'_>,
    cover: Option<&Pixmap>,
) -> Result<(), ComposeError> {
    let (w, h) = dst.size();
    let (x0, y0, x1, y1) = shape.pixel_bounds(w, h);
    for y in y0..y1 {
        for x in x0..x1 {
            if let Scissor::Rects(rects) = &op.scissor
                && !rects.iter().any(|r| inside(r, x, y))
            {
                continue;
            }
            let p = Point::new(f64::from(x) + 0.5, f64::from(y) + 0.5);
            let coverage = shape.coverage(p);
            if coverage <= 0.0 {
                continue;
            }
            let Some(q) = map.apply(p) else {
                continue;
            };
            let cover_alpha =
                cover.map_or(1.0, |c| f64::from(c.texel(i64::from(x), i64::from(y))[3]));
            let k = op.opacity * coverage * cover_alpha;
            if k <= 0.0 {
                continue;
            }
            let s = texture.sample(q, op.sampling == Sampling::Nearest);
            let s: Premul = s.map(|v| v * k as f32);
            let d = dst.get(x, y);
            dst.set(x, y, combine(s, d, op.blend, op.composite));
        }
    }
    Ok(())
}

/// Whether pixel (`x`, `y`) lies in `r`.
fn inside(r: &PixelRect, x: u32, y: u32) -> bool {
    x >= r.x && y >= r.y && x - r.x < r.width && y - r.y < r.height
}
