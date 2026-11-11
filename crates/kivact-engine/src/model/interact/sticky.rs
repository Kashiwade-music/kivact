//! Role: Sticky boxes laid out in flow, then shifted by their scrollport's scroll.
//!
//! Sticky boxes (CSS `position: sticky`): laid out in flow, then moved by
//! the scroll of their scrollport to keep their insets from its edges.
//! The move (`HostNode::shift`) is part of the box's position everywhere
//! the model places it ([`Model::location`], so `rect`, hit testing,
//! the paint list, accessibility, overlay anchors, scroll into view and
//! the reports all see the stuck box).
//!
//! Invariants:
//! - A box's scrollport is the border box of its nearest ancestor with a
//!   scrolling axis below its nearest layer root (overlay or fixed box),
//!   else the window. Its shift keeps it `inset` from the scrollport's
//!   edges on the sides whose insets are set (percentages of the
//!   scrollport's size), but never moves its border box out of its
//!   containing block: its parent's content box, or the scrolled content
//!   when its parent is the scroll container itself. Top wins over
//!   bottom and left over right, as in CSS.
//! - Shifts are settled after every layout pass and scroll (no layout
//!   computation), outer boxes before the sticky boxes inside them, from
//!   the offsets as clamped. A shift that changed marks positions moved
//!   (`Changes::moved`) and the box stuck (`Changes::stuck`), which makes
//!   accessibility build and compare its whole tree again.
//! - A sticky box is laid out by taffy as a relative box without insets
//!   (`layout::style`); it is drawn after, and hit before, its in-flow
//!   siblings of the same z-index (`Model::stack_key`).
//! - The work is proportional to the sticky boxes (each walks its
//!   ancestors), none without any.
//!
//! Limits: a box has one scrollport for both axes, as in CSS, and a clip
//! that does not scroll (`overflow: hidden`) is none. The transforms
//! between a box and its scrollport are not looked at.

use kivact_types::{Dimension, NodeId, Point, Rect};

use crate::model::Model;

impl Model {
    /// Settle every sticky box's shift (see the module invariants).
    pub(crate) fn stick(&mut self) {
        if self.tree.sticky.is_empty() {
            return;
        }
        let mut ids: Vec<(usize, NodeId)> = self
            .tree
            .sticky
            .iter()
            .map(|id| (self.tree.chain(Some(*id)).len(), *id))
            .collect();
        ids.sort_unstable();
        for (_, id) in ids {
            let shift = self.sticky_shift(id).unwrap_or_default();
            if let Some(n) = self.tree.get_mut(id)
                && n.shift != shift
            {
                n.shift = shift;
                self.changes.stuck(id);
                self.a11y.moved(id);
            }
        }
    }

    /// The shift of the sticky box `id` now (`None`: none, it is not
    /// shown or not laid out).
    fn sticky_shift(&self, id: NodeId) -> Option<Point> {
        let n = self.tree.get(id)?;
        if !self.tree.drawn(id) {
            return None;
        }
        let inset = n.layout()?.inset;
        let parent = self.tree.flat_parent(n)?;
        let r = self.rect(id)?;
        // Where layout put it, with the scrolls and outer shifts of now.
        let at = Rect {
            x: r.x - n.shift.x,
            y: r.y - n.shift.y,
            ..r
        };
        let (scroller, port) = self.scrollport(parent);
        let block = self.containing_block(parent, scroller == Some(parent))?;
        let len = |d: Dimension, of: f64| match d {
            Dimension::Px(v) => Some(v),
            Dimension::Percent(p) => Some(of * p / 100.0),
            Dimension::Calc { percent, px } => Some(of * percent / 100.0 + px),
            _ => None,
        };
        Some(Point {
            x: shift(
                (at.x, at.width),
                (block.x, block.width),
                (port.x, port.width),
                len(inset.left, port.width),
                len(inset.right, port.width),
            ),
            y: shift(
                (at.y, at.height),
                (block.y, block.height),
                (port.y, port.height),
                len(inset.top, port.height),
                len(inset.bottom, port.height),
            ),
        })
    }

    /// The scroll container whose scroll a sticky box below `from` (its
    /// parent) follows, and its window-space box (the window without one).
    fn scrollport(&self, from: NodeId) -> (Option<NodeId>, Rect) {
        let window = Rect {
            x: 0.0,
            y: 0.0,
            width: self.viewport.width,
            height: self.viewport.height,
        };
        for x in self.tree.chain(Some(from)) {
            let Some(n) = self.tree.get(x) else { break };
            if n.scroll_axes() != (false, false) {
                return (Some(x), self.rect(x).unwrap_or(window));
            }
            if n.is_layer() {
                break;
            }
        }
        (None, window)
    }

    /// The window-space box a sticky child of `parent` stays in: the
    /// parent's content box, or (`scrolls`: the parent is its scroll
    /// container) the content the parent scrolls over.
    fn containing_block(&self, parent: NodeId, scrolls: bool) -> Option<Rect> {
        let r = self.rect(parent)?;
        let l = self.computed_layout(parent)?;
        let c = l.content_box;
        if !scrolls {
            return Some(Rect {
                x: r.x + c.x,
                y: r.y + c.y,
                ..c
            });
        }
        let s = self.tree.get(parent)?.scroll;
        Some(Rect {
            x: r.x + c.x - s.x,
            y: r.y + c.y - s.y,
            width: (l.content_size.width - (l.size.width - c.width)).max(c.width),
            height: (l.content_size.height - (l.size.height - c.height)).max(c.height),
        })
    }
}

/// The shift along one axis of a box at `(start, len)` in a containing
/// block `block` and a scrollport `port`, with the start and end insets
/// `lo` and `hi` (see the module invariants).
fn shift(
    (start, len): (f64, f64),
    block: (f64, f64),
    port: (f64, f64),
    lo: Option<f64>,
    hi: Option<f64>,
) -> f64 {
    let mut d = 0.0;
    if let Some(hi) = hi {
        // Its end stays `hi` before the port's end, its start in the block.
        let over = start + len - (port.0 + port.1 - hi);
        if over > 0.0 {
            d = (-over).max(block.0 - start).min(0.0);
        }
    }
    if let Some(lo) = lo {
        // Its start stays `lo` after the port's start, its end in the block.
        let under = port.0 + lo - start;
        if under > 0.0 {
            d = under.min(block.0 + block.1 - (start + len)).max(0.0);
        }
    }
    if d.is_finite() { d } else { 0.0 }
}
