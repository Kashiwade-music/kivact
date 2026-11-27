//! Role: Shared plan types and free helpers for the planning leaves.

use arvane_protocol::{
    CompositeLayer, Composition, Group, LayerEffects, LayerFilter, LayerItem, PixelRect, Plane,
    RasterPaint, RenderCmd, RenderScene,
};
use kivact_types::Matrix4;
use kivact_types::geometry::{Affine, Point, Rect};
use kivact_types::geometry3d::{Polygon, Projective};
use kivact_types::pixel::BlendMode;
use kivact_types::scene::Outline;

use super::ops::{Grid, TargetId};
use crate::cache::ChunkKey;

/// Where a set of items draws: its target, the map from layer space to the
/// target's px, the target's size, and the clip that covers every draw.
#[derive(Clone, Copy, Debug)]
pub(super) struct Dest {
    /// The target drawn into.
    pub(super) target: TargetId,
    /// Layer space → target px.
    pub(super) to: Matrix4,
    /// The target's size in px.
    pub(super) size: (u32, u32),
    /// Coverage inherited from an inline ancestor's clip, applied to every draw.
    pub(super) cover: Option<ChunkKey>,
}

/// A target whose current content lies behind the items drawn after it:
/// the content of an enclosing layer, outermost first.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Behind {
    /// The target holding the content.
    pub(crate) target: TargetId,
    /// Layer space → the target's px.
    pub(crate) to: Matrix4,
    /// Layer space → window px of the layer that draws the content.
    pub(crate) to_win: Matrix4,
    /// Whether the layer that holds the content is a backdrop root (effects not default, or the output).
    pub(crate) root: bool,
}

/// Rounds a px quantity up to a whole px, ignoring the rounding error of the
/// maps that produced it: a scale of 1 + 1 ulp must not add a px.
pub(crate) fn ceil_px(v: f64) -> f64 {
    (v - PX_EPSILON).ceil()
}

/// Rounds a px quantity down to a whole px, ignoring the rounding error of the maps that produced it.
pub(crate) fn floor_px(v: f64) -> f64 {
    (v + PX_EPSILON).floor()
}

/// The error the px rounding forgives (far below any visible px).
const PX_EPSILON: f64 = 1e-6;

/// The layer and items being drawn.
pub(super) struct ItemCtx<'a> {
    /// The layer that owns the items.
    pub(super) layer: &'a CompositeLayer,
    /// Position of `items[0]` in `layer.items`.
    pub(super) first: usize,
    /// The items to draw, in order.
    pub(super) items: &'a [LayerItem],
    /// The grid of the layer's chunks.
    pub(super) grid: Grid,
    /// Where the items draw.
    pub(super) dest: Dest,
    /// Layer space → window px, for damage.
    pub(super) to_win: Matrix4,
    /// Whether the rasters are cached (false for inline layers, which draw into their parent).
    pub(super) cache_chunks: bool,
}

/// The number of Raster items before position `at` of the layer (the chunk ordinal of a raster).
pub(super) fn raster_ordinal(layer: &CompositeLayer, at: usize) -> u32 {
    layer.items[..at]
        .iter()
        .filter(|item| matches!(item, LayerItem::Raster(_)))
        .count() as u32
}

/// A white coverage scene: the rectangle `rect` clipped by every shape of `clips`.
pub(super) fn coverage_scene(clips: &[Outline], rect: Rect) -> RenderScene {
    let white = kivact_types::Rgba {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    let mut commands = Vec::with_capacity(clips.len() * 2 + 3);
    for clip in clips {
        commands.push(RenderCmd::PushGroup(Group {
            clip: Some(clip.clone()),
            opacity: 1.0,
            blend: BlendMode::Normal,
            blur: None,
        }));
    }
    commands.push(RenderCmd::FillShape {
        shape: Outline::Rect(rect),
        paint: RasterPaint::Solid(white),
    });
    commands.extend(clips.iter().map(|_| RenderCmd::PopGroup));
    RenderScene { commands }
}

/// A scene placed by `t` in its owner's space: the commands wrapped in a transform.
pub(super) fn wrapped(scene: &RenderScene, t: Affine) -> RenderScene {
    let mut commands = Vec::with_capacity(scene.commands.len() + 2);
    commands.push(RenderCmd::PushTransform(t));
    commands.extend(scene.commands.iter().cloned());
    commands.push(RenderCmd::PopTransform);
    RenderScene { commands }
}

/// Whether a layer may be drawn into its parent instead of a surface,
/// ignoring the runtime affine test. The rules keep each draw's coverage and
/// shader grid in its parent's space.
pub(super) fn inline_candidate(comp: &Composition, index: u32) -> bool {
    let Some(layer) = comp.layers.get(index as usize) else {
        return false;
    };
    if index == 0
        || layer.plane != Plane::Flat
        || layer.effects != LayerEffects::default()
        || subtree_has_backdrop(comp, index)
    {
        return false;
    }
    layer.items.iter().all(|item| match item {
        LayerItem::Raster(scene) => scene
            .commands
            .iter()
            .all(|c| !matches!(c, RenderCmd::PushGroup(g) if g.blend != BlendMode::Normal)),
        LayerItem::External(external) => external.clips.is_empty(),
        LayerItem::Shader(_) => false,
        LayerItem::Layer { clips, .. } => clips.is_empty(),
    })
}

/// Whether a layer or one of its descendants has a backdrop.
pub(super) fn subtree_has_backdrop(comp: &Composition, index: u32) -> bool {
    let Some(layer) = comp.layers.get(index as usize) else {
        return false;
    };
    layer.effects.backdrop.is_some()
        || layer.items.iter().any(|item| match item {
            LayerItem::Layer { index, .. } => subtree_has_backdrop(comp, *index),
            _ => false,
        })
}

/// The area of a pixel rectangle.
pub(super) fn area(r: &PixelRect) -> u64 {
    u64::from(r.width) * u64::from(r.height)
}

/// The map from the px of a space `from` to the px of a space `to`, both given as maps from layer space.
pub(crate) fn px_map(from: &Matrix4, to: &Matrix4) -> Option<Projective> {
    Some(from.flatten().invert()?.then(to.flatten()))
}

/// The integer translation `(tx, ty)` of `map` (target px to texel), when it is one.
pub(crate) fn integer_translation(map: &Projective) -> Option<(i64, i64)> {
    let m = &map.m;
    let unit = m[0][0] == 1.0
        && m[0][1] == 0.0
        && m[1][0] == 0.0
        && m[1][1] == 1.0
        && m[2][0] == 0.0
        && m[2][1] == 0.0
        && m[2][2] == 1.0;
    let (tx, ty) = (m[0][2], m[1][2]);
    (unit && tx.is_finite() && ty.is_finite() && tx.fract() == 0.0 && ty.fract() == 0.0)
        .then_some((tx as i64, ty as i64))
}

/// The pixel rectangle `[x0, x1) × [y0, y1)` of `points` when they are the
/// four distinct corners of an axis-aligned rectangle on integer coordinates.
pub(crate) fn pixel_rect(points: &[Point]) -> Option<(i64, i64, i64, i64)> {
    if points.len() != 4
        || points
            .iter()
            .any(|p| p.x.fract() != 0.0 || p.y.fract() != 0.0)
    {
        return None;
    }
    let x0 = points.iter().map(|p| p.x).fold(f64::INFINITY, f64::min);
    let x1 = points.iter().map(|p| p.x).fold(f64::NEG_INFINITY, f64::max);
    let y0 = points.iter().map(|p| p.y).fold(f64::INFINITY, f64::min);
    let y1 = points.iter().map(|p| p.y).fold(f64::NEG_INFINITY, f64::max);
    let corner = |x: f64, y: f64| points.iter().any(|p| p.x == x && p.y == y);
    let all = corner(x0, y0) && corner(x1, y0) && corner(x1, y1) && corner(x0, y1);
    (all && x0 < x1 && y0 < y1).then_some((x0 as i64, x1 as i64, y0 as i64, y1 as i64))
}

/// The filled rectangle of a size, as a polygon in its own px.
pub(crate) fn full_polygon(size: (u32, u32)) -> Polygon {
    let (w, h) = (f64::from(size.0), f64::from(size.1));
    Polygon {
        points: vec![
            Point::new(0.0, 0.0),
            Point::new(w, 0.0),
            Point::new(w, h),
            Point::new(0.0, h),
        ],
    }
}

/// The filter's reach beyond a layer's bounds, in layer units (used by the cull and the surface size).
pub(crate) fn filter_reach(filter: &LayerFilter, bounds: Rect) -> f64 {
    match filter {
        LayerFilter::Blur { std_dev } => 3.0 * std_dev,
        LayerFilter::DropShadow {
            offset, std_dev, ..
        } => offset.x.hypot(offset.y) + 3.0 * std_dev,
        LayerFilter::ColorMatrix(_) => 0.0,
        LayerFilter::Graph(graph) => match graph.margin {
            Some(m) => m.top.max(m.right).max(m.bottom).max(m.left),
            None => 0.1 * bounds.width.max(bounds.height),
        },
        LayerFilter::Shader(shader) => shader.margin,
    }
}
