//! Role: CSS border-image: a source (an image, a gradient or a solid color)
//! sliced into nine regions and drawn into the border ring of a box.
//!
//! Invariants:
//! - The border image area is the border box grown by the outset on each
//!   side. An image source is its natural size (nothing is drawn until it
//!   loads); a gradient or a solid color is as large as the area.
//! - Slices are source lengths: px, or a percentage of the source size per
//!   side. Widths are the border widths, or `width` (a percentage of the
//!   area). When the widths of an axis would overflow the area, all four
//!   shrink by one factor. A middle slice wider than the source's middle is
//!   empty.
//! - Corners are stretched into their corners. Edges and, when `fill` is
//!   set, the middle tile along their axis as `repeat` says: stretched,
//!   repeated centered and clipped, rounded to whole tiles, or spaced. An
//!   axis with more than 256 tiles stretches instead. The middle's tiles
//!   scale with the top edge horizontally and the left edge vertically.
//! - An image tile is one `ImageSampled` of the whole image's pixels. A
//!   gradient or solid tile is a clip, a transform onto the tile and one
//!   fill of the source rectangle. A source that cannot be drawn (a
//!   coordinate gradient, a shader) draws nothing.

use crate::lower::SceneOut;
use crate::lower::context::Context;
use crate::lower::fill::gradient;
use crate::lower::sanitize;
use arvane::asset::PixelSize;
use kivact_types::render::{Paint, RenderCmd as D};
use kivact_types::scene::Shape;
use kivact_types::{
    BorderImage, BorderImageRepeat as Repeat, Edges, FillImage, Len, Pixels, Rect, Transform,
};

/// The most tiles along an axis before its edges are stretched instead.
const MAX_TILES: f64 = 256.0;

/// What a border image's tiles are drawn from.
enum Source {
    /// The image's pixels, and how many of them span one source px.
    Image { pixels: Pixels, scale: f64 },
    /// A paint over the border image area, in that area's coordinates.
    Paint { paint: Paint, area: Rect },
}

/// One run of tiles along an axis: where it lies in the destination and
/// which source px (`src`, from start to end) its tiles show.
#[derive(Clone, Copy)]
struct Piece {
    /// Start in the destination.
    at: f64,
    /// Length in the destination.
    len: f64,
    /// Source start and end, in source px.
    src: (f64, f64),
}

/// The geometry of a border image: its area, the source size, the source
/// slices and the destination widths around the area.
struct Nine {
    /// The border image area.
    area: Rect,
    /// The source size.
    size: (f64, f64),
    /// The slices: the source px kept at each side.
    slice: Edges<f64>,
    /// The widths: the destination px at each side.
    width: Edges<f64>,
}

/// Draw `image` into the border ring of a box whose border box is
/// `border_box`, with the border widths `borders` when it has no `width`.
pub(crate) fn lower_border_image(
    out: &mut SceneOut,
    image: &BorderImage,
    border_box: Rect,
    borders: Edges<f64>,
    ctx: &Context<'_>,
) {
    let area = outset(border_box, image.outset);
    let Some((source, size)) = resolve(&image.source, area, ctx) else {
        return;
    };
    let nine = Nine {
        area,
        size,
        slice: slices(image, size),
        width: widths(image, borders, area),
    };
    for (src, dst) in tiles(&nine, image.fill, image.repeat) {
        emit(out, &source, src, dst);
    }
}

/// A finite value, or zero.
fn finite(v: f64) -> f64 {
    if v.is_finite() { v } else { 0.0 }
}

/// The border image area: `border_box` grown by `outset` on each side.
fn outset(border_box: Rect, grow: Edges<f64>) -> Rect {
    Rect {
        x: border_box.x - finite(grow.left),
        y: border_box.y - finite(grow.top),
        width: border_box.width + finite(grow.left) + finite(grow.right),
        height: border_box.height + finite(grow.top) + finite(grow.bottom),
    }
}

/// The source to draw and its size: an image's natural size, or the size of
/// `area` for a gradient or a color. `None` when nothing can be drawn yet.
fn resolve(source: &FillImage, area: Rect, ctx: &Context<'_>) -> Option<(Source, (f64, f64))> {
    let area_size = (area.width, area.height);
    match source {
        FillImage::Image(asset) => {
            let natural = ctx.natural(asset)?;
            if !(natural.width > 0.0 && natural.height > 0.0) {
                return None;
            }
            let device = PixelSize::of(natural, ctx.raster.get());
            let (pixels, _) = ctx.pixels(asset, device, 0)?;
            let scale = pixels.width as f64 / natural.width;
            let size = (natural.width, natural.height);
            Some((Source::Image { pixels, scale }, size))
        }
        FillImage::Gradient(g) => {
            let paint = gradient(g, area)?;
            if !sanitize::fill(&paint) {
                ctx.drop_command();
                return None;
            }
            Some((Source::Paint { paint, area }, area_size))
        }
        FillImage::Solid(color) => {
            let paint = Paint::Solid(color.sanitize());
            Some((Source::Paint { paint, area }, area_size))
        }
        FillImage::Coordinate(_) | FillImage::Shader(_) => None,
    }
}

/// The four slices in source px, each a length against the source size
/// along its side.
fn slices(image: &BorderImage, (sw, sh): (f64, f64)) -> Edges<f64> {
    let s = &image.slice;
    let px = |len: Len, reference: f64| finite(len.of(reference)).max(0.0);
    Edges {
        top: px(s.top, sh),
        right: px(s.right, sw),
        bottom: px(s.bottom, sh),
        left: px(s.left, sw),
    }
}

/// The four widths in destination px: the border widths, or `width`
/// (a percentage of the area), shrunk by one factor when an axis does not
/// fit the area.
fn widths(image: &BorderImage, borders: Edges<f64>, area: Rect) -> Edges<f64> {
    let mut w = match &image.width {
        None => borders,
        Some(e) => Edges {
            top: finite(e.top.of(area.height)).max(0.0),
            right: finite(e.right.of(area.width)).max(0.0),
            bottom: finite(e.bottom.of(area.height)).max(0.0),
            left: finite(e.left.of(area.width)).max(0.0),
        },
    };
    let fit_x = area.width / (w.left + w.right);
    let fit_y = area.height / (w.top + w.bottom);
    let factor = fit_x.min(fit_y);
    if factor < 1.0 {
        w = Edges {
            top: w.top * factor,
            right: w.right * factor,
            bottom: w.bottom * factor,
            left: w.left * factor,
        };
    }
    w
}

/// The source and destination rectangles of every tile of `nine`, in
/// drawing order: the corners, the edges and (when `fill`) the middle.
fn tiles(nine: &Nine, fill: bool, repeat: [Repeat; 2]) -> Vec<(Rect, Rect)> {
    let (a, s, w) = (nine.area, nine.slice, nine.width);
    let (sw, sh) = nine.size;
    let x_src = (s.left, (sw - s.right).max(s.left));
    let y_src = (s.top, (sh - s.bottom).max(s.top));
    let x_dst = (a.x + w.left, (a.width - w.left - w.right).max(0.0));
    let y_dst = (a.y + w.top, (a.height - w.top - w.bottom).max(0.0));
    let whole = |src: (f64, f64), dst: (f64, f64)| {
        vec![Piece {
            at: dst.0,
            len: dst.1,
            src,
        }]
    };
    let x_left = whole((0.0, s.left), (a.x, w.left));
    let x_right = whole((sw - s.right, sw), (a.right() - w.right, w.right));
    let y_top = whole((0.0, s.top), (a.y, w.top));
    let y_bottom = whole((sh - s.bottom, sh), (a.bottom() - w.bottom, w.bottom));
    let x_run = |thickness: f64, slice: f64| {
        let tile = (x_src.1 - x_src.0) * scale(thickness, slice);
        pieces(x_src, x_dst, tile, repeat[0])
    };
    let y_run = |thickness: f64, slice: f64| {
        let tile = (y_src.1 - y_src.0) * scale(thickness, slice);
        pieces(y_src, y_dst, tile, repeat[1])
    };
    let (x_top, x_bottom) = (x_run(w.top, s.top), x_run(w.bottom, s.bottom));
    let (y_left, y_right) = (y_run(w.left, s.left), y_run(w.right, s.right));
    let mut cells: Vec<(&Vec<Piece>, &Vec<Piece>)> = vec![
        (&x_left, &y_top),
        (&x_right, &y_top),
        (&x_right, &y_bottom),
        (&x_left, &y_bottom),
        (&x_top, &y_top),
        (&x_bottom, &y_bottom),
        (&x_left, &y_left),
        (&x_right, &y_right),
    ];
    if fill {
        cells.push((&x_top, &y_left));
    }
    let mut out = Vec::new();
    for (xs, ys) in cells {
        for x in xs {
            for y in ys {
                out.push(crossing(x, y));
            }
        }
    }
    out
}

/// The tile at the crossing of `x` and `y`: its source and destination.
fn crossing(x: &Piece, y: &Piece) -> (Rect, Rect) {
    let src = Rect {
        x: x.src.0,
        y: y.src.0,
        width: x.src.1 - x.src.0,
        height: y.src.1 - y.src.0,
    };
    let dst = Rect {
        x: x.at,
        y: y.at,
        width: x.len,
        height: y.len,
    };
    (src, dst)
}

/// The factor from a source thickness to a destination width (zero when
/// the source has no thickness).
fn scale(width: f64, slice: f64) -> f64 {
    if slice > 0.0 { width / slice } else { 0.0 }
}

/// The pieces of a middle run of `src` laid into `dst` (`start`, `length`)
/// with tiles `tile` px long, as `mode` says.
fn pieces(src: (f64, f64), dst: (f64, f64), tile: f64, mode: Repeat) -> Vec<Piece> {
    let (start, len) = dst;
    if len.is_nan() || len <= 0.0 {
        return Vec::new();
    }
    let whole = vec![Piece {
        at: start,
        len,
        src,
    }];
    match mode {
        Repeat::Stretch => whole,
        _ if tile.is_nan() || tile <= 0.0 => Vec::new(),
        _ if len / tile > MAX_TILES => whole,
        Repeat::Repeat => repeated(src, dst, tile),
        Repeat::Round => rounded(src, dst, len / tile),
        Repeat::Space => spaced(src, dst, tile, len / tile),
    }
}

/// The source part between the fractions `from` and `to` of `src`.
fn part(src: (f64, f64), from: f64, to: f64) -> (f64, f64) {
    let span = src.1 - src.0;
    (src.0 + from * span, src.0 + to * span)
}

/// Tiles of `tile` px centered on the run `dst`, clipped to it.
fn repeated(src: (f64, f64), dst: (f64, f64), tile: f64) -> Vec<Piece> {
    let (start, len) = dst;
    let end = start + len;
    let mid = start + len / 2.0;
    let half = (len / tile / 2.0).ceil() as i64 + 1;
    (-half..=half)
        .filter_map(|k| {
            let at = mid - tile / 2.0 + k as f64 * tile;
            let (from, to) = (at.max(start), (at + tile).min(end));
            (to > from).then(|| Piece {
                at: from,
                len: to - from,
                src: part(src, (from - at) / tile, (to - at) / tile),
            })
        })
        .collect()
}

/// About `count` whole tiles spread over the run, each an equal share of
/// it.
fn rounded(src: (f64, f64), dst: (f64, f64), count: f64) -> Vec<Piece> {
    let n = count.round().max(1.0) as usize;
    let step = dst.1 / n as f64;
    (0..n)
        .map(|i| Piece {
            at: dst.0 + i as f64 * step,
            len: step,
            src,
        })
        .collect()
}

/// The whole tiles of `tile` px that fit the run `dst`, spread with equal
/// gaps (none when no tile fits).
fn spaced(src: (f64, f64), dst: (f64, f64), tile: f64, count: f64) -> Vec<Piece> {
    let n = count.floor();
    if n < 1.0 {
        return Vec::new();
    }
    let n = n as usize;
    let gap = (dst.1 - n as f64 * tile) / (n as f64 + 1.0);
    (0..n)
        .map(|i| Piece {
            at: dst.0 + gap * (i as f64 + 1.0) + i as f64 * tile,
            len: tile,
            src,
        })
        .collect()
}

/// Draw the source part `src` (source px) into `dst`.
fn emit(out: &mut SceneOut, source: &Source, src: Rect, dst: Rect) {
    if !(src.width > 0.0 && src.height > 0.0 && dst.width > 0.0 && dst.height > 0.0) {
        return;
    }
    match source {
        Source::Image { pixels, scale } => out.push(D::ImageSampled {
            image: pixels.clone(),
            rect: dst,
            source: Some(Rect {
                x: src.x * scale,
                y: src.y * scale,
                width: src.width * scale,
                height: src.height * scale,
            }),
            opacity: 1.0,
            pixelated: false,
        }),
        Source::Paint { paint, area } => {
            let frame = Rect {
                x: area.x + src.x,
                y: area.y + src.y,
                width: src.width,
                height: src.height,
            };
            let (kx, ky) = (dst.width / src.width, dst.height / src.height);
            out.push(D::PushClip(dst));
            out.push(D::PushTransform(Transform {
                a: kx,
                b: 0.0,
                c: 0.0,
                d: ky,
                e: dst.x - frame.x * kx,
                f: dst.y - frame.y * ky,
            }));
            out.push(D::FillShape {
                shape: Shape::Rect(frame),
                paint: paint.clone(),
            });
            out.push(D::PopTransform);
            out.push(D::PopClip);
        }
    }
}
