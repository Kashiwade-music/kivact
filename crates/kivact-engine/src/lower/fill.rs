//! Role: Fills: background layers and CSS gradients as Arvane paints.
//!
//! Fills: a box's (or a scene rect's) background layers as Arvane paints,
//! placed like CSS background layers, and CSS gradients as Arvane
//! gradients.
//!
//! Invariants:
//! - A layer's tile size and position are against the area it fills (a
//!   node's border box: `background-origin: border-box`); its tiles are
//!   clipped to the area's outline (rounded corners included). The last
//!   layer is drawn first.
//! - The commands do not grow with the area: a tile covering the area is
//!   one fill; up to [`FEW`] gradient copies are each a clipped fill of
//!   their own; more copies, and every image, are one pattern fill (Arvane
//!   repeats the tile).
//! - Gradient stops are placed as CSS places them (missing positions
//!   spread evenly, a position before an earlier one moved up to it); a
//!   repeating gradient repeats from its first stop to its last.
//!   A repeating radial gradient repeats its stops shifted by the phase of
//!   its first stop. A conic gradient's angles are CSS's (0° up, turned to
//!   Arvane's 0° right); a non-repeating one is one whole turn drawn with
//!   `Repeat`, as Arvane sweeps from 0° to 360° and would pad the angles
//!   below a negative start.
//! - A coordinate gradient (canvas `createLinearGradient` ..) ignores the
//!   layer's tile: its points are against the origin of what is painted (a
//!   scene's canvas origin, a box's top left corner); it has no transform of
//!   its own (a scene layer's transform moves it) and does not follow the
//!   bounds of the shape it fills, so where shapes filled with one gradient
//!   overlap they show the same colors. It becomes Arvane's `Linear`,
//!   `RadialTwoCircle` (the two-point radial of both renderers) or `Conic`
//!   (from its start angle, 0° to the right, over one turn with `Repeat`)
//!   with its extend and its stops as kivact orders them (by offset; equal
//!   offsets in the order added, a hard edge); one `sanitize::fill`
//!   rejects paints nothing (`coordinate`).
//! - An image shows nothing until loaded; its failure is not reported.

use super::context::Context;
use crate::lower::effects::Tint;
use crate::lower::shape::BoxShape;
use kivact_types::render::RenderCmd as D;
use kivact_types::{
    CoordinateGradient, CoordinateShape, Fill, FillImage, FillLayer, FillSize, Gradient,
    GradientKind, RadialExtent,
};
use kivact_types::{Rgba, Size};

use kivact_types::render::{GradientShape, Pattern, PatternSource};
use kivact_types::{Extend, Point, Rect};

/// Copies of a gradient tile drawn one by one; more are a pattern.
const FEW: u32 = 4;

impl BoxShape {
    /// Fill the outline with `paint`.
    pub(crate) fn fill(
        &self,
        out: &mut crate::lower::SceneOut,
        paint: kivact_types::render::Paint,
    ) {
        match *self {
            BoxShape::Rect(rect) => out.push(D::RoundedRect {
                rect,
                radius: 0.0,
                fill: paint,
            }),
            BoxShape::Circular(rect, radii) => {
                if radii.iter().all(|r| *r == radii[0]) {
                    out.push(D::RoundedRect {
                        rect,
                        radius: radii[0],
                        fill: paint,
                    });
                } else {
                    out.push(D::RoundedRectCorners {
                        rect,
                        radii,
                        fill: paint,
                    });
                }
            }
            BoxShape::Rounded(_) => out.push(D::FillShape {
                shape: self.shape(),
                paint,
            }),
        }
    }
}

/// Where the copies of a layer's tile lie in an area.
struct Tiles {
    /// The first copy.
    first: Rect,
    nx: u32,
    ny: u32,
}

impl Tiles {
    /// The copies of a `size` tile at `position` (resolved against the
    /// area less the tile, as CSS does), repeated as `layer` says.
    fn of(layer: &FillLayer, area: Rect, size: (f64, f64)) -> Option<Tiles> {
        let (tw, th) = size;
        if !(tw.is_finite() && th.is_finite() && tw > 0.0 && th > 0.0) {
            return None;
        }
        let ox = area.x + layer.position[0].of(area.width - tw);
        let oy = area.y + layer.position[1].of(area.height - th);
        if !(ox.is_finite() && oy.is_finite()) {
            return None;
        }
        // The first copy at or before the area's start, and how many
        // reach its end.
        let axis = |o: f64, t: f64, start: f64, len: f64, repeat: bool| -> (f64, u32) {
            if repeat {
                let first = o - ((o - start) / t).ceil() * t;
                let n = ((start + len - first) / t).ceil().max(1.0);
                (first, n.min(u32::MAX as f64) as u32)
            } else {
                (o, 1)
            }
        };
        let (x, nx) = axis(ox, tw, area.x, area.width, layer.repeat_x);
        let (y, ny) = axis(oy, th, area.y, area.height, layer.repeat_y);
        Some(Tiles {
            first: Rect {
                x,
                y,
                width: tw,
                height: th,
            },
            nx,
            ny,
        })
    }

    fn copies(&self) -> u64 {
        u64::from(self.nx) * u64::from(self.ny)
    }

    /// One copy covering the whole area.
    fn covers(&self, area: Rect) -> bool {
        self.copies() == 1
            && self.first.x <= area.x
            && self.first.y <= area.y
            && self.first.right() >= area.right()
            && self.first.bottom() >= area.bottom()
    }

    fn each(&self) -> impl Iterator<Item = Rect> + '_ {
        (0..self.ny).flat_map(move |j| {
            (0..self.nx).map(move |i| Rect {
                x: self.first.x + i as f64 * self.first.width,
                y: self.first.y + j as f64 * self.first.height,
                ..self.first
            })
        })
    }

    /// The pattern repeating `source` from the first copy.
    fn pattern(&self, layer: &FillLayer, source: PatternSource) -> kivact_types::render::Paint {
        kivact_types::render::Paint::Pattern(Pattern {
            source,
            tile: self.first,
            repeat_x: layer.repeat_x,
            repeat_y: layer.repeat_y,
        })
    }
}

impl Context<'_> {
    /// Fill `shape` with `fill` through `tint` (see the module invariants);
    /// `origin` is what coordinate gradients' points are against.
    pub(crate) fn fill(
        &self,
        out: &mut crate::lower::SceneOut,
        shape: &BoxShape,
        fill: &Fill,
        tint: &Tint,
        origin: Point,
    ) {
        match fill {
            Fill::Solid(color) => {
                shape.fill(out, kivact_types::render::Paint::Solid(tint.color(*color)))
            }
            Fill::Layers(layers) => {
                for layer in layers.iter().rev() {
                    self.fill_layer(out, shape, layer, tint, origin);
                }
            }
        }
    }

    /// Fill `shape` with `paint`, or drop it (counted) when Arvane would
    /// reject it: one rejected command fails the whole frame.
    fn put(
        &self,
        out: &mut crate::lower::SceneOut,
        shape: &BoxShape,
        paint: kivact_types::render::Paint,
    ) {
        if crate::lower::sanitize::fill(&paint) {
            shape.fill(out, paint);
        } else {
            self.drop_command();
        }
    }

    /// The size of a layer's tile in `area` (`None`: nothing to draw yet,
    /// an image still loading).
    fn tile_size(&self, layer: &FillLayer, area: Rect) -> Option<(f64, f64)> {
        let natural = match &layer.image {
            FillImage::Image(asset) => {
                let n = self.natural(asset)?;
                Some((n.width, n.height))
            }
            _ => None,
        };
        let (aw, ah) = (area.width, area.height);
        let scale_to = |cover: bool| match natural {
            Some((w, h)) if w > 0.0 && h > 0.0 => {
                let (sx, sy) = (aw / w, ah / h);
                let s = if cover { sx.max(sy) } else { sx.min(sy) };
                (w * s, h * s)
            }
            _ => (aw, ah),
        };
        Some(match layer.size {
            FillSize::Auto => natural.unwrap_or((aw, ah)),
            FillSize::Cover => scale_to(true),
            FillSize::Contain => scale_to(false),
            FillSize::Explicit(w, h) => {
                let w = w.map(|w| w.of(aw));
                let h = h.map(|h| h.of(ah));
                match (w, h, natural) {
                    (Some(w), Some(h), _) => (w, h),
                    (Some(w), None, Some((nw, nh))) if nw > 0.0 => (w, w * nh / nw),
                    (None, Some(h), Some((nw, nh))) if nh > 0.0 => (h * nw / nh, h),
                    (w, h, natural) => {
                        let (nw, nh) = natural.unwrap_or((aw, ah));
                        (w.unwrap_or(nw), h.unwrap_or(nh))
                    }
                }
            }
        })
    }

    /// One background layer over `shape`.
    fn fill_layer(
        &self,
        out: &mut crate::lower::SceneOut,
        shape: &BoxShape,
        layer: &FillLayer,
        tint: &Tint,
        origin: Point,
    ) {
        if let FillImage::Coordinate(g) = &layer.image {
            if let Some(paint) = coordinate(g, origin) {
                shape.fill(out, tint.paint(paint));
            }
            return;
        }
        let area = shape.rect();
        let Some(size) = self.tile_size(layer, area) else {
            return;
        };
        let Some(tiles) = Tiles::of(layer, area, size) else {
            return;
        };
        match &layer.image {
            FillImage::Solid(color) => {
                let paint = kivact_types::render::Paint::Solid(tint.color(*color));
                if tiles.covers(area) || (layer.repeat_x && layer.repeat_y) {
                    self.put(out, shape, paint);
                } else {
                    self.fill_copies(out, shape, &tiles, |_| Some(paint.clone()));
                }
            }
            FillImage::Gradient(g) => {
                if tiles.covers(area) {
                    if let Some(paint) = gradient(g, tiles.first) {
                        self.put(out, shape, tint.paint(paint));
                    }
                } else if tiles.copies() <= u64::from(FEW) {
                    self.fill_copies(out, shape, &tiles, |copy| {
                        gradient(g, copy).map(|p| tint.paint(p))
                    });
                } else {
                    let local = Rect {
                        x: 0.0,
                        y: 0.0,
                        ..tiles.first
                    };
                    if let Some(kivact_types::render::Paint::Gradient(g)) = gradient(g, local) {
                        let paint = tiles.pattern(layer, PatternSource::Gradient(g));
                        self.put(out, shape, tint.paint(paint));
                    }
                }
            }
            // Placed above, by coordinates.
            FillImage::Coordinate(_) => {}
            FillImage::Image(asset) => {
                let source = asset;
                let device = arvane::asset::PixelSize::of(
                    Size {
                        width: tiles.first.width,
                        height: tiles.first.height,
                    },
                    self.raster.get(),
                );
                let pixels = self.pixels(source, device);
                if let Some((image, _)) = pixels {
                    let paint = tiles.pattern(
                        layer,
                        PatternSource::Image {
                            image,
                            color_matrix: None,
                        },
                    );
                    self.put(out, shape, tint.paint(paint));
                }
            }
        }
    }

    /// Each copy of a tile, clipped to it, filled with its own paint.
    fn fill_copies(
        &self,
        out: &mut crate::lower::SceneOut,
        shape: &BoxShape,
        tiles: &Tiles,
        paint: impl Fn(Rect) -> Option<kivact_types::render::Paint>,
    ) {
        let area = shape.rect();
        for copy in tiles.each().take(FEW as usize) {
            if !copy.intersects(area) {
                continue;
            }
            let Some(paint) = paint(copy) else { continue };
            out.push(D::PushClip(copy));
            self.put(out, shape, paint);
            out.push(D::PopClip);
        }
    }

    /// One layer as a single paint over `area` (a mask's): a tile covering
    /// the area, else a pattern (`None`: nothing to draw yet). `origin` is
    /// what a coordinate gradient's points are against.
    pub(crate) fn layer_paint(
        &self,
        layer: &FillLayer,
        area: Rect,
        origin: Point,
    ) -> Option<kivact_types::render::Paint> {
        if let FillImage::Coordinate(g) = &layer.image {
            return coordinate(g, origin);
        }
        let size = self.tile_size(layer, area)?;
        let tiles = Tiles::of(layer, area, size)?;
        match &layer.image {
            FillImage::Solid(color) => {
                let paint =
                    kivact_types::render::Paint::Solid(kivact_types::Rgba::sanitize(*color));
                if tiles.covers(area) || (layer.repeat_x && layer.repeat_y) {
                    Some(paint)
                } else {
                    // A solid tile is a one-stop gradient tile.
                    let g = kivact_types::render::Gradient {
                        shape: GradientShape::Linear {
                            start: Point { x: 0.0, y: 0.0 },
                            end: Point { x: 1.0, y: 0.0 },
                        },
                        stops: vec![(0.0, kivact_types::Rgba::sanitize(*color))],
                        extend: Extend::Pad,
                    };
                    Some(tiles.pattern(layer, PatternSource::Gradient(g)))
                }
            }
            FillImage::Gradient(g) => {
                if tiles.covers(area) {
                    return gradient(g, tiles.first);
                }
                let local = Rect {
                    x: 0.0,
                    y: 0.0,
                    ..tiles.first
                };
                match gradient(g, local)? {
                    kivact_types::render::Paint::Gradient(g) => {
                        Some(tiles.pattern(layer, PatternSource::Gradient(g)))
                    }
                    other => Some(other),
                }
            }
            FillImage::Coordinate(_) => None,
            FillImage::Image(asset) => {
                let device = arvane::asset::PixelSize::of(
                    Size {
                        width: tiles.first.width,
                        height: tiles.first.height,
                    },
                    self.raster.get(),
                );
                let (image, _) = self.pixels(asset, device)?;
                Some(tiles.pattern(
                    layer,
                    PatternSource::Image {
                        image,
                        color_matrix: None,
                    },
                ))
            }
        }
    }
}

/// The stops of `g` as fractions of its line (CSS color stop fixup):
/// `length` is the line's length in px (a conic gradient's: 360°).
fn stops(g: &Gradient, length: f64) -> Vec<(f64, Rgba)> {
    let n = g.stops.len();
    let mut at: Vec<Option<f64>> = g
        .stops
        .iter()
        .map(|s| {
            s.at.map(|l| {
                let t = if length > 0.0 {
                    l.px / length + l.pct / 100.0
                } else {
                    l.pct / 100.0
                };
                if t.is_finite() { t } else { 0.0 }
            })
        })
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
    g.stops
        .iter()
        .zip(at)
        .map(|(s, t)| (t.unwrap_or(0.0), kivact_types::Rgba::sanitize(s.color)))
        .collect()
}

/// Stops spanning `[lo, hi]` mapped onto `[0, 1]`.
fn normalized(stops: &[(f64, Rgba)], lo: f64, hi: f64) -> Vec<(f64, Rgba)> {
    let span = (hi - lo).max(f64::MIN_POSITIVE);
    stops
        .iter()
        .map(|(t, c)| (((t - lo) / span).clamp(0.0, 1.0), *c))
        .collect()
}

/// The color of the stops at `t` (premultiplied interpolation).
fn color_at(stops: &[(f64, Rgba)], t: f64) -> Rgba {
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

/// One period of stops (in `[0, 1]`) shifted by `phase` (a fraction of
/// the period), wrapped around: the color at `u` is the period's at
/// `u - phase`.
fn rotated(period: &[(f64, Rgba)], phase: f64) -> Vec<(f64, Rgba)> {
    let phase = phase.rem_euclid(1.0);
    if phase == 0.0 {
        return period.to_vec();
    }
    let seam = color_at(period, 1.0 - phase);
    let mut out = vec![(0.0, seam)];
    out.extend(
        period
            .iter()
            .filter(|(t, _)| t + phase >= 1.0)
            .map(|(t, c)| (t + phase - 1.0, *c)),
    );
    out.extend(
        period
            .iter()
            .filter(|(t, _)| t + phase < 1.0)
            .map(|(t, c)| (t + phase, *c)),
    );
    out.push((1.0, seam));
    out
}

/// A CSS gradient over the box `rect` as an Arvane paint (`None`: no
/// stops).
pub(crate) fn gradient(g: &Gradient, rect: Rect) -> Option<kivact_types::render::Paint> {
    if g.stops.is_empty() {
        return None;
    }
    let lerp = |a: Point, b: Point, t: f64| Point {
        x: a.x + (b.x - a.x) * t,
        y: a.y + (b.y - a.y) * t,
    };
    let solid = |stops: &[(f64, Rgba)]| {
        Some(kivact_types::render::Paint::Solid(
            stops.last().map(|s| s.1).unwrap_or_default(),
        ))
    };
    let (shape, stops, extend) = match g.kind {
        GradientKind::Linear { angle_deg } => {
            let angle = if angle_deg.is_finite() {
                angle_deg
            } else {
                180.0
            };
            let GradientShape::Linear { start, end } =
                GradientShape::linear_angle(rect, angle - 90.0)
            else {
                return None;
            };
            let length = (end.x - start.x).hypot(end.y - start.y);
            let s = stops(g, length);
            let (first, last) = (s[0].0, s[s.len() - 1].0);
            let (lo, hi, extend) = if g.repeating {
                if last - first <= 0.0 {
                    return solid(&s);
                }
                (first, last, Extend::Repeat)
            } else {
                (first.min(0.0), last.max(1.0), Extend::Pad)
            };
            (
                GradientShape::Linear {
                    start: lerp(start, end, lo),
                    end: lerp(start, end, hi),
                },
                normalized(&s, lo, hi),
                extend,
            )
        }
        GradientKind::Radial { .. } => return radial_gradient(g, rect),
        GradientKind::Conic { from_deg, center } => {
            let cx = rect.x + center[0].of(rect.width);
            let cy = rect.y + center[1].of(rect.height);
            let from = if from_deg.is_finite() { from_deg } else { 0.0 } - 90.0;
            let s = stops(g, 360.0);
            let (first, last) = (s[0].0, s[s.len() - 1].0);
            let (lo, hi, extend) = if g.repeating {
                if last - first <= 0.0 {
                    return solid(&s);
                }
                (first, last, Extend::Repeat)
            } else {
                // One whole turn: repeating it wraps the angles below the
                // start (Arvane sweeps from 0° to 360°, so a start below 0°
                // would pad them).
                (0.0, 1.0, Extend::Repeat)
            };
            (
                GradientShape::Conic {
                    center: Point { x: cx, y: cy },
                    start_deg: from + lo * 360.0,
                    end_deg: from + hi * 360.0,
                },
                normalized(&s, lo, hi),
                extend,
            )
        }
    };
    Some(kivact_types::render::Paint::Gradient(
        kivact_types::render::Gradient {
            shape,
            stops,
            extend,
        },
    ))
}

/// A gradient by coordinates as an Arvane paint, its points moved by `origin`
/// (`None`: what `sanitize::fill` rejects: no stop, offsets that are not
/// finite, outside `0..=1` or out of order, a point or angle that is not
/// finite, a negative radius).
pub(crate) fn coordinate(
    g: &CoordinateGradient,
    origin: Point,
) -> Option<kivact_types::render::Paint> {
    let at = |p: kivact_types::Point| Point {
        x: origin.x + p.x,
        y: origin.y + p.y,
    };
    let stops = g
        .stops
        .iter()
        .map(|(t, c)| (*t, kivact_types::Rgba::sanitize(*c)))
        .collect();
    let (shape, extend) = match g.shape {
        CoordinateShape::Linear { start, end } => (
            GradientShape::Linear {
                start: at(start),
                end: at(end),
            },
            g.extend,
        ),
        CoordinateShape::RadialTwoCircle {
            start_center,
            start_radius,
            end_center,
            end_radius,
        } => (
            GradientShape::RadialTwoCircle {
                start_center: at(start_center),
                start_radius,
                end_center: at(end_center),
                end_radius,
            },
            g.extend,
        ),
        // One turn; repeating it wraps the angles below the start.
        CoordinateShape::Conic { center, start_deg } => (
            GradientShape::Conic {
                center: at(center),
                start_deg,
                end_deg: start_deg + 360.0,
            },
            Extend::Repeat,
        ),
    };
    let paint = kivact_types::render::Paint::Gradient(kivact_types::render::Gradient {
        shape,
        stops,
        extend,
    });
    super::sanitize::fill(&paint).then_some(paint)
}

fn radial_gradient(g: &Gradient, rect: Rect) -> Option<kivact_types::render::Paint> {
    let GradientKind::Radial {
        circle,
        extent,
        center,
    } = g.kind
    else {
        unreachable!("invariant: radial gradient");
    };
    let solid = |stops: &[(f64, Rgba)]| {
        Some(kivact_types::render::Paint::Solid(
            stops.last().map(|s| s.1).unwrap_or_default(),
        ))
    };

    let cx = rect.x + center[0].of(rect.width);
    let cy = rect.y + center[1].of(rect.height);
    let (rx, ry) = radial_radii(rect, cx, cy, circle, extent);
    if !(rx.is_finite() && ry.is_finite() && rx > 0.0 && ry > 0.0) {
        return solid(&stops(g, 0.0));
    }
    let s: Vec<_> = stops(g, rx)
        .into_iter()
        .map(|(t, c)| (t.max(0.0), c))
        .collect();
    let (first, last) = (s[0].0, s[s.len() - 1].0);
    let (scale, stops, extend) = if g.repeating {
        let period = last - first;
        if period <= 0.0 {
            return solid(&s);
        }
        let one = normalized(&s, first, last);
        (period, rotated(&one, first / period), Extend::Repeat)
    } else {
        let hi = last.max(1.0);
        (hi, normalized(&s, 0.0, hi), Extend::Pad)
    };
    let (shape, stops, extend) = (
        GradientShape::Radial {
            center: Point { x: cx, y: cy },
            radius_x: rx * scale,
            radius_y: ry * scale,
        },
        stops,
        extend,
    );
    Some(kivact_types::render::Paint::Gradient(
        kivact_types::render::Gradient {
            shape,
            stops,
            extend,
        },
    ))
}

fn radial_radii(rect: Rect, cx: f64, cy: f64, circle: bool, extent: RadialExtent) -> (f64, f64) {
    let (dl, dr) = ((cx - rect.x).abs(), (rect.right() - cx).abs());
    let (dt, db) = ((cy - rect.y).abs(), (rect.bottom() - cy).abs());
    let corner = |x: f64, y: f64| x.hypot(y);
    match extent {
        RadialExtent::ClosestSide if circle => {
            let r = dl.min(dr).min(dt).min(db);
            (r, r)
        }
        RadialExtent::ClosestSide => (dl.min(dr), dt.min(db)),
        RadialExtent::FarthestSide if circle => {
            let r = dl.max(dr).max(dt).max(db);
            (r, r)
        }
        RadialExtent::FarthestSide => (dl.max(dr), dt.max(db)),
        RadialExtent::ClosestCorner if circle => {
            let r = corner(dl.min(dr), dt.min(db));
            (r, r)
        }
        RadialExtent::ClosestCorner => (
            dl.min(dr) * std::f64::consts::SQRT_2,
            dt.min(db) * std::f64::consts::SQRT_2,
        ),
        RadialExtent::FarthestCorner if circle => {
            let r = corner(dl.max(dr), dt.max(db));
            (r, r)
        }
        RadialExtent::FarthestCorner => (
            dl.max(dr) * std::f64::consts::SQRT_2,
            dt.max(db) * std::f64::consts::SQRT_2,
        ),
        RadialExtent::Radii(x, _) if circle => {
            let r = x.of(rect.width);
            (r, r)
        }
        RadialExtent::Radii(x, y) => (x.of(rect.width), y.of(rect.height)),
    }
}
