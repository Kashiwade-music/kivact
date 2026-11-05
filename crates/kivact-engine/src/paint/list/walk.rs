//! Role: Rebuild the paint list by walking the tree in paint order within the view.

use super::ink::{grow, ink, local_transform, overlaps, placed, reach};
use crate::contract::NodeType;
use crate::model::Model;
use crate::model::transform::map_rect;
use crate::paint::state::{ChildIndex, Entry, ListState, Look, Step};
use crate::tree::node::HostNode;
use kivact_types::cell::Generation;
use kivact_types::render::ColorMatrix;
use kivact_types::{NodeId, Point, Rect};
use std::collections::{HashMap, HashSet};

/// Logical pixels the clip is grown by before culling: more than a device
/// pixel at every scale from 0.5 up (and `1 / scale` below).
const MARGIN: f64 = 2.0;

/// Containers with at least this many children keep a [`ChildIndex`].
const INDEXED: usize = 32;

/// Whether the listed `step` of node `n` needs a `Close` after its
/// descendants: it pops a clip, a layer or a transform it pushed, or draws
/// its `DrawOver` scene.
fn closes(step: &Step, n: &HostNode) -> bool {
    step.clips
        || step.presence_clip.is_some()
        || step.layers > 0
        || step.transform.is_some()
        || (step.own && n.scenes()[1].is_some())
}

struct Walk<'a> {
    host: &'a Model,
    margin: f64,
    indices: &'a mut HashMap<NodeId, ChildIndex>,
    /// The layout pass the tree is laid out by.
    pass: Generation,
    /// The generation of the nodes' inks.
    generation: Generation,
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
    fn node(
        &mut self,
        id: NodeId,
        origin: Point,
        parent_alpha: f64,
        matrix: Option<u32>,
        clip: Rect,
    ) {
        let host = self.host;
        let Some(n) = host.node(id) else {
            return;
        };
        if !host.tree().drawn(id) {
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
        let Some((transform, clip, o)) = self.place_transform(local, clip, origin) else {
            return;
        };
        let clips = n.clips();
        let presence_clip = presence_clip(host, id, clip, o, layout.size);
        let (group, layers, alpha) = opacity(host, id, n, &look, parent_alpha);
        let own_matrix = match look.recolor {
            Some(own) => Some(self.matrix(own, matrix)),
            None => matrix,
        };
        let step = Step {
            id,
            origin: o,
            transform,
            alpha,
            clips,
            presence_clip,
            rounded: look.rounded,
            own: visible,
            layers,
            group,
            matrix: own_matrix,
            parent_matrix: matrix,
            clip,
        };
        let listed = visible || layers > 0 || transform.is_some();
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
        let child_clip = presence_clip.map_or(child_clip, |p| child_clip.intersect(&p));
        let scroll = n.scroll();
        // The children's layout locations are relative to this origin.
        let base = Point {
            x: o.x - scroll.x,
            y: o.y - scroll.y,
        };
        self.children(id, n, base, alpha, own_matrix, child_clip);
        if listed && closes(&step, n) {
            self.out.push(Entry::Close(step));
        }
    }

    fn place_transform(
        &mut self,
        local: Option<kivact_types::Transform>,
        clip: Rect,
        origin: Point,
    ) -> Option<(Option<u32>, Rect, Point)> {
        // A translation moves its origin; another transform is pushed, and
        // its subtree is culled in its own space.
        Some(match local {
            Some(t) if t.a == 1.0 && t.b == 0.0 && t.c == 0.0 && t.d == 1.0 => {
                let o = Point {
                    x: origin.x + t.e,
                    y: origin.y + t.f,
                };
                (None, clip, o)
            }
            Some(t) => {
                let t = crate::model::transform::placed(t, origin);
                let Some(back) = t.invert() else {
                    // Flattened: nothing of it shows.
                    return None;
                };
                self.transforms.push(t);
                let index = (self.transforms.len() - 1) as u32;
                (Some(index), map_rect(Some(&back), clip), origin)
            }
            None => (None, clip, origin),
        })
    }

    /// The index of the color matrix of a node with the color filters
    /// `own`: its color filters, then its ancestors' (`parent`).
    fn matrix(&mut self, own: ColorMatrix, parent: Option<u32>) -> u32 {
        let m = match parent.and_then(|p| self.matrices.get(p as usize)) {
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
        alpha: f64,
        matrix: Option<u32>,
        clip: Rect,
    ) {
        let host = self.host;
        let place = |at: Point| Point {
            x: base.x + at.x,
            y: base.y + at.y,
        };
        if host.tree().flat_children(id).len() < INDEXED {
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
            ChildIndex::build(
                host,
                host.tree().flat_children(host.tree().nodes.id_at(n.index)),
                pass,
                generation,
            )
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
        if host.sorts_children(n) {
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
        let key = ListState::Built {
            size: window,
            scale: self.paint.scale,
        };
        if list.state != key {
            let margin = MARGIN.max(1.0 / self.paint.scale.max(f64::MIN_POSITIVE));
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
                walk.node(root, Point::default(), 1.0, None, viewport);
            }
            for (layer, origin) in self.layers() {
                walk.node(layer, origin, 1.0, None, viewport);
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
            list.state = key;
        }
        self.paint.list = list;
    }
}

impl Model {
    /// Build an independent list in the target's local output space.
    pub(crate) fn subtree_paint_list(
        &self,
        id: NodeId,
        clip: Rect,
    ) -> crate::paint::state::PaintList {
        let mut indices = HashMap::new();
        let mut walk = Walk {
            host: self,
            margin: MARGIN,
            indices: &mut indices,
            pass: self.paint.list.pass,
            generation: self.paint.list.ink_gen,
            used: HashSet::new(),
            out: Vec::new(),
            matrices: Vec::new(),
            transforms: Vec::new(),
        };
        walk.node(id, Point::default(), 1.0, None, clip);
        let bounds = self.rect(id).unwrap_or_default();
        for (layer, origin) in self.layers() {
            if layer != id && self.tree().is_in_subtree(layer, id) {
                walk.node(
                    layer,
                    Point {
                        x: origin.x - bounds.x,
                        y: origin.y - bounds.y,
                    },
                    1.0,
                    None,
                    clip,
                );
            }
        }
        crate::paint::state::PaintList {
            entries: walk.out,
            matrices: walk.matrices,
            transforms: walk.transforms,
            ..Default::default()
        }
    }
}

fn presence_clip(
    host: &Model,
    id: NodeId,
    clip: Rect,
    o: Point,
    size: kivact_types::Size,
) -> Option<Rect> {
    host.presence.sizes.get(&id).map(|m| {
        let mut rect = clip;
        match m.axis {
            kivact_types::Orientation::Horizontal => {
                rect.x = o.x;
                rect.width = size.width;
            }
            kivact_types::Orientation::Vertical => {
                rect.y = o.y;
                rect.height = size.height;
            }
            kivact_types::Orientation::Unspecified => {}
        }
        rect
    })
}
fn opacity(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    look: &Look,
    parent_alpha: f64,
) -> (bool, u8, f64) {
    // Group opacity over more than one thing it draws.
    let content = match n.kind() {
        _ if host.editors[n.index.0].is_some() => 2,
        NodeType::Text | NodeType::Image => 1,
        _ => 0,
    };
    let more = host.tree().flat_children(id).len() != 0
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
    (group, layers, alpha)
}
