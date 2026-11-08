//! Role: The layer effects of CSS values (filters, masks, backdrops) and the
//! shader items of shader fills.
//!
//! Invariants:
//! - A color filter becomes a layer color matrix, in order; `blur` and
//!   `drop-shadow` become layer filters; a filter graph takes its images as
//!   pixels at their natural size; a filter shader is a shader use.
//! - A shader's images are their natural pixels and its time is its clock's
//!   (a still shader, and a non-essential one under reduced motion, is at 0).
//!   A shader whose images are not ready draws its fallback in place.
//! - A mask is the scene of its fill over its area. A shader mask is not
//!   supported and is left out.
//! - A shader fill is an item: its coverage (white where the shader shows)
//!   and its fallback are scenes in the item's space, inside the groups and
//!   clips open at the item.

use std::sync::Arc;

use arvane::asset::PixelSize;
use kivact_types::effect::ShaderUse;
use kivact_types::paint::{
    Fill, FillImage, FillLayer, FilterOp, MaskLayer, ShaderClock, ShaderSpec,
};
use kivact_types::pixel::ColorMatrix;
use kivact_types::render::{
    LayerFilter, LayerItem, Paint, RenderBackdrop, RenderCmd as D, RenderMask, RenderScene,
    ShaderItem,
};
use kivact_types::scene::Shape;
use kivact_types::{Asset, Pixels, Point, Rect, Rgba};

use crate::lower::SceneOut;
use crate::lower::context::Context;
use crate::lower::effects::Tint;
use crate::lower::fill::Boxes;
use crate::lower::sanitize;

/// A CSS blur radius as a standard deviation, sanitized.
fn blur_px(px: f64) -> f64 {
    if px.is_finite() { px.max(0.0) } else { 0.0 }
}

/// The color matrix of a color filter function (`None` for the others).
fn color_matrix(op: &FilterOp) -> Option<ColorMatrix> {
    Some(match *op {
        FilterOp::Brightness(a) => ColorMatrix::brightness(a),
        FilterOp::Contrast(a) => ColorMatrix::contrast(a),
        FilterOp::Saturate(a) => ColorMatrix::saturate(a),
        FilterOp::Grayscale(a) => ColorMatrix::grayscale(a),
        FilterOp::Sepia(a) => ColorMatrix::sepia(a),
        FilterOp::HueRotate(deg) => ColorMatrix::hue_rotate(deg),
        FilterOp::Invert(a) => ColorMatrix::invert(a),
        FilterOp::Opacity(a) => ColorMatrix::opacity(a),
        FilterOp::Blur(_)
        | FilterOp::DropShadow { .. }
        | FilterOp::Graph(_)
        | FilterOp::Shader(_) => return None,
    })
}

/// Whether `fill` has a shader layer.
pub(crate) fn has_shader(fill: &Fill) -> bool {
    matches!(fill, Fill::Layers(layers) if layers.iter().any(|l| matches!(l.image, FillImage::Shader(_))))
}

/// A scene drawn by `draw` into an empty output (no items are made there).
fn scene_of(draw: impl FnOnce(&mut SceneOut)) -> RenderScene {
    let mut out = SceneOut::default();
    draw(&mut out);
    out.into_scene()
}

impl Context<'_> {
    /// The layer filters of the CSS filter functions `ops`, in order.
    pub(crate) fn layer_filters(&self, ops: &[FilterOp]) -> Vec<LayerFilter> {
        ops.iter().filter_map(|op| self.layer_filter(op)).collect()
    }

    /// The layer filter of one CSS filter function (`None`: dropped).
    fn layer_filter(&self, op: &FilterOp) -> Option<LayerFilter> {
        Some(match op {
            FilterOp::Blur(px) => LayerFilter::Blur {
                std_dev: blur_px(*px),
            },
            FilterOp::DropShadow {
                offset,
                blur,
                color,
            } => LayerFilter::DropShadow {
                offset: Point {
                    x: sanitize::offset(offset.x),
                    y: sanitize::offset(offset.y),
                },
                std_dev: blur_px(*blur) / 2.0,
                color: *color,
            },
            FilterOp::Graph(graph) => {
                LayerFilter::Graph(Arc::new(graph.map_images(|asset| self.graph_pixels(asset))))
            }
            FilterOp::Shader(spec) => LayerFilter::Shader(self.shader_use(spec)?),
            other => LayerFilter::ColorMatrix(color_matrix(other)?),
        })
    }

    /// The masks of a layer over `area` (layer space), one per mask layer
    /// that paints. A shader mask is not drawn: it is counted as dropped.
    pub(crate) fn layer_masks(&self, masks: &[MaskLayer], area: Rect) -> Vec<RenderMask> {
        masks
            .iter()
            .filter_map(|mask| {
                if matches!(mask.fill.image, FillImage::Shader(_)) {
                    self.drop_command();
                    return None;
                }
                let origin = Point {
                    x: area.x,
                    y: area.y,
                };
                let paint = self.layer_paint(&mask.fill, area, origin)?;
                let scene = scene_of(|out| {
                    out.push(D::RoundedRect {
                        rect: area,
                        radius: 0.0,
                        fill: paint,
                    });
                });
                Some(RenderMask {
                    scene,
                    mode: mask.mode,
                    composite: mask.composite,
                })
            })
            .collect()
    }

    /// The backdrop of a layer clipped to `shape` (layer space), filtered by
    /// the CSS `backdrop-filter` `ops` (`None`: no backdrop).
    pub(crate) fn layer_backdrop(&self, ops: &[FilterOp], shape: Shape) -> Option<RenderBackdrop> {
        (!ops.is_empty()).then(|| RenderBackdrop {
            shape,
            filters: self.layer_filters(ops),
        })
    }

    /// Fill `boxes` with `fill` as [`Context::fill`] does, except that each
    /// shader layer is an item of its own, in the layers' order.
    pub(crate) fn fill_items(
        &self,
        out: &mut SceneOut,
        boxes: &Boxes,
        fill: &Fill,
        tint: &Tint,
        origin: Point,
    ) {
        let Fill::Layers(layers) = fill else {
            return self.fill(out, boxes, fill, tint, origin);
        };
        if !has_shader(fill) {
            return self.fill(out, boxes, fill, tint, origin);
        }
        for layer in layers.iter().rev() {
            match &layer.image {
                FillImage::Shader(spec) => self.shader_item(out, boxes, layer, spec, tint, origin),
                _ => self.fill(out, boxes, &Fill::Layers(vec![layer.clone()]), tint, origin),
            }
        }
    }

    /// One shader layer over its clip box as a shader item, its coverage and
    /// its fallback in the item's space; the fallback alone while the
    /// shader's images are not ready.
    fn shader_item(
        &self,
        out: &mut SceneOut,
        boxes: &Boxes,
        layer: &FillLayer,
        spec: &ShaderSpec,
        tint: &Tint,
        origin: Point,
    ) {
        let Some(shape) = boxes.clip(layer.clip) else {
            return;
        };
        let fallback = scene_of(|tmp| {
            if let Some(image) = &spec.fallback {
                let as_layer = FillLayer {
                    image: image.clone(),
                    ..layer.clone()
                };
                self.fill(tmp, boxes, &Fill::Layers(vec![as_layer]), tint, origin);
            }
        });
        let Some(shader) = self.shader_use(spec) else {
            out.extend(fallback.commands);
            return;
        };
        let coverage = scene_of(|tmp| {
            shape.fill(
                tmp,
                Paint::Solid(Rgba {
                    r: 1.0,
                    g: 1.0,
                    b: 1.0,
                    a: 1.0,
                }),
            );
        });
        let item = ShaderItem {
            shader,
            rect: shape.rect(),
            transform: out.space(),
            coverage: out.within_open(coverage),
            fallback: out.within_open(fallback),
            opacity: tint.alpha,
        };
        out.item(LayerItem::Shader(item));
    }

    /// The shader use of `spec` at its clock's time with its images at their
    /// natural pixels (`None` while an image is not ready). A still clock, or
    /// reduced motion for a shader that is not essential, is at time 0.
    pub(crate) fn shader_use(&self, spec: &ShaderSpec) -> Option<ShaderUse> {
        let mut images = [None, None, None, None];
        for (slot, asset) in spec.images.iter().enumerate() {
            if let Some(asset) = asset {
                images[slot] = Some(self.natural_pixels(asset)?);
            }
        }
        let time = match spec.clock {
            ShaderClock::Still => 0.0,
            _ if self.time.reduced && !spec.essential => 0.0,
            _ => self.time.seconds,
        };
        Some(ShaderUse {
            program: spec.program.clone(),
            params: spec.params.clone(),
            images,
            time,
            margin: spec.margin,
        })
    }

    /// The pixels of `asset` at its natural size (`None` while not ready).
    fn natural_pixels(&self, asset: &Asset) -> Option<Pixels> {
        let size = self.natural(asset)?;
        self.pixels(asset, PixelSize::of(size, 1.0), 0)
            .map(|(pixels, _)| pixels)
    }

    /// The pixels of a filter graph's image: a transparent pixel while it is
    /// not ready.
    fn graph_pixels(&self, asset: &Asset) -> Pixels {
        self.natural_pixels(asset)
            .unwrap_or_else(|| Pixels::new(1, 1, vec![0u8; 4]))
    }
}
