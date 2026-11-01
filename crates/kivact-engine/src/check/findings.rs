//! Role: Apply the layout rules to a text or a subtree and collect findings.

use super::boxes::{Axes, ancestors, clips, padding_box, reach, visual_parent};
use super::shaped_lines::shape;
use crate::model::Model;
use crate::model::interact::visual;
use crate::model::semantics::semantic_role;
use crate::tree::node::HostNode;
#[cfg(feature = "testing")]
use kivact_wire::NodeType;
use kivact_wire::{
    Align, FlexDirection, Justify, LayoutCheck, LayoutFinding, LayoutLimit, LayoutProps,
    LayoutRule, NodeId, Position, Rect,
};

/// How far (px) the whole-pixel layout can move a box from its ideal
/// place.
const PIXEL_ROUNDING: f32 = 0.5;

/// The findings of the texts in the subtree of `scope` (the whole tree
/// without one), in tree order; `ink`: the vertical extent of a line is
/// that of its glyphs' outlines, else of its metrics.
#[cfg(feature = "testing")]
pub(crate) fn layout_findings(
    host: &Model,
    engine: &mut arvane::Engine,
    check: &LayoutCheck,
    scope: Option<NodeId>,
    ink: bool,
) -> Vec<LayoutFinding> {
    let mut out = Vec::new();
    let Some(start) = scope.or(host.tree().root()) else {
        return out;
    };
    if !host.tree().shown(start) || ignored_above(host, start, check) {
        return out;
    }
    let mut stack = vec![start];
    while let Some(id) = stack.pop() {
        let Some(n) = host.node(id) else { continue };
        if n.hidden() || ignores(n, check) {
            continue;
        }
        if n.kind() == NodeType::Text
            || host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some()
        {
            text_findings(host, engine, check, id, ink, &mut out);
        }
        stack.extend(host.tree().children_ids(n).rev());
    }
    out
}

/// Whether `id` or an ancestor has a test id the check ignores.
#[cfg(feature = "testing")]
fn ignored_above(host: &Model, id: NodeId, check: &LayoutCheck) -> bool {
    !check.ignore_test_ids.is_empty()
        && host
            .tree()
            .chain(Some(id))
            .into_iter()
            .any(|a| host.node(a).is_some_and(|n| ignores(n, check)))
}

#[cfg(feature = "testing")]
fn ignores(n: &HostNode, check: &LayoutCheck) -> bool {
    n.semantics()
        .and_then(|s| s.test_id.as_ref())
        .is_some_and(|t| check.ignore_test_ids.iter().any(|i| i == t))
}

/// The findings of the text or text input `id`, appended to `out`.
pub(crate) fn text_findings(
    host: &Model,
    engine: &mut arvane::Engine,
    check: &LayoutCheck,
    id: NodeId,
    ink: bool,
    out: &mut Vec<LayoutFinding>,
) {
    let Some(n) = host.node(id) else { return };
    let Some(shaped) = shape(host, engine, id, ink) else {
        return;
    };
    if shaped.lines.is_empty() {
        return;
    }
    let input = host.editors[n.index.0]
        .as_deref()
        .map(|editor| editor.editor.state())
        .is_some();
    let tolerance = check.overflow_tolerance;
    let text = shaped.rect();
    let up = ancestors(host, id);
    let vp = visual_parent(host, &up, check);
    let mut push = |rule, amount, text_rect, limit_rect| {
        out.push(LayoutFinding {
            node: id,
            rule,
            amount,
            text_rect,
            limit_rect,
        });
    };

    // Clipped with no ellipsis: by the nearest box that cuts a line.
    let mut clipped_at = None;
    for clip in clips(host, id, &up) {
        let cut: Vec<(f32, Rect)> = shaped
            .lines
            .iter()
            .filter(|l| !l.cut)
            .map(|l| (reach(l.ink, clip.rect, clip.axes), l.ink))
            .filter(|(by, _)| *by > tolerance)
            .collect();
        if let Some(most) = cut.iter().map(|(by, _)| *by).reduce(f32::max) {
            let rect = cut
                .iter()
                .map(|(_, r)| *r)
                .reduce(|a, b| a.union(b))
                .unwrap_or(text);
            push(LayoutRule::ClippedWithoutEllipsis, most, rect, clip.rect);
            clipped_at = Some(clip.node);
            break;
        }
    }

    // Overflow: out of its own box, else out of its visual parent's.
    let own = visual::text_box(host, id);
    let own_by = reach(text, own, Axes::held_by(n, input));
    if own_by > tolerance {
        push(
            LayoutRule::Overflow {
                into: LayoutLimit::OwnBox,
            },
            own_by,
            text,
            own,
        );
    }
    if let Some(v) = vp.filter(|v| clipped_at != Some(*v))
        && let (Some(vn), Some(limit)) = (host.node(v), padding_box(host, v))
    {
        let by = reach(text, limit, Axes::held_by(vn, input));
        if by > tolerance && by > own_by + tolerance {
            push(
                LayoutRule::Overflow {
                    into: LayoutLimit::VisualParent,
                },
                by,
                text,
                limit,
            );
        }
    }

    // Wrapped: more than one line below a role that expects one.
    let lines = shaped.lines.len();
    if lines >= 2 && !n.content().is_some_and(|c| c.contains('\n')) {
        let clamped = host.text_props(id).more().max_lines.is_some_and(|m| m >= 2);
        let owner = std::iter::once(id)
            .chain(up.iter().copied())
            .find(|a| single_line(host, *a, check));
        if let (false, Some(owner)) = (clamped, owner)
            && let Some(limit) = host.rect(owner)
        {
            let extra = shaped.lines[1..].iter().map(|l| l.height).sum();
            push(
                LayoutRule::Wrapped {
                    lines: lines as u32,
                },
                extra,
                text,
                limit,
            );
        }
    }

    // Off center: every box up to the visual parent centers it.
    if let (1, Some(v)) = (lines, vp) {
        let upto = up.iter().position(|a| *a == v).map_or(0, |i| i + 1);
        let line = &shaped.lines[0];
        if line.cap_height > 0.0 && centered_in(host, id, &up[..upto]) {
            let content = visual::text_box(host, v);
            let middle = line.baseline - line.cap_height / 2.0;
            let off = (middle - (content.y + content.height / 2.0)).abs();
            // The layout rounds boxes to whole pixels: half a pixel of
            // slack on top, or an odd leftover would always be off.
            let allowed = check
                .center_tolerance
                .unwrap_or_else(|| (shaped.font_size * 0.08).max(1.0))
                + PIXEL_ROUNDING;
            if off > allowed {
                push(LayoutRule::OffCenter, off, text, content);
            }
        }
    }
}

/// Whether the role of `id` expects its text on one line.
fn single_line(host: &Model, id: NodeId, check: &LayoutCheck) -> bool {
    host.node(id)
        .and_then(|node| semantic_role(node, host.editors[node.index.0].is_some()))
        .is_some_and(|r| check.single_line_roles.contains(&r))
}

/// Whether each of `boxes` (the ancestors of `child`, nearest first)
/// centers its child vertically, so the child sits in the middle of the
/// last.
fn centered_in(host: &Model, child: NodeId, boxes: &[NodeId]) -> bool {
    let mut child = child;
    for b in boxes {
        let Some(parent) = host.node(*b) else {
            return false;
        };
        if !centers(host, parent, child) {
            return false;
        }
        child = *b;
    }
    true
}

/// Whether `parent` centers `child` vertically: a row aligning it in the
/// middle, or a column justifying its only in-flow child there.
fn centers(host: &Model, parent: &HostNode, child: NodeId) -> bool {
    let default = LayoutProps::default();
    let layout = parent.layout().unwrap_or(&default);
    if layout.wrap || layout.grid.as_ref().is_some_and(|g| g.container.is_some()) {
        return false;
    }
    // A box out of the flow sits at its insets, wherever its parent aligns
    // the others.
    let position = host
        .node(child)
        .and_then(|c| c.layout())
        .map(|l| l.position);
    if !matches!(position, None | Some(Position::Relative | Position::Sticky)) {
        return false;
    }
    match layout.direction {
        FlexDirection::Row | FlexDirection::RowReverse => {
            let own = host
                .node(child)
                .and_then(|c| c.layout())
                .and_then(|l| l.align_self);
            own.or(layout.align_items) == Some(Align::Center)
        }
        FlexDirection::Column | FlexDirection::ColumnReverse => {
            layout.justify == Some(Justify::Center)
                && host
                    .tree()
                    .children_ids(parent)
                    .filter(|c| *c != child)
                    .filter_map(|c| host.node(c))
                    .all(|c| {
                        c.hidden()
                            || c.is_layer()
                            || c.layout().is_some_and(|l| l.position == Position::Absolute)
                    })
        }
        // Variants added later fall back to not centered.
        _ => false,
    }
}
