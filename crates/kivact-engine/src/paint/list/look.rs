//! Role: A node's look as the paint list compares it: paint props and stacking.

use super::ink::EVERYWHERE;
use crate::model::Model;
use crate::paint::state::Look;
use crate::tree::node::HostNode;
use kivact_lower::effects;
use kivact_lower::sanitize::{extent, offset, unit};
use kivact_types::{Edges, FilterOp, NodeId, PaintProps, Rect};

impl Look {
    pub(crate) fn of(p: &PaintProps) -> Self {
        let shadows = p
            .shadows
            .iter()
            .filter(|s| !s.inset)
            .map(|s| {
                // The blur's gaussian (standard deviation blur / 2) is
                // drawn at most kivact_types::render::SHADOW_REACH deviations out.
                let blur = extent(s.blur) / 2.0 * kivact_types::render::SHADOW_REACH;
                offset(s.offset.x).abs().max(offset(s.offset.y).abs())
                    + offset(s.spread).max(0.0)
                    + blur
            })
            .fold(0.0, f64::max);
        let ring = p.ring.map_or(0.0, |r| extent(r.width));
        let filter: &[FilterOp] = p.effects.as_ref().map_or(&[], |e| &e.filter);
        let border = kivact_lower::shape::border_widths(p.border.as_ref());
        let items = [
            p.background.is_some(),
            border != Edges::all(0.0),
            !p.shadows.is_empty(),
            p.ring.is_some(),
        ]
        .iter()
        .filter(|x| **x)
        .count() as u8;
        Look {
            opacity: unit(p.opacity, 1.0),
            overflow: shadows.max(ring),
            border: Edges {
                top: extent(border.top),
                right: extent(border.right),
                bottom: extent(border.bottom),
                left: extent(border.left),
            },
            fx: effects::filter_reach(filter).min(EVERYWHERE),
            layers: effects::layer_count(p),
            recolor: effects::color_matrix(filter),
            clip: p.clip,
            items,
            rounded: !p.radius.is_zero(),
        }
    }

    /// What of a paint the paint list is made of, so that a paint with the
    /// same facts lists the same steps and needs no new list: whether and
    /// how it is composited (opacity, which a listed step carries as its
    /// alpha, layers, color matrix), what it draws and where it reaches
    /// (items, rounding, overflow, border, filter reach: they decide the
    /// culling and the clip of its children) and whether it clips. Colors,
    /// radii values, shadow colors and the like are not among them.
    pub(crate) fn stacking(&self) -> impl PartialEq + use<> {
        (
            self.opacity,
            self.overflow,
            self.border,
            self.fx,
            self.layers,
            self.recolor,
            self.items,
            self.rounded,
            self.clip,
        )
    }

    /// Its canvas: `bounds` inside its border.
    pub(super) fn inside(&self, bounds: Rect) -> Rect {
        let b = self.border;
        Rect {
            x: bounds.x + b.left,
            y: bounds.y + b.top,
            width: (bounds.width - b.left - b.right).max(0.0),
            height: (bounds.height - b.top - b.bottom).max(0.0),
        }
    }

    /// The look of `id` now: its base paint's, or with state styles its
    /// paint's with the active deltas.
    pub(super) fn now(host: &Model, id: NodeId, n: &HostNode) -> Self {
        if n.states().is_some() || host.motion.get(id).is_some() {
            Look::of(&host.effective_paint(id))
        } else {
            host.paint.look[n.index.0].unwrap_or_else(|| {
                n.paint()
                    .map_or_else(|| Look::of(&PaintProps::default()), Look::of)
            })
        }
    }
}
