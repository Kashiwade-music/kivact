//! Role: A node's effects as Arvane values: tint, group and layer reach.
//!
//! A node's effects as Arvane values: the opacity every color of its
//! subtree is faded by ([`Tint`]) and the group it is composited in (group
//! opacity, clip path, blend mode).
//!
//! Invariants:
//! - Color filters (brightness .. opacity), layer filters (`blur`,
//!   `drop-shadow`), masks and `backdrop-filter` need the pixels of a
//!   compositor layer; the paint model does not lower them.
//! - One group carries the clip path, the group opacity and the blend
//!   mode (CSS order: clip, opacity, blend).

use kivact_types::render::{Group, Paint, PatternSource, RenderCmd};
use kivact_types::scene::Shape;
use kivact_types::{BlendMode, FilterOp, PaintProps, Rgba};

/// How the colors of one node are drawn: faded by `alpha` (the opacity of
/// the node and its ancestors that is not a group's).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Tint {
    /// Accumulated non-group opacity multiplier.
    pub(crate) alpha: f64,
}

impl Tint {
    /// A shared color as drawn.
    pub(crate) fn color(&self, color: Rgba) -> Rgba {
        self.arvane(kivact_types::Rgba::sanitize(color))
    }

    /// Apply opacity to an already-lowered neutral color, without
    /// repeating shared component sanitization.
    pub(crate) fn arvane(&self, color: Rgba) -> Rgba {
        Rgba {
            a: color.a * self.alpha,
            ..color
        }
    }

    /// A paint as drawn (every stop faded; a pattern's image is drawn at
    /// full opacity: a tile image has no opacity of its own).
    pub(crate) fn paint(&self, paint: Paint) -> Paint {
        if self.alpha == 1.0 {
            return paint;
        }
        let fade = |g: &mut kivact_types::render::Gradient| {
            for stop in &mut g.stops {
                stop.1 = self.arvane(stop.1);
            }
        };
        match paint {
            Paint::Solid(color) => Paint::Solid(self.arvane(color)),
            Paint::Gradient(mut g) => {
                fade(&mut g);
                Paint::Gradient(g)
            }
            Paint::Pattern(mut p) => {
                if let PatternSource::Gradient(g) = &mut p.source {
                    fade(g);
                }
                Paint::Pattern(p)
            }
        }
    }
}

/// A blur's standard deviation (CSS `blur(px)`), sanitized.
fn blur(px: f64) -> f64 {
    if px.is_finite() { px.max(0.0) } else { 0.0 }
}

/// How far the layer filters of `ops` can draw beyond what they filter.
pub(crate) fn filter_reach(ops: &[FilterOp]) -> f64 {
    ops.iter()
        .map(|op| match *op {
            FilterOp::Blur(px) => blur(px) * kivact_types::render::SHADOW_REACH,
            FilterOp::DropShadow {
                offset, blur: b, ..
            } => {
                let offset = offset.x.abs().max(offset.y.abs());
                (if offset.is_finite() { offset } else { 0.0 })
                    + blur(b) / 2.0 * kivact_types::render::SHADOW_REACH
            }
            _ => 0.0,
        })
        .sum()
}

/// Whether the effects of `p` need a group around the node, whatever its
/// opacity: a clip path or a blend mode.
pub(crate) fn grouped(p: &PaintProps) -> bool {
    p.effects
        .as_ref()
        .is_some_and(|e| e.blend != BlendMode::Normal || e.clip_path.is_some())
}

/// The group a node is composited in: `clip` (the clip path), the group
/// `opacity` and the blend mode of `p`.
pub(crate) fn group(p: &PaintProps, clip: Option<Shape>, opacity: f64) -> Group {
    Group {
        clip,
        opacity,
        blend: p.effects.as_ref().map_or(BlendMode::Normal, |e| e.blend),
        blur: None,
    }
}

/// The command that opens a group clipped to `shape`.
pub(crate) fn clip_group(shape: Shape) -> RenderCmd {
    RenderCmd::PushGroup(Group {
        clip: Some(shape),
        opacity: 1.0,
        blend: BlendMode::Normal,
        blur: None,
    })
}

/// The command that opens a group blurred by `std_dev`.
pub(crate) fn blur_group(std_dev: f64) -> RenderCmd {
    RenderCmd::PushGroup(Group {
        clip: None,
        opacity: 1.0,
        blend: BlendMode::Normal,
        blur: Some(std_dev),
    })
}

/// How far a backdrop blurs what lies around its shape (what it reads
/// beyond the shape).
pub(crate) fn backdrop_reach(p: &PaintProps) -> f64 {
    p.effects.as_ref().map_or(0.0, |e| {
        e.backdrop
            .iter()
            .map(|op| match *op {
                FilterOp::Blur(px) => blur(px) * kivact_types::render::SHADOW_REACH,
                _ => 0.0,
            })
            .sum()
    })
}
