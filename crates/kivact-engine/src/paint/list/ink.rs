//! Role: Ink bounds: how far a node, its text and its scene paint beyond its box.

use crate::contract::NodeType;
use crate::model::Model;
use crate::model::transform::map_rect;
use crate::paint::state::{Ink, Look};
use crate::tree::node::HostNode;
use kivact_layout::NodeLayout;
use kivact_types::cell::Generation;
use kivact_types::{DrawCmd, DrawList, NodeId, Point, Rect};

/// The extent of what reaches everywhere (finite, so unions keep it).
pub(super) const EVERYWHERE: f64 = f64::MAX / 4.0;

/// Everything one node itself can draw on, with its own filter reach
/// (see the module invariants), untransformed, relative to its layout
/// box's top-left corner.
pub(super) fn reach(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    look: &Look,
) -> Rect {
    grow(reach_local(host, id, n, layout, look), look.fx)
}

/// Everything one node itself can draw on, before its own filters (the
/// box, its content, shadows, glyphs and canvas scenes).
fn reach_local(host: &Model, id: NodeId, n: &HostNode, layout: &NodeLayout, look: &Look) -> Rect {
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        width: layout.size.width,
        height: layout.size.height,
    };
    let content = Rect {
        width: layout.content_size.width.max(layout.size.width),
        height: layout.content_size.height.max(layout.size.height),
        ..bounds
    };
    let glyphs = match n.kind() {
        _ if n.kind() == NodeType::Text || host.editors[n.index.0].is_some() => {
            text_reach(host, id)
        }
        _ => 0.0,
    };
    let own = look.overflow.max(glyphs.abs());
    // A non-finite extent reaches everywhere.
    let own = if own.is_finite() {
        own.min(EVERYWHERE)
    } else {
        EVERYWHERE
    };
    let mut rect = grow(bounds.union(content), own);
    let canvas = look.inside(bounds);
    for scene in n.scenes().iter().flatten() {
        if let Some(r) = scene_reach(scene, canvas) {
            rect = rect.union(r);
        }
    }
    rect
}

/// `r` (relative to a box's top-left corner) mapped by the box's transform
/// `t`, placed at the box's origin `at` in its parent's space.
pub(super) fn placed(t: Option<&kivact_types::Transform>, r: Rect, at: Point) -> Rect {
    let r = map_rect(t, r);
    Rect {
        x: r.x + at.x,
        y: r.y + at.y,
        ..r
    }
}

/// The ink of `id` (its node `n`, laid out as `layout`) relative to its
/// layout location (`None`: it draws nothing): its reach and, unless it
/// clips, its children's inks at their scrolled locations, mapped by its
/// transform. Kept in the node (see the module invariants).
pub(super) fn ink(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    generation: Generation,
) -> Option<Rect> {
    if let Some(kept) = host.paint.ink[n.index.0].get()
        && kept.generation == generation
        && kept.size == layout.size
        && kept.content == layout.content_size
    {
        return kept.rect;
    }
    let rect = host.tree().drawn(id).then(|| {
        let look = Look::now(host, id, n);
        // Blurred by the node's own filters, then transformed by it.
        let local = grow(ink_local(host, id, n, layout, generation), look.fx);
        map_rect(local_transform(host, id, n, layout).as_ref(), local)
    });
    host.paint.ink[n.index.0].set(Some(Ink {
        rect,
        size: layout.size,
        content: layout.content_size,
        generation,
    }));
    rect
}

/// The ink of `id` before its own filters and transform: its own reach
/// and, unless it clips, its children's inks at their scrolled locations
/// (each with its own effects and transform).
pub(super) fn ink_local(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    generation: Generation,
) -> Rect {
    let look = Look::now(host, id, n);
    let mut rect = reach_local(host, id, n, layout, &look);
    if !n.clips() {
        let scroll = n.scroll();
        for child in host.tree().flat_children(host.tree().nodes.id_at(n.index)) {
            let Some(c) = host.node(child).filter(|c| !c.is_layer()) else {
                continue;
            };
            let l = host.computed_layout(child).unwrap_or_default();
            if let Some(r) = ink(host, child, c, &l, generation) {
                let mut at = Rect {
                    x: r.x - scroll.x + l.location.x,
                    y: r.y - scroll.y + l.location.y,
                    ..r
                };
                if c.place.sticky {
                    // Wherever its scrollport may stick it.
                    at = sticky_reach(n, layout, &l, at);
                }
                rect = rect.union(at);
            }
        }
    }
    rect
}

/// How far the text `id` paints out of its box (the font size and the
/// shadows it inherits, each from the nearest ancestor-or-self `Text`
/// prop that sets it, like `NodeTree::text_props`, without copying the
/// props).
fn text_reach(host: &Model, id: NodeId) -> f64 {
    host.text_reach(id)
}

/// The ink `at` (in the content space of `parent`, laid out as `layout`)
/// of a sticky child laid out as `child`, grown over every shift the
/// child may take: anywhere its box stays in its containing block (the
/// parent's content box, or the content a scrolling parent scrolls over;
/// `model::interact::sticky`), so culling never depends on a scroll offset.
fn sticky_reach(parent: &HostNode, layout: &NodeLayout, child: &NodeLayout, at: Rect) -> Rect {
    let c = layout.content_box;
    let block = if parent.scroll_axes() != (false, false) {
        Rect {
            width: (layout.content_size.width - (layout.size.width - c.width)).max(c.width),
            height: (layout.content_size.height - (layout.size.height - c.height)).max(c.height),
            ..c
        }
    } else {
        c
    };
    let (x, y) = (child.location.x, child.location.y);
    let (w, h) = (child.size.width, child.size.height);
    let lo_x = (block.x - x).min(0.0);
    let hi_x = (block.x + block.width - x - w).max(0.0);
    let lo_y = (block.y - y).min(0.0);
    let hi_y = (block.y + block.height - y - h).max(0.0);
    Rect {
        x: at.x + lo_x,
        y: at.y + lo_y,
        width: at.width + hi_x - lo_x,
        height: at.height + hi_y - lo_y,
    }
}

/// Everything a canvas scene at `canvas` can draw on (`None`: nothing).
fn scene_reach(scene: &DrawList, canvas: Rect) -> Option<Rect> {
    if !scene
        .cmds
        .iter()
        .any(|cmd| matches!(cmd, DrawCmd::PushTransform(_) | DrawCmd::PushLayer(_)))
    {
        return plain_scene_reach(&scene.cmds, canvas);
    }
    let mut transform = kivact_types::Transform::IDENTITY;
    let mut transforms = Vec::new();
    let mut reach = 0.0;
    let mut layers = Vec::new();
    let mut out = None;
    for cmd in &scene.cmds {
        match cmd {
            DrawCmd::PushTransform(m) => {
                transforms.push(transform);
                transform = m.then(transform);
            }
            DrawCmd::PopTransform => transform = transforms.pop().unwrap_or_default(),
            DrawCmd::PushLayer(layer) => {
                layers.push(reach);
                reach += crate::lower::effects::filter_reach(&layer.filter);
            }
            DrawCmd::PopLayer => reach = layers.pop().unwrap_or(0.0),
            _ => {
                if let Some(r) = plain_scene_reach(std::slice::from_ref(cmd), Rect::default()) {
                    let r = transform.bounds(grow(r, reach));
                    let r = Rect {
                        x: canvas.x + r.x,
                        y: canvas.y + r.y,
                        width: r.width,
                        height: r.height,
                    };
                    out = Some(out.map_or(r, |old: Rect| old.union(r)));
                }
            }
        }
    }
    out
}

fn plain_scene_reach(cmds: &[DrawCmd], canvas: Rect) -> Option<Rect> {
    let mut out: Option<Rect> = None;
    let mut add = |r: Rect| out = Some(out.map_or(r, |o| o.union(r)));
    let point = |p: Point, extent: f64| Rect {
        x: canvas.x + p.x - extent,
        y: canvas.y + p.y - extent,
        width: 2.0 * extent,
        height: 2.0 * extent,
    };
    let at = |r: Rect| Rect {
        x: canvas.x + r.x,
        y: canvas.y + r.y,
        ..r
    };
    for cmd in cmds {
        match cmd {
            DrawCmd::Rect { rect, .. }
            | DrawCmd::Pixels { rect, .. }
            | DrawCmd::ImagePart { rect, .. }
            | DrawCmd::RoundedRect { rect, .. }
            | DrawCmd::Image { rect, .. }
            | DrawCmd::External { rect, .. } => add(at(*rect)),
            DrawCmd::StrokeRect { rect, width, .. } => add(grow(at(*rect), width.abs())),
            DrawCmd::Line {
                from, to, width, ..
            } => {
                add(point(*from, width.abs()));
                add(point(*to, width.abs()));
            }
            DrawCmd::Text(run) => {
                let size = run.font_size.abs();
                let chars = run.text.chars().count() as f64 + 1.0;
                // Letter spacing follows every character.
                let spacing = run.letter_spacing.max(0.0);
                // Word spacing follows every space.
                let spaces = run.text.chars().filter(|c| *c == ' ').count() as f64;
                let word_spacing = run.word_spacing.max(0.0) * spaces;
                add(Rect {
                    x: canvas.x + run.origin.x - size,
                    y: canvas.y + run.origin.y - 2.0 * size,
                    width: size * (2.0 * chars + 1.0) + spacing * chars + word_spacing,
                    height: 4.0 * size,
                });
            }
            DrawCmd::Paragraph { shaped, origin, .. } => {
                // Glyphs overhang their lines by about a line.
                let pad = shaped.size.height;
                add(Rect {
                    x: canvas.x + origin.x - pad,
                    y: canvas.y + origin.y - pad,
                    width: shaped.size.width + 2.0 * pad,
                    height: shaped.size.height + 2.0 * pad,
                });
            }
            DrawCmd::Shape(p) => add(grow(
                at(crate::lower::scene_shape::shape(&p.shape).bounds()),
                p.stroke
                    .as_ref()
                    .map_or(0.0, |s| s.width.abs() * s.miter_limit.max(1.0)),
            )),
            DrawCmd::BorderImage { rect, image } => {
                let o = image.outset;
                add(at(Rect {
                    x: rect.x - o.left,
                    y: rect.y - o.top,
                    width: rect.width + o.left + o.right,
                    height: rect.height + o.top + o.bottom,
                }));
            }
            // A drawable's extent is its subtree's; the paint model does not draw it.
            DrawCmd::Drawable { .. } => {}
            DrawCmd::PushLayer(_) | DrawCmd::PopLayer => {}
            DrawCmd::PushClip(_)
            | DrawCmd::PopClip
            | DrawCmd::PushTransform(_)
            | DrawCmd::PopTransform => {}
        }
    }
    out
}

pub(super) fn grow(r: Rect, by: f64) -> Rect {
    Rect {
        x: r.x - by,
        y: r.y - by,
        width: r.width + 2.0 * by,
        height: r.height + 2.0 * by,
    }
}

/// Whether `a` and `b` share area. A non-finite extent (an unsanitized
/// scene or layout) counts as overlapping, so it is never culled.
pub(super) fn overlaps(a: Rect, b: Rect) -> bool {
    let finite = [a.x, a.y, a.width, a.height].iter().all(|v| v.is_finite());
    !finite
        || (a.x < b.x + b.width
            && b.x < a.x + a.width
            && a.y < b.y + b.height
            && b.y < a.y + a.height)
}

/// The transform of `id` relative to its box's top-left corner, if it has
/// one (read only for the nodes whose paint transforms them).
pub(super) fn local_transform(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
) -> Option<kivact_types::Transform> {
    if n.place.transformed || host.layout_motion_transform(id).is_some() {
        host.local_transform(id, layout.size)
    } else {
        None
    }
}
