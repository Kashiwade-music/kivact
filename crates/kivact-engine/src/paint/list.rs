//! Role: The retained, culled paint list of the nodes a frame draws in paint order.
//!
//! The retained paint list: the nodes a frame draws, in paint order (the
//! tree's, children by z-index), with the origin of their untransformed
//! layout box, the transform they are drawn through and the opacity they
//! are drawn at, culled against the window and their clipping ancestors.
//!
//! Invariants:
//! - The list is rebuilt only when something may move, show, hide,
//!   restack or reclip nodes (typed setter invalidation for writes, `Model::sync`
//!   for layout passes, scrolls and state styles), so a frame after a
//!   content change (text, values, colors, images) draws the same nodes
//!   again with their current content: O(visible nodes), not O(nodes). A
//!   write of a paint, a text style or state styles asks for a rebuild
//!   only when what the steps are made of changed: the node's `Place`
//!   (transform, z-index, sticky, fixed), its paint's [`Look::stacking`]
//!   facts (opacity, layers, color matrix, items, rounding, overflow,
//!   border, filter reach, clip) or, for a text, how far it paints out of
//!   its box.
//! - A node is left out when everything it can draw (its reach) lies
//!   outside the clip its ancestors impose, grown by [`MARGIN`] (so a
//!   pixel the clip lets through partially is never lost): its box, the
//!   extent of its content (taffy's, text overflowing a short box
//!   included), its shadows (an outer one reaches its offset plus its
//!   spread plus `kivact_types::SHADOW_REACH` × blur / 2) and focus ring, a
//!   text's glyph overhang (one font size) and text shadows, and its
//!   canvas scenes' commands (a scene text run counted at two font sizes
//!   and its letter spacing per character, a stroke at its width × its
//!   miter limit, each mapped by the scene's own transforms and grown by
//!   its layers' filters), all mapped by its transform.
//! - Its subtree is left out with it only when the subtree's ink (its
//!   reach and, unless it clips, its children's inks at their locations,
//!   all mapped by its transform: an absolutely placed badge above it, a
//!   child's wide shadow, a child transformed out of it) lies outside the
//!   clip too. Culling never changes a pixel of the frame.
//! - A transformed node is drawn through its transform (pushed around its
//!   whole drawing, its subtree included), and its subtree is culled
//!   against the clip mapped back into its own space (the bounds of the
//!   inverse); a transform that flattens it lists nothing. A translation
//!   alone moves the origin of the node and its subtree instead (no push:
//!   the same pixels, less work for the many moving boxes of an
//!   animation), so the origins are in the own space of the nearest pushed
//!   transform moved by the translations below it.
//! - The first build computes the ink of every subtree it reaches (so
//!   culling never drops a descendant drawn outside its ancestor), and a
//!   node keeps its ink ([`Ink`]) until something in its subtree changes
//!   (a write, a repaint, a scroll, a new measure: `PaintList::invalidate`
//!   and `PaintList::forget_ink` forget it and its ancestors'), its own
//!   local rounded geometry changes (ancestor placement can change a child's
//!   pixel rounding even when the ancestor's size stays the same), or the
//!   generation moves on (new fonts).
//!   Position-only changes keep the node's local ink and invalidate its
//!   parent's union; the layout pass refreshes child-index locations.
//! - Children follow the tree's order, sorted (stably) by
//!   `Model::stack_key` (the z-index, then sticky after its siblings of the
//!   same z-index) only in a container whose `Place::stacks` is set (a
//!   child with a z-index, or a state style that can give it one, set it
//!   once for good); any other container costs nothing for it.
//! - The layers (`Model::layers`: the fixed boxes outside every overlay,
//!   then each overlay in attach order followed by the fixed boxes inside
//!   it) each start afresh after the root's tree at their window origin,
//!   with the window as their clip, full opacity and no color matrix: they
//!   take none of their ancestors' scrolls, clips, transforms, opacity or
//!   color filters, and are not in their parent's ink or child index.
//! - A node is composited in layers of its own only when it needs them: a
//!   layer filter, a clip path, a mask or a blend mode, or an opacity
//!   below 1 over more than one thing it draws (children, a scene, or two
//!   of its background, border, shadows, ring and content), which is CSS's
//!   group opacity; a lone background or text is faded by its colors'
//!   alpha instead. Such a node is listed (to open and close its layers)
//!   even when only its descendants show. Its subtree is drawn inside
//!   them at full alpha.
//! - A layer filter (blur, drop shadow) grows the node's reach and ink by
//!   how far it draws, and the clip its children are culled against, so a
//!   blurred pixel from outside the window still reaches it; a backdrop
//!   filter grows the whole viewport by its blur's reach (the largest
//!   seen), as it reads what lies around its box.
//! - Color filters compose down the tree into one matrix per node
//!   ([`PaintList::matrices`]); a node draws its colors through its own.
//! - A rebuild is proportional to the nodes it lists, not to the tree: a
//!   container with many children keeps a child index ([`ChildIndex`]):
//!   each child's ink (in the container's unscrolled content space) sorted
//!   along the axis the children follow, so a rebuild finds the children
//!   that reach the clip by binary search. After a layout pass an index
//!   takes the children's new locations, looking again only at the
//!   children whose size changed; a change in a child's subtree makes it
//!   look at that child again; a change of the container's children drops
//!   it (`PaintList::invalidate`). Scrolling keeps it.

use super::state::{ChildIndex, Entry, Ink, Look, PaintList, Step};
use std::collections::{HashMap, HashSet};

use crate::model::Model;
use crate::model::changes::Changes;
use crate::model::transform::map_rect;
use crate::tree::node::HostNode;
use kivact_layout::NodeLayout;
use kivact_lower::effects;
use kivact_lower::sanitize::{extent, offset, unit};
use kivact_types::ColorMatrix;
use kivact_wire::{Edges, FilterOp, NodeId, NodeType, PaintProps, Point, Rect, Scene, SceneCmd};

mod index;

/// Logical pixels the clip is grown by before culling: more than a device
/// pixel at every scale from 0.5 up (and `1 / scale` below).
const MARGIN: f32 = 2.0;
/// Containers with at least this many children keep a [`ChildIndex`].
const INDEXED: usize = 32;
/// The extent of what reaches everywhere (finite, so unions keep it).
const EVERYWHERE: f32 = f32::MAX / 4.0;

/// No color matrix (an index into [`PaintList::matrices`]).
pub(crate) const NO_MATRIX: u32 = u32::MAX;
/// No transform (an index into [`PaintList::transforms`]).
pub(crate) const NO_TRANSFORM: u32 = u32::MAX;

impl PaintList {
    pub(crate) fn new() -> Self {
        PaintList {
            stale: true,
            ..PaintList::default()
        }
    }

    /// Note what `changes` outdated in the inks and the child indices:
    /// after a layout pass every location (and the sizes that changed); a
    /// node whose children changed (its index is dropped); a
    /// node whose drawing changed (its paint, scenes, states, visibility or
    /// inherited text size) or that scrolled forgets its ink and its
    /// ancestors' ([`PaintList::forget_ink`]).
    fn invalidate(
        &mut self,
        changes: &Changes,
        tree: &crate::tree::NodeTree,
        ink: &[std::cell::Cell<Option<Ink>>],
    ) {
        if changes.was_laid_out() {
            self.pass += 1;
        }
        for id in changes.regrouped() {
            self.indices.remove(&id);
        }
        for id in changes.redrawn() {
            self.forget_ink(id, tree, ink);
        }
    }

    /// Something in the subtree of `id` changed what it draws or where:
    /// it and its ancestors forget their inks, and the indices holding
    /// them look at them again. Up to the first node without an ink (of
    /// this generation): no ink above it, and no index entry, was found
    /// from it since it forgot its own (finding an ink finds the inks it
    /// is made of, and an index entry is an ink), so they forgot theirs
    /// with it.
    pub(crate) fn forget_ink(
        &mut self,
        id: NodeId,
        tree: &crate::tree::NodeTree,
        ink: &[std::cell::Cell<Option<Ink>>],
    ) {
        let mut at = Some(id);
        while let Some(x) = at {
            let Some(n) = tree.get(x) else {
                break;
            };
            if ink[n.index.0]
                .take()
                .is_none_or(|kept| kept.generation != self.ink_gen)
            {
                break;
            }
            at = tree.parent_id(n);
            if let Some(index) = at.and_then(|parent| self.indices.get_mut(&parent)) {
                index.touch(x);
            }
        }
    }

    /// A node has a backdrop filter reading `reach` around its box: the
    /// viewport grows to it (it never shrinks back, which only lists a few
    /// more nodes).
    pub(crate) fn note_backdrop(&mut self, reach: f32) {
        if reach.is_finite() && reach > self.backdrop_reach {
            self.backdrop_reach = reach;
            self.stale = true;
        }
    }

    /// Every node's measure may have changed (fonts were added): forget
    /// every ink and index.
    pub(crate) fn forget_inks(&mut self) {
        self.ink_gen += 1;
        self.indices.clear();
        self.stale = true;
    }
}

impl Model {
    /// The facts of `id`'s target paint (its base paint with its active
    /// state deltas) that its paint list steps are made of ([`Look::stacking`]).
    pub(crate) fn paint_stacking(&self, id: NodeId) -> impl PartialEq + use<> {
        Look::of(&self.target_paint(id)).stacking()
    }

    /// Bring the paint list up to date for the next frame: rebuilt if
    /// anything restacked since the last one (see the module invariants).
    pub(crate) fn paint_list(&mut self) {
        let mut list = std::mem::take(&mut self.paint.list);
        let window = self.window();
        let key = Some((window, self.paint.scale));
        if list.stale || list.built_for != key {
            let margin = MARGIN.max(1.0 / self.paint.scale.max(f32::MIN_POSITIVE));
            let reach = margin + list.backdrop_reach;
            let viewport = Rect {
                x: -reach,
                y: -reach,
                width: window.width + 2.0 * reach,
                height: window.height + 2.0 * reach,
            };
            let mut walk = Walk {
                host: self,
                margin,
                indices: &mut list.indices,
                pass: list.pass,
                generation: list.ink_gen,
                used: HashSet::new(),
                out: Vec::with_capacity(list.entries.len()),
                matrices: Vec::new(),
                transforms: Vec::new(),
            };
            if let Some(root) = self.tree().root() {
                walk.node(root, Point::default(), 1.0, NO_MATRIX, viewport);
            }
            for (layer, origin) in self.layers() {
                walk.node(layer, origin, 1.0, NO_MATRIX, viewport);
            }
            let Walk {
                out,
                used,
                matrices,
                transforms,
                ..
            } = walk;
            list.matrices = matrices;
            list.transforms = transforms;
            // Indices of containers this walk did not reach (removed, or
            // culled with an ancestor) are built again when reached.
            list.indices.retain(|id, _| used.contains(id));
            list.entries = out;
            list.stale = false;
            list.built_for = key;
        }
        self.paint.list = list;
    }
}

impl Look {
    pub(crate) fn of(p: &PaintProps) -> Self {
        let shadows = p
            .shadows
            .iter()
            .filter(|s| !s.inset)
            .map(|s| {
                // The blur's gaussian (standard deviation blur / 2) is
                // drawn at most kivact_types::SHADOW_REACH deviations out.
                let blur = extent(s.blur) / 2.0 * kivact_types::SHADOW_REACH;
                offset(s.offset.x).abs().max(offset(s.offset.y).abs())
                    + offset(s.spread).max(0.0)
                    + blur
            })
            .fold(0.0, f32::max);
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
    fn inside(&self, bounds: Rect) -> Rect {
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
    fn now(host: &Model, id: NodeId, n: &HostNode) -> Self {
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

/// Everything one node itself can draw on (see the module invariants),
/// untransformed, relative to its layout box's top-left corner.
fn reach(host: &Model, id: NodeId, n: &HostNode, layout: &NodeLayout, look: &Look) -> Rect {
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
        _ if n.kind() == NodeType::Text
            || host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some() =>
        {
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
    grow(rect, look.fx)
}

/// `r` (relative to a box's top-left corner) mapped by the box's transform
/// `t`, placed at the box's origin `at` in its parent's space.
fn placed(t: Option<&kivact_types::Transform>, r: Rect, at: Point) -> Rect {
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
fn ink(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
    generation: u64,
) -> Option<Rect> {
    if let Some(kept) = host.paint.ink[n.index.0].get()
        && kept.generation == generation
        && kept.size == layout.size
        && kept.content == layout.content_size
    {
        return kept.rect;
    }
    let rect = (!n.hidden()).then(|| {
        let look = Look::now(host, id, n);
        let mut rect = reach(host, id, n, layout, &look);
        if !n.clips() {
            let scroll = n.scroll();
            for child in host.tree().children_ids(n) {
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
                    // Blurred with the node.
                    rect = rect.union(grow(at, look.fx));
                }
            }
        }
        map_rect(local_transform(host, id, n, layout).as_ref(), rect)
    });
    host.paint.ink[n.index.0].set(Some(Ink {
        rect,
        size: layout.size,
        content: layout.content_size,
        generation,
    }));
    rect
}

/// The transform of `id` relative to its box's top-left corner, if it has
/// one (read only for the nodes whose paint transforms them).
fn local_transform(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    layout: &NodeLayout,
) -> Option<kivact_types::Transform> {
    if n.place.transformed {
        host.local_transform(id, layout.size)
    } else {
        None
    }
}

/// How far the text `id` paints out of its box (the font size and the
/// shadows it inherits, each from the nearest ancestor-or-self `Text`
/// prop that sets it, like `NodeTree::text_props`, without copying the
/// props).
fn text_reach(host: &Model, id: NodeId) -> f32 {
    host.text_reach(id)
}

struct Walk<'a> {
    host: &'a Model,
    margin: f32,
    indices: &'a mut HashMap<NodeId, ChildIndex>,
    /// The layout pass the tree is laid out by.
    pass: u64,
    /// The generation of the nodes' inks.
    generation: u64,
    /// The containers whose index this walk used.
    used: HashSet<NodeId>,
    out: Vec<Entry>,
    /// The listed nodes' color matrices.
    matrices: Vec<ColorMatrix>,
    /// The listed transformed nodes' transforms.
    transforms: Vec<kivact_types::Transform>,
}

impl Walk<'_> {
    /// List `id` and its subtree; `origin` is its untransformed layout
    /// box's origin and `clip` the area its ancestors let it draw in
    /// (grown by the margin), both in its parent's own space.
    fn node(&mut self, id: NodeId, origin: Point, parent_alpha: f32, matrix: u32, clip: Rect) {
        let host = self.host;
        let Some(n) = host.node(id) else {
            return;
        };
        if n.hidden() {
            return;
        }
        let layout = host.layout_at(n.index).unwrap_or_default();
        let look = Look::now(host, id, n);
        let local = local_transform(host, id, n, &layout);
        let own = reach(host, id, n, &layout, &look);
        let visible = overlaps(placed(local.as_ref(), own, origin), clip);
        if !visible {
            // Neither it nor anything in its subtree shows.
            let shows = ink(host, id, n, &layout, self.generation).is_some_and(|k| {
                let at = Rect {
                    x: k.x + origin.x,
                    y: k.y + origin.y,
                    ..k
                };
                overlaps(at, clip)
            });
            if !shows {
                return;
            }
        }
        // A translation moves its origin; another transform is pushed, and
        // its subtree is culled in its own space.
        let (transform, clip, o) = match local {
            Some(t) if t.a == 1.0 && t.b == 0.0 && t.c == 0.0 && t.d == 1.0 => {
                let o = Point {
                    x: origin.x + t.e,
                    y: origin.y + t.f,
                };
                (NO_TRANSFORM, clip, o)
            }
            Some(t) => {
                let t = crate::model::transform::placed(t, origin);
                let Some(back) = t.invert() else {
                    // Flattened: nothing of it shows.
                    return;
                };
                self.transforms.push(t);
                let index = (self.transforms.len() - 1) as u32;
                (index, map_rect(Some(&back), clip), origin)
            }
            None => (NO_TRANSFORM, clip, origin),
        };
        let clips = n.clips();
        // Group opacity over more than one thing it draws.
        let content = match n.kind() {
            _ if host.editors[n.index.0]
                .as_deref()
                .map(|editor| editor.editor.state())
                .is_some() =>
            {
                2
            }
            NodeType::Text | NodeType::Image => 1,
            _ => 0,
        };
        let more = !host.tree().nodes.links(n.index).children.is_empty()
            || n.scenes().iter().any(Option::is_some)
            || look.items + content > 1
            || look.layers > 0;
        let group = look.opacity < 1.0 && more;
        let layers = if group {
            look.layers.max(1)
        } else {
            look.layers
        };
        let alpha = if group {
            parent_alpha
        } else {
            parent_alpha * look.opacity
        };
        let own_matrix = match look.recolor {
            Some(own) => self.matrix(own, matrix),
            None => matrix,
        };
        let step = Step {
            id,
            origin: o,
            transform,
            alpha,
            clips,
            rounded: look.rounded,
            own: visible,
            layers,
            group,
            matrix: own_matrix,
            parent_matrix: matrix,
            clip,
        };
        let listed = visible || layers > 0 || transform != NO_TRANSFORM;
        if listed {
            self.out.push(Entry::Open(step));
        }
        let clip = grow(clip, look.fx);
        let child_clip = if clips {
            let padding = look.inside(Rect {
                x: o.x,
                y: o.y,
                width: layout.size.width,
                height: layout.size.height,
            });
            clip.intersect(&grow(padding, self.margin))
        } else {
            clip
        };
        let scroll = n.scroll();
        // The children's layout locations are relative to this origin.
        let base = Point {
            x: o.x - scroll.x,
            y: o.y - scroll.y,
        };
        self.children(id, n, base, alpha, own_matrix, child_clip);
        if listed
            && (clips
                || layers > 0
                || transform != NO_TRANSFORM
                || (visible && n.scenes()[1].is_some()))
        {
            self.out.push(Entry::Close(step));
        }
    }

    /// The index of the color matrix of a node with the color filters
    /// `own`: its color filters, then its ancestors' (`parent`).
    fn matrix(&mut self, own: ColorMatrix, parent: u32) -> u32 {
        let m = match self.matrices.get(parent as usize) {
            Some(p) => own.then(*p),
            None => own,
        };
        self.matrices.push(m);
        (self.matrices.len() - 1) as u32
    }

    /// The children of `id` (its node `n`), whose layout locations are
    /// relative to `base`: through its child index when it has many.
    fn children(
        &mut self,
        id: NodeId,
        n: &HostNode,
        base: Point,
        alpha: f32,
        matrix: u32,
        clip: Rect,
    ) {
        let host = self.host;
        let place = |at: Point| Point {
            x: base.x + at.x,
            y: base.y + at.y,
        };
        if host.tree().nodes.links(n.index).children.len() < INDEXED {
            for child in host.stacked(n) {
                let Some(c) = host.node(child).filter(|c| !c.is_layer()) else {
                    continue;
                };
                let at = host.location(c);
                self.node(child, place(at), alpha, matrix, clip);
            }
            return;
        }
        self.used.insert(id);
        let (pass, generation) = (self.pass, self.generation);
        let index = self.indices.entry(id).or_insert_with(|| {
            ChildIndex::build(host, host.tree().children_ids(n), pass, generation)
        });
        index.refresh(host, pass, generation);
        // The clip in the index's space (the children's layout space).
        let window = Rect {
            x: clip.x - base.x,
            y: clip.y - base.y,
            ..clip
        };
        let mut reached: Vec<(usize, NodeId, Point)> = index
            .reaching(window)
            .map(|e| (e.order, e.id, e.location))
            .collect();
        // Sticky children may be stuck anywhere: the walk culls them.
        let before = reached.len();
        reached.extend(index.sticky().map(|e| (e.order, e.id, e.location)));
        if reached.len() > before {
            reached.sort_by_key(|(order, _, _)| *order);
        }
        if n.place.stacks {
            reached.sort_by_key(|(_, c, _)| host.stack_key(*c));
        }
        for (_, child, at) in reached {
            let at = match host.node(child).filter(|c| c.place.sticky) {
                Some(c) => Point {
                    x: at.x + c.shift.x,
                    y: at.y + c.shift.y,
                },
                None => at,
            };
            self.node(child, place(at), alpha, matrix, clip);
        }
    }
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
fn scene_reach(scene: &Scene, canvas: Rect) -> Option<Rect> {
    if !scene
        .cmds
        .iter()
        .any(|cmd| matches!(cmd, SceneCmd::PushTransform(_) | SceneCmd::PushLayer(_)))
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
            SceneCmd::PushTransform(m) => {
                transforms.push(transform);
                transform = kivact_lower::path::transform(*m).then(transform);
            }
            SceneCmd::PopTransform => transform = transforms.pop().unwrap_or_default(),
            SceneCmd::PushLayer(layer) => {
                layers.push(reach);
                reach += kivact_lower::effects::filter_reach(&layer.filter);
            }
            SceneCmd::PopLayer => reach = layers.pop().unwrap_or(0.0),
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
fn plain_scene_reach(cmds: &[SceneCmd], canvas: Rect) -> Option<Rect> {
    let mut out: Option<Rect> = None;
    let mut add = |r: Rect| out = Some(out.map_or(r, |o| o.union(r)));
    let point = |p: Point, extent: f32| Rect {
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
            SceneCmd::Rect { rect, .. }
            | SceneCmd::Pixels { rect, .. }
            | SceneCmd::ImagePart { rect, .. }
            | SceneCmd::RoundedRect { rect, .. }
            | SceneCmd::Image { rect, .. }
            | SceneCmd::External { rect, .. } => add(at(*rect)),
            SceneCmd::StrokeRect { rect, width, .. } => add(grow(at(*rect), width.abs())),
            SceneCmd::Line {
                from, to, width, ..
            } => {
                add(point(*from, width.abs()));
                add(point(*to, width.abs()));
            }
            SceneCmd::Text(run) => {
                let size = run.font_size.abs();
                let chars = run.text.chars().count() as f32 + 1.0;
                // Letter spacing follows every character.
                let spacing = run.letter_spacing.max(0.0);
                // Word spacing follows every space.
                let spaces = run.text.chars().filter(|c| *c == ' ').count() as f32;
                let word_spacing = run.word_spacing.max(0.0) * spaces;
                add(Rect {
                    x: canvas.x + run.origin.x - size,
                    y: canvas.y + run.origin.y - 2.0 * size,
                    width: size * (2.0 * chars + 1.0) + spacing * chars + word_spacing,
                    height: 4.0 * size,
                });
            }
            SceneCmd::Paragraph { shaped, origin, .. } => {
                // Glyphs overhang their lines by about a line.
                let pad = shaped.size.height;
                add(Rect {
                    x: canvas.x + origin.x - pad,
                    y: canvas.y + origin.y - pad,
                    width: shaped.size.width + 2.0 * pad,
                    height: shaped.size.height + 2.0 * pad,
                });
            }
            SceneCmd::Shape(p) => add(grow(
                at(kivact_lower::scene_shape::shape(&p.shape).bounds()),
                p.stroke
                    .as_ref()
                    .map_or(0.0, |s| s.width.abs() * s.miter_limit.max(1.0)),
            )),
            SceneCmd::PushLayer(_) | SceneCmd::PopLayer => {}
            SceneCmd::PushClip(_)
            | SceneCmd::PopClip
            | SceneCmd::PushTransform(_)
            | SceneCmd::PopTransform => {}
        }
    }
    out
}

fn grow(r: Rect, by: f32) -> Rect {
    Rect {
        x: r.x - by,
        y: r.y - by,
        width: r.width + 2.0 * by,
        height: r.height + 2.0 * by,
    }
}

/// Whether `a` and `b` share area. A non-finite extent (an unsanitized
/// scene or layout) counts as overlapping, so it is never culled.
fn overlaps(a: Rect, b: Rect) -> bool {
    let finite = [a.x, a.y, a.width, a.height].iter().all(|v| v.is_finite());
    !finite
        || (a.x < b.x + b.width
            && b.x < a.x + a.width
            && a.y < b.y + b.height
            && b.y < a.y + a.height)
}

#[cfg(all(debug_assertions, feature = "testing"))]
mod self_check;

impl super::state::PaintState {
    /// Invalidate the retained list for one sync: redraw the frame, forget
    /// font-dependent ink, and rebuild the list when stacking changed.
    pub(crate) fn outdate_paint(
        &mut self,
        tree: &crate::tree::NodeTree,
        changes: &Changes,
        redraw: bool,
        fonts: bool,
        restack: bool,
    ) {
        if redraw {
            self.needs_redraw = true;
        }
        let mut list = std::mem::take(&mut self.list);
        if fonts {
            list.forget_inks();
        }
        list.stale |= restack;
        list.invalidate(changes, tree, &self.ink);
        self.list = list;
    }
}
