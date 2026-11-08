//! Role: Layers: geometry, raster scale, signatures, surfaces, backdrops and effects.

use std::borrow::Cow;
use std::f64::consts::SQRT_2;

use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect, Size, Transform};
use kivact_types::geometry3d::Projective;
use kivact_types::paint::BackfaceVisibility;
use kivact_types::pixel::BlendMode;
use kivact_types::render::{
    Composite, CompositeLayer, LayerEffects, LayerItem, Plane, RasterGrid, RenderBackdrop,
    RenderScene,
};
use kivact_types::scene::Shape;

use super::bounds::{mapped_bounds, points_bounds};
use super::builder::Builder;
use super::frame::{
    Behind, Dest, ItemCtx, ceil_px, coverage_scene, filter_reach, full_polygon, inline_candidate,
    px_map, subtree_has_backdrop,
};
use super::hash::{self, Fnv};
use super::ops::{DrawOp, Grid, Op, PlanError, Sampling, Scissor, Source, TargetId, TargetKind};
use crate::cache::{ChunkKey, ChunkRole, LayerRecord, Retained};

/// The largest surface side in px (a larger surface is rastered at a lower scale).
const MAX_SURFACE_PX: f64 = 8192.0;
/// The smallest raster scale a surface takes.
const MIN_SCALE: f64 = 1.0 / 16.0;
/// The reciprocal of the step an axis-parallel raster scale is rounded to (one part in 10⁹).
const SCALE_SNAP: f64 = 1e9;
/// The largest raster scale a surface takes.
const MAX_SCALE: f64 = 16.0;
/// The cell, in px, that a `RasterGrid::Layer` surface's bounds snap to (design §8.3 step 2).
const LAYER_GRID_CELL_PX: f64 = 256.0;

/// The geometry of a layer's surface in its target.
#[derive(Clone, Copy, Debug)]
pub(super) struct Geo {
    /// Layer space → target px (flattened).
    pub(super) h: Projective,
    /// The surface rectangle in layer space, with its effects' reach.
    pub(super) rect: Rect,
    /// Px per layer unit of the surface.
    pub(super) scale: f64,
    /// The surface's size in px.
    pub(super) size: (u32, u32),
    /// Layer space → surface px.
    pub(super) ms: Matrix4,
    /// Whether the surface's draw is pixel aligned (sampled nearest).
    pub(super) nearest: bool,
}

impl Geo {
    /// The surface's grid: its origin at the rectangle's corner.
    pub(super) fn grid(&self) -> Grid {
        Grid {
            origin: Point::new(self.rect.x, self.rect.y),
            scale: self.scale,
            size: self.size,
        }
    }
}

/// The scene of a layer that is a single raster with default effects: its chunk is its surface.
fn chunk_shortcut(layer: &CompositeLayer) -> Option<&RenderScene> {
    if layer.effects != LayerEffects::default() || layer.items.len() != 1 {
        return None;
    }
    match &layer.items[0] {
        LayerItem::Raster(scene) => Some(scene),
        _ => None,
    }
}

/// How far a layer's effects reach beyond its bounds, in layer units.
fn reach_of(effects: &LayerEffects, bounds: Rect) -> f64 {
    let own: f64 = effects
        .filters
        .iter()
        .map(|f| filter_reach(f, bounds))
        .sum();
    let behind: f64 = effects.backdrop.as_ref().map_or(0.0, |b| {
        b.filters.iter().map(|f| filter_reach(f, bounds)).sum()
    });
    own + behind
}

/// The raster scale of a surface: exact for an axis-parallel map, else the next √2 step.
fn quantize(raw: f64, axis_parallel: bool) -> f64 {
    let raw = if raw.is_finite() && raw > 0.0 {
        raw
    } else {
        1.0
    };
    let scale = if axis_parallel {
        // The map's rounding (a scale of 1 + 1 ulp for a translation) must not
        // change the chunk grid from frame to frame.
        (raw * SCALE_SNAP).round() / SCALE_SNAP
    } else {
        2f64.powf((2.0 * raw.log2()).ceil() / 2.0)
    };
    scale.clamp(MIN_SCALE, MAX_SCALE)
}

/// The intersection of a surface rectangle with the visible rectangle, grown by the reach.
fn visible_part(base: Rect, h: &Projective, target: Rect, reach: f64) -> Option<Rect> {
    let Some(inv) = h.invert() else {
        return Some(base);
    };
    let corners = [
        Point::new(target.x, target.y),
        Point::new(target.right(), target.y),
        Point::new(target.right(), target.bottom()),
        Point::new(target.x, target.bottom()),
    ];
    let mut mapped = Vec::with_capacity(4);
    for corner in corners {
        match inv.apply(corner) {
            Some(p) => mapped.push(p),
            None => return Some(base),
        }
    }
    let visible = points_bounds(&mapped).inflate(reach, reach);
    super::bounds::intersect(base, visible)
}

/// Whether a surface's map is the whole window's: axis-parallel with positive scales.
fn axis_parallel(h: &Projective) -> Option<Transform> {
    h.as_affine()
        .filter(|t| t.b.abs() < 1e-12 && t.c.abs() < 1e-12 && t.a > 0.0 && t.d > 0.0)
}

/// The hash of a layer's own drawing parameters and window mapping.
fn own_hash(layer: &CompositeLayer, to_win: &Matrix4, clips: &[Shape]) -> u64 {
    let mut h = Fnv::new();
    hash::layer_key(&mut h, layer.key);
    h.rect(layer.bounds);
    h.tag(layer.plane as u8);
    hash::backface(&mut h, layer.backface);
    h.matrix4(to_win);
    h.len(clips.len());
    for clip in clips {
        hash::shape(&mut h, clip);
    }
    let mut retain = Vec::new();
    hash::effects(&mut h, &layer.effects, &mut retain);
    h.finish()
}

/// The shared allocations a layer record keeps alive: its effects' and its items' pixels.
fn layer_retain(layer: &CompositeLayer) -> Vec<Retained> {
    let mut retain = Vec::new();
    let mut h = Fnv::new();
    hash::effects(&mut h, &layer.effects, &mut retain);
    for item in &layer.items {
        match item {
            LayerItem::External(e) => hash::external(&mut h, e, &mut retain),
            LayerItem::Shader(s) => hash::shader_item(&mut h, s, &mut retain),
            _ => {}
        }
    }
    retain
}

impl<'a> Builder<'a> {
    /// The geometry of `layer` drawn at `to_t` into a target of `tsize`; None when it is culled.
    pub(super) fn geometry(
        &self,
        layer: &CompositeLayer,
        to_t: &Matrix4,
        tsize: (u32, u32),
    ) -> Option<Geo> {
        let reach = reach_of(&layer.effects, layer.bounds);
        let base = layer.bounds.inflate(reach, reach);
        let h = to_t.flatten();
        let target = Rect::new(0.0, 0.0, f64::from(tsize.0), f64::from(tsize.1));
        let reached = h.map_rect(base)?.bounds()?;
        if !reached.intersects(target) {
            return None;
        }
        let mut rect = visible_part(base, &h, target, reach)?;
        let affine = axis_parallel(&h);
        let mut scale = quantize(h.max_scale(rect), affine.is_some());
        if layer.grid == RasterGrid::Layer {
            let cell = LAYER_GRID_CELL_PX / scale;
            let x0 = (rect.x / cell).floor() * cell;
            let y0 = (rect.y / cell).floor() * cell;
            let x1 = (rect.right() / cell).ceil() * cell;
            let y1 = (rect.bottom() / cell).ceil() * cell;
            rect = super::bounds::intersect(base, Rect::new(x0, y0, x1 - x0, y1 - y0))?;
        }
        let mut origin_px = None;
        if let Some(t) = affine {
            let inv = h.invert()?;
            match layer.grid {
                RasterGrid::Target => {
                    // Snap the origin so its px is whole: crisp, moved only when the fraction changes.
                    let p = h.apply(Point::new(rect.x, rect.y))?;
                    let snapped = inv.apply(Point::new(p.x.floor(), p.y.floor()))?;
                    rect.x = snapped.x;
                    rect.y = snapped.y;
                }
                RasterGrid::Layer => {
                    rect.x = (rect.x * scale).floor() / scale;
                    rect.y = (rect.y * scale).floor() / scale;
                }
            }
            let p = h.apply(Point::new(rect.x, rect.y))?;
            origin_px = Some((p, t));
        }
        let size = loop {
            let w = ceil_px(rect.width * scale);
            let hh = ceil_px(rect.height * scale);
            if !(w.is_finite() && hh.is_finite()) || w <= 0.0 || hh <= 0.0 {
                return None;
            }
            if w <= MAX_SURFACE_PX && hh <= MAX_SURFACE_PX {
                break (w as u32, hh as u32);
            }
            scale /= SQRT_2;
        };
        let nearest = origin_px.is_some_and(|(p, t)| {
            let whole = |v: f64| (v - v.round()).abs() < 1e-6;
            (t.a - scale).abs() < 1e-9 && (t.d - scale).abs() < 1e-9 && whole(p.x) && whole(p.y)
        });
        let ms = Matrix4::translate(-rect.x, -rect.y, 0.0).then(Matrix4::scale(scale, scale, 1.0));
        Some(Geo {
            h,
            rect,
            scale,
            size,
            ms,
            nearest,
        })
    }

    /// The signature of a layer drawn at `to_t` (memoised per layer and map): its content, scale and rectangle,
    /// folding in its children's signatures.
    pub(super) fn signature(
        &mut self,
        index: u32,
        to_t: &Matrix4,
        tsize: (u32, u32),
    ) -> Result<u64, PlanError> {
        let mut key = Fnv::new();
        key.matrix4(to_t);
        key.u64(u64::from(tsize.0));
        key.u64(u64::from(tsize.1));
        let key = key.finish();
        if let Some((known, sig)) = self.signatures.get(&index)
            && *known == key
        {
            return Ok(*sig);
        }
        let layer = self.layer(index)?;
        let geo = self.geometry(layer, to_t, tsize);
        let mut h = Fnv::new();
        hash::layer_key(&mut h, layer.key);
        h.rect(layer.bounds);
        h.tag(layer.plane as u8);
        hash::backface(&mut h, layer.backface);
        h.tag(layer.grid as u8);
        match geo {
            Some(g) => {
                h.tag(1);
                h.rect(g.rect);
                h.f64(g.scale);
                h.u64(u64::from(g.size.0));
                h.u64(u64::from(g.size.1));
            }
            None => h.tag(0),
        }
        let mut retain = Vec::new();
        hash::effects(&mut h, &layer.effects, &mut retain);
        for item in &layer.items {
            match item {
                LayerItem::Raster(scene) => {
                    h.tag(1);
                    for c in hash::scene_hashes(scene, &mut retain) {
                        h.u64(c);
                    }
                }
                LayerItem::Layer {
                    index: child,
                    clips,
                } => {
                    h.tag(2);
                    let child_layer = self.layer(*child)?;
                    h.matrix4(&child_layer.transform);
                    h.len(clips.len());
                    for clip in clips {
                        hash::shape(&mut h, clip);
                    }
                    if let Some(g) = geo {
                        let child_to = child_layer.transform.then(g.ms);
                        let child_sig = self.signature(*child, &child_to, g.size)?;
                        h.u64(child_sig);
                    }
                }
                LayerItem::External(e) => {
                    h.tag(3);
                    hash::external(&mut h, e, &mut retain);
                }
                LayerItem::Shader(s) => {
                    h.tag(4);
                    hash::shader_item(&mut h, s, &mut retain);
                }
            }
        }
        let sig = h.finish();
        self.signatures.insert(index, (key, sig));
        Ok(sig)
    }

    /// Plan a layer drawn into `dest` (its target, map and size, with an inherited cover), with `clips` on its item.
    pub(super) fn emit_layer(
        &mut self,
        index: u32,
        dest: Dest,
        to_win: Matrix4,
        clips: &'a [Shape],
    ) -> Result<(), PlanError> {
        let Dest {
            target: t,
            to: to_t,
            size: tsize,
            cover,
        } = dest;
        let layer = self.layer(index)?;
        if layer.backface == BackfaceVisibility::Hidden && layer.transform.shows_back() {
            return Ok(());
        }
        if layer.plane == Plane::Preserve3d {
            return self.emit_context(index, dest, to_win, clips);
        }
        let Some(geo) = self.geometry(layer, &to_t, tsize) else {
            return Ok(());
        };
        let mark = self.damage.len();
        let sig = self.signature(index, &to_t, tsize)?;
        let inline = self.inline.contains(&index)
            && inline_candidate(self.comp, index)
            && geo.h.as_affine().is_some();
        if inline {
            let cx = ItemCtx {
                layer,
                first: 0,
                items: &layer.items,
                grid: geo.grid(),
                dest: Dest {
                    target: t,
                    to: to_t,
                    size: tsize,
                    cover,
                },
                to_win,
                cache_chunks: false,
            };
            self.emit_items(&cx)?;
        } else {
            let src = self.build_surface(index, layer, &geo, &to_win, sig)?;
            self.draw_surface(layer, &geo, src, t, cover);
        }
        self.expand_effects(layer, &geo, mark, &to_win);
        let win = mapped_bounds(&to_win, geo.rect);
        self.record(layer, win, sig, &to_win, clips, inline);
        Ok(())
    }

    /// Plan a layer's surface (or its chunk) and everything that fills it, returning what the parent draws.
    pub(super) fn build_surface(
        &mut self,
        index: u32,
        layer: &'a CompositeLayer,
        geo: &Geo,
        to_win: &Matrix4,
        sig: u64,
    ) -> Result<Source<'a>, PlanError> {
        if let Some(scene) = chunk_shortcut(layer) {
            let key = ChunkKey {
                layer: layer.key,
                role: ChunkRole::Content(0),
            };
            self.chunk(key, Cow::Borrowed(scene), geo.grid(), to_win, true)?;
            return Ok(Source::Chunk(key));
        }
        let cache = self.cache;
        let reuse = self.cacheable()
            && !subtree_has_backdrop(self.comp, index)
            && cache
                .layers
                .get(&layer.key)
                .is_some_and(|r| r.signature == sig);
        let surface = if self.cacheable() {
            self.push_target(
                geo.size,
                TargetKind::Kept {
                    key: layer.key,
                    signature: sig,
                    reuse,
                },
            )
        } else {
            self.new_temp(geo.size)
        };
        if reuse {
            self.reused_surfaces += 1;
            self.reuse_subtree(index)?;
            return Ok(Source::Target(surface));
        }
        if self.cacheable() {
            self.ops.push(Op::Clear {
                target: surface,
                rect: None,
            });
        }
        let content = if layer.effects.filters.is_empty() {
            surface
        } else {
            self.new_temp(geo.size)
        };
        if let Some(backdrop) = &layer.effects.backdrop {
            self.emit_backdrop(layer, backdrop, geo, content, to_win)?;
        }
        let cx = ItemCtx {
            layer,
            first: 0,
            items: &layer.items,
            grid: geo.grid(),
            dest: Dest {
                target: content,
                to: geo.ms,
                size: geo.size,
                cover: None,
            },
            to_win: *to_win,
            cache_chunks: true,
        };
        self.behind.push(Behind {
            target: content,
            to: geo.ms,
            to_win: *to_win,
            root: layer.effects != LayerEffects::default(),
        });
        let emitted = self.emit_items(&cx);
        self.behind.pop();
        emitted?;
        self.emit_effects(layer, geo, content, surface, to_win)?;
        Ok(Source::Target(surface))
    }

    /// Draw what lies behind a backdrop layer, filter it, and draw it into the surface under the backdrop's shape.
    ///
    /// What lies behind is the content of the backdrop root and of the
    /// transparent layers between it and this layer, drawn outermost first
    /// (CONTRACT-BACKDROP-ROOT), each through its own map into this layer's px.
    fn emit_backdrop(
        &mut self,
        layer: &'a CompositeLayer,
        backdrop: &'a RenderBackdrop,
        geo: &Geo,
        content: TargetId,
        to_win: &Matrix4,
    ) -> Result<(), PlanError> {
        let size = geo.size;
        let behind = self.new_temp(size);
        let start = self.behind.iter().rposition(|b| b.root).unwrap_or(0);
        let chain: Vec<Behind> = self.behind[start..].to_vec();
        for entry in chain {
            // A draw's map runs from the destination px to the source px.
            let Some(map) = px_map(&geo.ms, to_win)
                .zip(px_map(&entry.to_win, &entry.to))
                .map(|(to_window, from_window)| to_window.then(from_window))
            else {
                continue;
            };
            self.ops.push(Op::Draw(DrawOp {
                target: behind,
                source: Source::Target(entry.target),
                polygon: full_polygon(size),
                split_edges: Vec::new(),
                map,
                opacity: 1.0,
                blend: BlendMode::Normal,
                composite: Composite::SourceOver,
                sampling: Sampling::Smooth,
                coverage: None,
                scissor: Scissor::Off,
            }));
        }
        let mut cur = behind;
        for filter in &backdrop.filters {
            let out = self.new_temp(size);
            self.ops.push(Op::Filter {
                input: cur,
                output: out,
                filter,
                px_per_unit: geo.scale,
                origin: geo.grid().origin,
                bounds: Size::new(layer.bounds.width, layer.bounds.height),
            });
            cur = out;
        }
        let key = ChunkKey {
            layer: layer.key,
            role: ChunkRole::Backdrop,
        };
        let scene = coverage_scene(std::slice::from_ref(&backdrop.shape), geo.grid().rect());
        self.chunk(key, Cow::Owned(scene), geo.grid(), to_win, true)?;
        self.ops.push(Op::Draw(DrawOp {
            target: content,
            source: Source::Target(cur),
            polygon: full_polygon(size),
            split_edges: Vec::new(),
            map: Projective::IDENTITY,
            opacity: 1.0,
            blend: BlendMode::Normal,
            composite: Composite::SourceOver,
            sampling: Sampling::Smooth,
            coverage: Some(key),
            scissor: Scissor::Off,
        }));
        if self.cacheable() {
            self.backdrops.extend(mapped_bounds(to_win, geo.rect));
        }
        Ok(())
    }

    /// Filter the surface, clip it and mask it, in the CSS order (design §8.3 step 7).
    fn emit_effects(
        &mut self,
        layer: &'a CompositeLayer,
        geo: &Geo,
        content: TargetId,
        surface: TargetId,
        to_win: &Matrix4,
    ) -> Result<(), PlanError> {
        let grid = geo.grid();
        let filters = &layer.effects.filters;
        let mut cur = content;
        for (i, filter) in filters.iter().enumerate() {
            let out = if i + 1 == filters.len() {
                surface
            } else {
                self.new_temp(geo.size)
            };
            self.ops.push(Op::Filter {
                input: cur,
                output: out,
                filter,
                px_per_unit: geo.scale,
                origin: grid.origin,
                bounds: Size::new(layer.bounds.width, layer.bounds.height),
            });
            cur = out;
        }
        if let Some(clip) = &layer.effects.clip {
            let key = ChunkKey {
                layer: layer.key,
                role: ChunkRole::LayerClip,
            };
            let scene = coverage_scene(std::slice::from_ref(clip), grid.rect());
            self.chunk(key, Cow::Owned(scene), grid, to_win, true)?;
            self.ops.push(Op::Clip {
                target: surface,
                coverage: key,
            });
        }
        if !layer.effects.masks.is_empty() {
            let mut masks = Vec::with_capacity(layer.effects.masks.len());
            for (n, mask) in layer.effects.masks.iter().enumerate() {
                let key = ChunkKey {
                    layer: layer.key,
                    role: ChunkRole::Mask(n as u32),
                };
                self.chunk(key, Cow::Borrowed(&mask.scene), grid, to_win, true)?;
                masks.push((key, mask.mode, mask.composite));
            }
            self.ops.push(Op::Mask {
                target: surface,
                masks,
            });
        }
        Ok(())
    }

    /// Draw a planned surface into its target (design §8.3 step 9).
    pub(super) fn draw_surface(
        &mut self,
        layer: &CompositeLayer,
        geo: &Geo,
        src: Source<'a>,
        t: TargetId,
        cover: Option<ChunkKey>,
    ) {
        let Some(map) = geo.h.invert().map(|inv| inv.then(geo.ms.flatten())) else {
            return;
        };
        let Some(polygon) = geo.h.map_rect(geo.rect) else {
            return;
        };
        self.ops.push(Op::Draw(DrawOp {
            target: t,
            source: src,
            polygon,
            split_edges: Vec::new(),
            map,
            opacity: layer.effects.opacity,
            blend: layer.effects.blend,
            composite: layer.effects.composite,
            sampling: Sampling::nearest(geo.nearest),
            coverage: cover,
            scissor: Scissor::Off,
        }));
    }

    /// Give a layer whose effects reach beyond its surface the damage of its whole output, when its subtree damaged.
    pub(super) fn expand_effects(
        &mut self,
        layer: &CompositeLayer,
        geo: &Geo,
        mark: usize,
        to_win: &Matrix4,
    ) {
        if !self.cacheable() {
            return;
        }
        let spreads = !layer.effects.filters.is_empty()
            || !layer.effects.masks.is_empty()
            || layer.effects.clip.is_some()
            || layer.effects.backdrop.is_some()
            || geo.h.as_affine().is_none()
            || to_win.flatten().as_affine().is_none();
        if spreads && self.damage.len() > mark {
            self.damage.truncate(mark);
            self.damage.extend(mapped_bounds(to_win, geo.rect));
        }
    }

    /// Record a layer this frame, adding the damage its change causes (own, new, or content).
    pub(super) fn record(
        &mut self,
        layer: &CompositeLayer,
        win: Option<Rect>,
        sig: u64,
        to_win: &Matrix4,
        clips: &[Shape],
        whole_on_change: bool,
    ) {
        if !self.cacheable() {
            return;
        }
        let own = own_hash(layer, to_win, clips);
        let cache = self.cache;
        match cache.layers.get(&layer.key) {
            None => self.damage.extend(win),
            Some(prev) if prev.own != own => {
                self.damage.extend(prev.window_bounds);
                self.damage.extend(win);
            }
            Some(prev) if prev.signature != sig => {
                if whole_on_change {
                    self.damage.extend(prev.window_bounds);
                    self.damage.extend(win);
                } else {
                    self.item_damage(layer, to_win);
                }
            }
            Some(_) => {}
        }
        self.layer_records.push((
            layer.key,
            LayerRecord {
                signature: sig,
                own,
                window_bounds: win,
                frame: self.frame,
                retain: layer_retain(layer),
            },
        ));
        self.seen.insert(layer.key);
    }

    /// Damage the window bounds of a layer's external and shader items (their content is not rastered into chunks).
    fn item_damage(&mut self, layer: &CompositeLayer, to_win: &Matrix4) {
        for item in &layer.items {
            match item {
                LayerItem::External(e) => {
                    let m = Matrix4::from_affine(e.transform).then(*to_win);
                    self.damage.extend(mapped_bounds(&m, e.rect));
                }
                LayerItem::Shader(s) => {
                    let m = Matrix4::from_affine(s.transform).then(*to_win);
                    self.damage.extend(mapped_bounds(&m, s.rect));
                }
                _ => {}
            }
        }
    }

    /// Mark a reused surface's descendants: their records and chunks are carried over unchanged.
    pub(super) fn reuse_subtree(&mut self, index: u32) -> Result<(), PlanError> {
        let layer = self.layer(index)?;
        self.reused_chunks.insert(layer.key);
        for item in &layer.items {
            if let LayerItem::Layer { index: child, .. } = item {
                let key = self.layer(*child)?.key;
                self.reused.insert(key);
                self.reused_chunks.insert(key);
                self.seen.insert(key);
                self.reuse_subtree(*child)?;
            }
        }
        Ok(())
    }
}
