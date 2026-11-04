//! Role: A node's effects as Arvane values: tint, layers and backdrop.
//!
//! A node's effects as Arvane values: the color filter every color of its
//! subtree goes through ([`Tint`]), the layers it is composited in (group
//! opacity, filters, clip path, mask, blending) and its backdrop.
//!
//! Invariants:
//! - Color functions (brightness .. opacity) compose into one
//!   `ColorMatrix`, applied to every color the subtree emits (paints and
//!   gradient stops, borders, shadows, text, scenes) and to its images
//!   through the image's matrix; Arvane is never asked to filter pixels by
//!   a matrix. A node's matrix applies after its descendants' (CSS).
//! - `blur` and `drop-shadow` are layer filters, nested in CSS order (the
//!   first innermost); a drop shadow's color goes through the color
//!   functions after it and the ancestors' matrices.
//! - One layer carries the clip path, the mask, the group opacity and the
//!   blend mode, outermost, so they apply after the filters (CSS order:
//!   filter, clip, mask, opacity, blend).

use kivact_types::Point;
use kivact_types::{BlendMode, FilterOp, PaintProps, Rgba};

use kivact_types::protocol::{ColorMatrix, Layer, LayerFilter};

/// How the colors of one node are drawn: faded by `alpha` (the opacity of
/// the node and its ancestors that is not a group's) and through `matrix`
/// (its and its ancestors' color filters).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tint {
    /// Accumulated non-group opacity multiplier.
    pub alpha: f32,
    /// Accumulated color transform, applied before the opacity multiplier.
    pub matrix: Option<ColorMatrix>,
}

impl Tint {
    /// A shared color as drawn.
    pub fn color(&self, color: Rgba) -> Rgba {
        self.arvane(crate::sanitize::color(color))
    }

    /// Apply matrix and opacity to an already-lowered neutral color,
    /// without repeating shared component sanitization.
    pub fn arvane(&self, color: Rgba) -> Rgba {
        let color = match &self.matrix {
            Some(m) => m.apply(color),
            None => color,
        };
        Rgba {
            a: color.a * self.alpha,
            ..color
        }
    }

    /// A paint as drawn (every stop, a pattern's image through the matrix).
    pub(crate) fn paint(
        &self,
        paint: kivact_types::protocol::Paint,
    ) -> kivact_types::protocol::Paint {
        let paint = match &self.matrix {
            Some(m) => paint.filtered(m),
            None => paint,
        };
        if self.alpha == 1.0 {
            paint
        } else {
            paint.filtered(&ColorMatrix::opacity(self.alpha))
        }
    }

    /// The tint of a child: its own color filter (`own`) first.
    pub(crate) fn then_own(&self, own: Option<ColorMatrix>) -> Option<ColorMatrix> {
        match (own, self.matrix) {
            (Some(own), Some(parent)) => Some(own.then(parent)),
            (own, parent) => own.or(parent),
        }
    }
}

fn op_matrix(op: &FilterOp) -> Option<ColorMatrix> {
    let finite = |v: f32, fallback: f32| if v.is_finite() { v } else { fallback };
    Some(match *op {
        FilterOp::Brightness(a) => ColorMatrix::brightness(finite(a, 1.0)),
        FilterOp::Contrast(a) => ColorMatrix::contrast(finite(a, 1.0)),
        FilterOp::Saturate(a) => ColorMatrix::saturate(finite(a, 1.0)),
        FilterOp::Grayscale(a) => ColorMatrix::grayscale(finite(a, 0.0)),
        FilterOp::Sepia(a) => ColorMatrix::sepia(finite(a, 0.0)),
        FilterOp::HueRotate(a) => ColorMatrix::hue_rotate(finite(a, 0.0)),
        FilterOp::Invert(a) => ColorMatrix::invert(finite(a, 0.0)),
        FilterOp::Opacity(a) => ColorMatrix::opacity(finite(a, 1.0)),
        FilterOp::Blur(_) | FilterOp::DropShadow { .. } => return None,
    })
}

/// The color functions of `ops`, composed in order (`None`: none, or only
/// ones that change nothing).
pub fn color_matrix(ops: &[FilterOp]) -> Option<ColorMatrix> {
    let m = ops.iter().filter_map(op_matrix).reduce(ColorMatrix::then)?;
    (!m.is_identity()).then_some(m)
}

/// A blur's standard deviation (CSS `blur(px)`), sanitized.
fn blur(px: f32) -> f32 {
    if px.is_finite() { px.max(0.0) } else { 0.0 }
}

/// How far the layer filters of `ops` can draw beyond what they filter.
pub fn filter_reach(ops: &[FilterOp]) -> f32 {
    ops.iter()
        .map(|op| match *op {
            FilterOp::Blur(px) => blur(px) * kivact_types::protocol::SHADOW_REACH,
            FilterOp::DropShadow {
                offset, blur: b, ..
            } => {
                let offset = offset.x.abs().max(offset.y.abs());
                (if offset.is_finite() { offset } else { 0.0 })
                    + blur(b) / 2.0 * kivact_types::protocol::SHADOW_REACH
            }
            _ => 0.0,
        })
        .sum()
}

/// Whether `op` is a layer filter that changes something.
fn is_layer_filter(op: &FilterOp) -> bool {
    match *op {
        FilterOp::Blur(px) => blur(px) > 0.0,
        FilterOp::DropShadow { .. } => true,
        _ => false,
    }
}

/// How many layers the effects of `p` need, whatever its opacity (one
/// per layer filter, at least one for a clip path, a mask or a blend
/// mode; [`layers`] makes them).
pub fn layer_count(p: &PaintProps) -> u8 {
    let Some(e) = &p.effects else {
        return 0;
    };
    let filters = e.filter.iter().filter(|op| is_layer_filter(op)).count();
    let other = e.blend != BlendMode::Normal || e.mask.is_some() || e.clip_path.is_some();
    let n = if other { filters.max(1) } else { filters };
    n.min(usize::from(u8::MAX)) as u8
}

/// The layers a node is composited in, outermost first (see the module
/// invariants): `outer` carries the clip, the mask, the group opacity
/// (`opacity`) and the blend mode; the layer filters follow, the last one
/// outermost. Drop shadow colors go through `parent` (the ancestors'
/// matrix) after the node's later color functions.
pub fn layers(
    p: &PaintProps,
    mut outer: Layer,
    opacity: f32,
    parent: Option<&ColorMatrix>,
) -> Vec<Layer> {
    outer.opacity = opacity;
    let mut filters: Vec<LayerFilter> = Vec::new();
    if let Some(e) = &p.effects {
        outer.blend = e.blend;
        for (i, op) in e.filter.iter().enumerate() {
            if !is_layer_filter(op) {
                continue;
            }
            match *op {
                FilterOp::Blur(px) => filters.push(LayerFilter::Blur { std_dev: blur(px) }),
                FilterOp::DropShadow {
                    offset,
                    blur: b,
                    color,
                } => {
                    let later = color_matrix(&e.filter[i + 1..]);
                    let tint = Tint {
                        alpha: 1.0,
                        matrix: Tint {
                            alpha: 1.0,
                            matrix: parent.copied(),
                        }
                        .then_own(later),
                    };
                    let finite = |v: f32| if v.is_finite() { v } else { 0.0 };
                    filters.push(LayerFilter::DropShadow {
                        offset: Point {
                            x: finite(offset.x),
                            y: finite(offset.y),
                        },
                        std_dev: blur(b) / 2.0,
                        color: tint.color(color),
                    });
                }
                _ => {}
            }
        }
    }
    let mut out = Vec::with_capacity(filters.len().max(1));
    let last = filters.pop();
    outer.filter = last;
    out.push(outer);
    out.extend(filters.into_iter().rev().map(|f| Layer {
        filter: Some(f),
        ..Layer::default()
    }));
    out
}

/// The backdrop of a node with a `backdrop-filter`, behind `shape`: its
/// blur and its color functions, then the ancestors' matrix (`parent`).
pub fn backdrop(
    p: &PaintProps,
    shape: kivact_types::protocol::Shape,
    parent: Option<&ColorMatrix>,
) -> Option<kivact_types::protocol::Backdrop> {
    let ops = &p.effects.as_ref()?.backdrop;
    if ops.is_empty() {
        return None;
    }
    let std_dev = ops
        .iter()
        .map(|op| match *op {
            FilterOp::Blur(px) => blur(px),
            _ => 0.0,
        })
        .sum();
    let parent = Tint {
        alpha: 1.0,
        matrix: parent.copied(),
    };
    Some(kivact_types::protocol::Backdrop {
        shape,
        std_dev,
        color_matrix: parent.then_own(color_matrix(ops)),
    })
}

/// How far a backdrop blurs what lies around its shape (what it reads
/// beyond the shape).
pub fn backdrop_reach(p: &PaintProps) -> f32 {
    p.effects.as_ref().map_or(0.0, |e| {
        e.backdrop
            .iter()
            .map(|op| match *op {
                FilterOp::Blur(px) => blur(px) * kivact_types::protocol::SHADOW_REACH,
                _ => 0.0,
            })
            .sum()
    })
}
