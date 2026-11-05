//! Role: Boxes a text is checked against: ancestors, parent, padding and clips.

use crate::model::Model;
use crate::model::interact::visual;
use crate::model::semantics::semantic_role;
use crate::tree::node::HostNode;
use kivact_types::{Edges, Fill, FillImage, LayoutCheck, NodeId, Rect};

/// The ancestors of `id`, nearest first, up to and including the first
/// overlay or fixed box (none for such a box itself).
pub(super) fn ancestors(host: &Model, id: NodeId) -> Vec<NodeId> {
    let tree = host.tree();
    let mut out = Vec::new();
    let Some(n) = tree.get(id) else { return out };
    if n.is_layer() {
        return out;
    }
    let mut at = tree.parent_id(n);
    while let Some(p) = at {
        let Some(pn) = tree.get(p) else { break };
        out.push(p);
        if pn.is_layer() {
            break;
        }
        at = tree.parent_id(pn);
    }
    out
}

/// The nearest of `ancestors` that paints a box of its own or expects one
/// line: a background that draws, a border, a clip or a single-line role.
pub(super) fn visual_parent(
    host: &Model,
    ancestors: &[NodeId],
    check: &LayoutCheck,
) -> Option<NodeId> {
    ancestors.iter().copied().find(|a| {
        host.node(*a).is_some_and(|n| {
            n.paint().is_some_and(|p| {
                p.background.as_ref().is_some_and(paints)
                    || crate::lower::shape::border_widths(p.border.as_ref()) != Edges::all(0.0)
            }) || n.clips()
                || semantic_role(n, host.editors[n.index.0].is_some())
                    .is_some_and(|r| check.single_line_roles.contains(&r))
        })
    })
}

/// The visual parent of the text `id`.
#[cfg(debug_assertions)]
pub(crate) fn visual_parent_of(host: &Model, id: NodeId, check: &LayoutCheck) -> Option<NodeId> {
    visual_parent(host, &ancestors(host, id), check)
}

/// Whether `fill` draws anything.
fn paints(fill: &Fill) -> bool {
    match fill {
        Fill::Solid(c) => c.a > 0.0,
        Fill::Layers(layers) => layers.iter().any(|l| match &l.image {
            FillImage::Solid(c) => c.a > 0.0,
            _ => true,
        }),
    }
}

// xtask-allow(state-fields): The two axes are independent geometric limit facts; every combination is valid.
/// The axes a limit holds on.
#[derive(Clone, Copy)]
pub(super) struct Axes {
    x: bool,
    y: bool,
}

impl Axes {
    /// Every axis but those `n` scrolls (a text input scrolls its text
    /// across).
    pub(super) fn held_by(n: &HostNode, input: bool) -> Axes {
        let (sx, sy) = n.scroll_axes();
        Axes {
            x: !sx && !input,
            y: !sy,
        }
    }
}

/// How far `r` reaches outside `limit` across `axes` (at most 0 inside).
pub(super) fn reach(r: Rect, limit: Rect, axes: Axes) -> f64 {
    let mut most = 0.0f64;
    if axes.x {
        most = most
            .max(limit.x - r.x)
            .max(r.x + r.width - (limit.x + limit.width));
    }
    if axes.y {
        most = most
            .max(limit.y - r.y)
            .max(r.y + r.height - (limit.y + limit.height));
    }
    most
}

/// The padding box of `id` (its border box inside its visible border), in
/// the layout space.
pub(super) fn padding_box(host: &Model, id: NodeId) -> Option<Rect> {
    let bounds = host.rect(id)?;
    let paint = host.node(id)?.paint().cloned().unwrap_or_default();
    Some(visual::padding_box(bounds, &paint))
}

/// A box that cuts what it holds: which node it is, where, and across
/// which axes.
pub(super) struct Clip {
    pub(super) node: NodeId,
    pub(super) rect: Rect,
    pub(super) axes: Axes,
}

/// The clipping ancestors of the text `id` (`up`, nearest first), with the
/// padding box each clips to.
pub(super) fn clips(host: &Model, id: NodeId, up: &[NodeId]) -> Vec<Clip> {
    let input = host.node(id).is_some_and(|n| {
        host.editors[n.index.0]
            .as_deref()
            .map(|editor| editor.editor.state())
            .is_some()
    });
    up.iter()
        .filter_map(|a| {
            let n = host.node(*a).filter(|n| n.clips())?;
            Some(Clip {
                node: *a,
                rect: padding_box(host, *a)?,
                axes: Axes::held_by(n, input),
            })
        })
        .collect()
}
