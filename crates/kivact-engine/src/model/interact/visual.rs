//! Role: Placement of a node's parts inside its untransformed layout box.
//!
//! Where a node's parts are painted inside its untransformed layout box
//! (the model's, `Model::rect`): its paint is drawn there, in its own
//! space, and its transforms (`model::transform`) are pushed around it by
//! the paint list.
//!
//! Invariants:
//! - The paint read here is sanitized (`paint::sanitize_paint`): opacity
//!   is always finite.

use crate::model::Model;
use kivact_types::{NodeId, PaintProps, Rect};

/// The paint `id` is drawn with now (state deltas applied), sanitized.
pub(crate) fn paint(host: &Model, id: NodeId) -> PaintProps {
    let mut p = host.effective_paint(id);
    kivact_lower::sanitize::paint(&mut p);
    p
}

/// Where the text of `id` is drawn in its own space: its content box
/// (inside padding and border, as layout resolved them).
pub(crate) fn text_box(host: &Model, id: NodeId) -> Rect {
    let content = host
        .computed_layout(id)
        .map(|l| l.content_box)
        .unwrap_or_default();
    content_at(host.rect(id).unwrap_or_default(), content)
}

/// A content box (relative to its node) placed at the box `bounds`.
pub(crate) fn content_at(bounds: Rect, content: Rect) -> Rect {
    Rect {
        x: bounds.x + content.x,
        y: bounds.y + content.y,
        ..content
    }
}

/// `bounds` without its visible border: where canvas scenes are placed
/// and the content of a clipping node is clipped (before its inner radii).
pub(crate) fn padding_box(bounds: Rect, paint: &PaintProps) -> Rect {
    let b = kivact_lower::shape::border_widths(paint.border.as_ref());
    Rect {
        x: bounds.x + b.left,
        y: bounds.y + b.top,
        width: (bounds.width - b.left - b.right).max(0.0),
        height: (bounds.height - b.top - b.bottom).max(0.0),
    }
}
