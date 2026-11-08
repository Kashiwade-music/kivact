//! Role: Shared plan types and free helpers for the planning leaves.

use kivact_types::Matrix4;
use kivact_types::geometry::{Point, Rect, Transform};
use kivact_types::geometry3d::{Polygon, Projective};
use kivact_types::pixel::BlendMode;
use kivact_types::render::{
    CompositeLayer, Composition, Group, LayerEffects, LayerFilter, LayerItem, Paint, PixelRect,
    Plane, RenderCmd, RenderScene,
};
use kivact_types::scene::Shape;

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
pub(super) fn coverage_scene(clips: &[Shape], rect: Rect) -> RenderScene {
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
        shape: Shape::Rect(rect),
        paint: Paint::Solid(white),
    });
    commands.extend(clips.iter().map(|_| RenderCmd::PopGroup));
    RenderScene { commands }
}

/// A scene placed by `t` in its owner's space: the commands wrapped in a transform.
pub(super) fn wrapped(scene: &RenderScene, t: Transform) -> RenderScene {
    let mut commands = Vec::with_capacity(scene.commands.len() + 2);
    commands.push(RenderCmd::PushTransform(t));
    commands.extend(scene.commands.iter().cloned());
    commands.push(RenderCmd::PopTransform);
    RenderScene { commands }
}

/// Whether a layer may be drawn into its parent instead of a surface (design §8.14),
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
