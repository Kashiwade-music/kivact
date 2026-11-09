//! Role: Rebuild the paint list by walking the tree in paint order within the view.
//!
//! A layer node (see `layers.rs`) is walked in its own space: its box at the
//! origin, its children's origins and clips relative to it, its transform
//! kept as a [`LayerSpec`]. Any other node is walked in the space of the
//! transforms pushed around it, as before.

use super::ink::{grow, ink, ink_local, local_transform, overlaps, placed, reach};
use crate::contract::NodeType;
use crate::layer::LayerView;
use crate::model::Model;
use crate::model::transform::map_rect;
use crate::paint::state::{
    ChildIndex, Entry, LayerSpec, ListState, Look, PaintList, StaleFacts, Step, SubtreeFacts,
};
use crate::tree::node::HostNode;
use kivact_types::cell::Generation;
use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::BackfaceVisibility;
use kivact_types::render::{Plane, RasterGrid};
use kivact_types::{NodeId, Point, Rect, Transform, WillChange};
use std::collections::{BTreeSet, BinaryHeap, HashMap, HashSet};

/// Logical pixels the clip is grown by before culling: more than a device
/// pixel at every scale from 0.5 up (and `1 / scale` below).
const MARGIN: f64 = 2.0;

/// Containers with at least this many children keep a [`ChildIndex`].
const INDEXED: usize = 32;

/// Whether the content `cull` (in layer space) of a layer mapped by `matrix`
/// shows in `clip`, decided from its projected corners. A corner that is not
/// finite or lies behind the eye leaves the question open: it shows.
fn projected_overlaps(matrix: &Matrix4, cull: Rect, clip: Rect) -> bool {
    let (x1, y1) = (cull.x + cull.width, cull.y + cull.height);
    let corners = [(cull.x, cull.y), (x1, cull.y), (cull.x, y1), (x1, y1)];
    let mut bounds = Rect {
        x: f64::INFINITY,
        y: f64::INFINITY,
        width: 0.0,
        height: 0.0,
    };
    let (mut max_x, mut max_y) = (f64::NEG_INFINITY, f64::NEG_INFINITY);
    for (x, y) in corners {
        let [px, py, _, pw] = matrix.transform_point([x, y, 0.0, 1.0]);
        if !(pw > 0.0 && pw.is_finite() && px.is_finite() && py.is_finite()) {
            return true;
        }
        bounds.x = bounds.x.min(px / pw);
        bounds.y = bounds.y.min(py / pw);
        max_x = max_x.max(px / pw);
        max_y = max_y.max(py / pw);
    }
    bounds.width = max_x - bounds.x;
    bounds.height = max_y - bounds.y;
    overlaps(bounds, clip)
}

/// Whether the listed `step` of node `n` needs a `Close` after its
/// descendants: it opens something (a layer, a transform, a group or a
/// clip), draws its `DrawOver` scene, or draws its scrollbars.
fn closes(step: &Step, n: &HostNode, host: &Model) -> bool {
    !step.opened().is_empty()
        || (step.own
            && (n.scenes()[1].is_some()
                || host.scrollbar_bars(step.id).iter().any(Option::is_some)))
}

/// The compositor layer a node is walked as, before its origin and clip
/// are placed.
struct LayerPlan {
    /// Layer space → the parent's space (the node's box at its origin).
    matrix: Matrix4,
    /// The same map when it is affine (culling maps the clip through it).
    affine: Option<Transform>,
    /// The content bounds in layer space (before its effects).
    bounds: Rect,
    /// The content in layer space with its own effects' reach: what the
    /// subtree is culled within when the map is not affine.
    cull: Rect,
    plane: Plane,
    backface: BackfaceVisibility,
    grid: RasterGrid,
}

struct Walk<'a> {
    host: &'a Model,
    margin: f64,
    indices: &'a mut HashMap<NodeId, ChildIndex>,
    /// What each subtree holds (decides the layers).
    facts: &'a HashMap<NodeId, SubtreeFacts>,
    /// The layout pass the tree is laid out by.
    pass: Generation,
    /// The generation of the nodes' inks.
    generation: Generation,
    /// The containers whose index this walk used.
    used: HashSet<NodeId>,
    out: Vec<Entry>,
    /// The listed transformed nodes' transforms.
    transforms: Vec<Transform>,
    /// The listed layers.
    layers: Vec<LayerSpec>,
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
        clip: Rect,
        parent_perspective: Option<Matrix4>,
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
        // A parent's perspective moves only a node that has a transform of
        // its own; the others stay in the plane of their parent.
        let perspective = parent_perspective.filter(|_| local.is_some());
        let plan = self.plan(id, n, &layout, &look, origin, perspective);
        let visible = match &plan {
            // A layer with a non-affine map is culled by its projected content.
            Some(p) if p.affine.is_none() => projected_overlaps(&p.matrix, p.cull, clip),
            _ => overlaps(placed(local.as_ref(), own, origin), clip),
        };
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
        let Some((transform, clip, o, layer)) = self.place(local, clip, origin, plan) else {
            return;
        };
        let clips = n.clips();
        let presence_clip = presence_clip(host, id, clip, o, layout.size);
        let (group, grouped, alpha) = opacity(host, id, n, &look, parent_alpha);
        // A layer applies its node's opacity itself.
        let alpha = if layer.is_some() { parent_alpha } else { alpha };
        let step = Step {
            id,
            origin: o,
            transform,
            alpha,
            clips,
            presence_clip,
            rounded: look.rounded,
            own: visible,
            grouped,
            group,
            clip,
            layer,
        };
        let listed = visible || grouped || transform.is_some() || layer.is_some();
        if listed {
            self.out.push(Entry::Open(step));
        }
        let child_clip = self.child_clip(&look, o, layout.size, clip, clips, presence_clip);
        let scroll = n.scroll();
        // The children's layout locations are relative to this origin.
        let base = Point {
            x: o.x - scroll.x,
            y: o.y - scroll.y,
        };
        let perspective = if look.perspective {
            host.perspective_at(id, o)
        } else {
            None
        };
        self.children(id, n, base, alpha, child_clip, perspective);
        if listed && closes(&step, n, host) {
            self.out.push(Entry::Close(step));
        }
    }

    /// The area a node's children may draw in: its own area grown by its
    /// filters, cut to its padding box when it clips and to its presence
    /// clip (`clip` is in its space, `o` its box's origin).
    fn child_clip(
        &self,
        look: &Look,
        o: Point,
        size: kivact_types::Size,
        clip: Rect,
        clips: bool,
        presence_clip: Option<Rect>,
    ) -> Rect {
        let clip = grow(clip, look.fx);
        let child_clip = if clips {
            let padding = look.inside(Rect {
                x: o.x,
                y: o.y,
                width: size.width,
                height: size.height,
            });
            clip.intersect(&grow(padding, self.margin))
        } else {
            clip
        };
        presence_clip.map_or(child_clip, |p| child_clip.intersect(&p))
    }

    /// The layer `id` is walked as, if it is one (`origin` is its box's
    /// origin in its parent's space).
    fn plan(
        &self,
        id: NodeId,
        n: &HostNode,
        layout: &kivact_layout::NodeLayout,
        look: &Look,
        origin: Point,
        parent_perspective: Option<Matrix4>,
    ) -> Option<LayerPlan> {
        if !self.facts.get(&id).is_some_and(|f| f.layer) {
            return None;
        }
        let host = self.host;
        let own = host
            .local_matrix(id, layout.size)
            .unwrap_or(Matrix4::IDENTITY);
        // M = Mn · T(o) · Pp: the parent's perspective after the placement.
        let matrix = own
            .then(Matrix4::translate(origin.x, origin.y, 0.0))
            .then(parent_perspective.unwrap_or(Matrix4::IDENTITY));
        let moves = host.motion.moves_layer(id) || host.layout_motion_transform(id).is_some();
        let content = ink_local(host, id, n, layout, self.generation);
        Some(LayerPlan {
            matrix,
            affine: matrix.as_affine(),
            bounds: content,
            cull: grow(content, look.fx),
            plane: if look.plane_3d() {
                Plane::Preserve3d
            } else {
                Plane::Flat
            },
            backface: if look.backface_hidden {
                BackfaceVisibility::Hidden
            } else {
                BackfaceVisibility::Visible
            },
            grid: if moves || look.will_change == WillChange::Transform {
                RasterGrid::Layer
            } else {
                RasterGrid::Target
            },
        })
    }

    /// Place a node: its transform index (a transform pushed around it),
    /// the clip its subtree is culled in, its origin and its layer. A layer
    /// is placed at its own origin, in its own space.
    fn place(
        &mut self,
        local: Option<Transform>,
        clip: Rect,
        origin: Point,
        plan: Option<LayerPlan>,
    ) -> Option<(Option<u32>, Rect, Point, Option<u32>)> {
        if let Some(plan) = plan {
            // A layer's subtree is culled in its space: through the inverse of
            // an affine map, else within its bounds.
            let clip = match plan.affine.and_then(|t| t.invert()) {
                Some(back) => map_rect(Some(&back), clip),
                None => plan.cull,
            };
            self.layers.push(LayerSpec {
                matrix: plan.matrix,
                bounds: plan.bounds,
                plane: plan.plane,
                backface: plan.backface,
                grid: plan.grid,
            });
            let index = (self.layers.len() - 1) as u32;
            return Some((None, clip, Point::default(), Some(index)));
        }
        let (transform, clip, o) = self.place_transform(local, clip, origin)?;
        Some((transform, clip, o, None))
    }

    fn place_transform(
        &mut self,
        local: Option<Transform>,
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

    /// The children of `id` (its node `n`), whose layout locations are
    /// relative to `base`: through its child index when it has many.
    fn children(
        &mut self,
        id: NodeId,
        n: &HostNode,
        base: Point,
        alpha: f64,
        clip: Rect,
        perspective: Option<Matrix4>,
    ) {
        let host = self.host;
        let place = |at: Point| Point {
            x: base.x + at.x,
            y: base.y + at.y,
        };
        if host.tree().flat_children(id).len() < INDEXED {
            for child in host.stacked(n) {
                let Some(c) = host
                    .node(child)
                    .filter(|c| !c.is_layer() && c.properties.drawable.is_none())
                else {
                    continue;
                };
                let at = host.location(c);
                self.node(child, place(at), alpha, clip, perspective);
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
            self.node(child, place(at), alpha, clip, perspective);
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
            let viewport = Rect {
                x: -margin,
                y: -margin,
                width: window.width + 2.0 * margin,
                height: window.height + 2.0 * margin,
            };
            let stale = std::mem::replace(
                &mut self.paint.stale_facts,
                StaleFacts::Nodes(BTreeSet::new()),
            );
            self.refresh_facts(stale, &mut list.facts);
            let mut walk = Walk {
                host: self,
                margin,
                indices: &mut list.indices,
                facts: &list.facts,
                pass: list.pass,
                generation: list.ink_gen,
                used: HashSet::new(),
                out: Vec::with_capacity(list.entries.len()),
                transforms: Vec::new(),
                layers: Vec::new(),
            };
            if let Some(root) = self.tree().root() {
                walk.node(root, Point::default(), 1.0, viewport, None);
            }
            for (layer, origin) in self.layers() {
                walk.node(layer, origin, 1.0, viewport, None);
            }
            let Walk {
                out,
                used,
                transforms,
                layers,
                ..
            } = walk;
            list.transforms = transforms;
            list.layers = layers;
            // Indices of containers this walk did not reach (removed, or
            // culled with an ancestor) are built again when reached.
            list.indices.retain(|id, _| used.contains(id));
            list.entries = out;
            list.state = key;
        }
        self.paint.list = list;
    }

    /// Bring `facts` up to date with the nodes marked stale since the last list.
    pub(crate) fn refresh_facts(
        &self,
        stale: StaleFacts,
        facts: &mut HashMap<NodeId, SubtreeFacts>,
    ) {
        match stale {
            StaleFacts::All => *facts = self.layer_facts(),
            StaleFacts::Nodes(set) => {
                let tree = self.tree();
                let mut queue: BinaryHeap<(usize, NodeId)> = set
                    .into_iter()
                    .filter(|id| tree.contains(*id))
                    .map(|id| (tree.depth(id), id))
                    .collect();
                let mut done = BTreeSet::new();
                while let Some((_, id)) = queue.pop() {
                    if !done.insert(id) {
                        continue;
                    }
                    let parent = if self.is_layer_node(id) {
                        // A layer root starts afresh: its facts are derived with no parent.
                        None
                    } else {
                        tree.get(id)
                            .and_then(|n| tree.flat_parent(n))
                            // A chain of transparent ancestors: the root is the
                            // layer root the facts were derived from.
                            .or_else(|| tree.root().filter(|root| *root != id))
                    };
                    let parent_look = parent.and_then(|p| self.drawn_look(p));
                    let new = crate::layer::node_facts(self, id, parent_look.as_ref(), facts);
                    let old = facts.get(&id).copied().unwrap_or_default();
                    if new == SubtreeFacts::default() {
                        facts.remove(&id);
                    } else {
                        facts.insert(id, new);
                    }
                    if new != old
                        && !self.is_layer_node(id)
                        && let Some(p) = parent
                    {
                        queue.push((tree.depth(p), p));
                    }
                }
            }
        }
        #[cfg(debug_assertions)]
        debug_assert_eq!(
            *facts,
            self.layer_facts(),
            "paint: retained subtree facts diverged"
        );
    }

    /// Build an independent list in the target's local output space.
    pub(crate) fn subtree_paint_list(&self, id: NodeId, clip: Rect) -> PaintList {
        let mut indices = HashMap::new();
        let mut facts = HashMap::new();
        self.subtree_facts(id, None, &mut facts);
        let bounds = self.rect(id).unwrap_or_default();
        let overlays: Vec<_> = self
            .layers()
            .into_iter()
            .filter(|(layer, _)| *layer != id && self.tree().is_in_subtree(*layer, id))
            .collect();
        for (layer, _) in &overlays {
            self.subtree_facts(*layer, None, &mut facts);
        }
        let mut walk = Walk {
            host: self,
            margin: MARGIN,
            indices: &mut indices,
            facts: &facts,
            pass: self.paint.list.pass,
            generation: self.paint.list.ink_gen,
            used: HashSet::new(),
            out: Vec::new(),
            transforms: Vec::new(),
            layers: Vec::new(),
        };
        walk.node(id, Point::default(), 1.0, clip, None);
        for (layer, origin) in overlays {
            walk.node(
                layer,
                Point {
                    x: origin.x - bounds.x,
                    y: origin.y - bounds.y,
                },
                1.0,
                clip,
                None,
            );
        }
        PaintList {
            entries: walk.out,
            transforms: walk.transforms,
            layers: walk.layers,
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
) -> (bool, bool, f64) {
    // Group opacity over more than one thing it draws.
    let content = match n.kind() {
        _ if host.editors[n.index.0].is_some() => 2,
        NodeType::Text | NodeType::Image => 1,
        _ => 0,
    };
    let more = host.tree().flat_children(id).len() != 0
        || n.scenes().iter().any(Option::is_some)
        || look.items + content > 1
        || look.grouped;
    let group = look.opacity < 1.0 && more;
    let grouped = group || look.grouped;
    let alpha = if group {
        parent_alpha
    } else {
        parent_alpha * look.opacity
    };
    (group, grouped, alpha)
}
