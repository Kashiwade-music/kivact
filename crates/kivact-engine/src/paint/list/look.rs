//! Role: A node's look as the paint list compares it: paint props and stacking.

use crate::paint::state::Look;
use kivact_types::Rect;

impl Look {
    /// What of a paint the paint list is made of, so that a paint with the
    /// same facts lists the same steps and needs no new list: whether and
    /// how it is composited (opacity, which a listed step carries as its
    /// alpha, group), what it draws and where it reaches
    /// (items, rounding, overflow, border, filter reach: they decide the
    /// culling and the clip of its children), whether it clips, and which
    /// layers it needs (filters, masks, backdrop, blend, isolation, 3D
    /// facts, will-change). Colors, radii values, shadow colors and the
    /// like are not among them.
    pub(crate) fn stacking(&self) -> impl PartialEq + use<> {
        (
            self.opacity,
            self.overflow,
            self.border,
            self.fx,
            self.grouped,
            self.items,
            self.rounded,
            self.clip,
            (
                self.filtered,
                self.masked,
                self.backdropped,
                self.blended,
                self.isolated,
                self.will_change,
            ),
            (
                self.preserves,
                self.flat,
                self.transformed,
                self.perspective,
                self.backface_hidden,
                self.shader,
            ),
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
}
