//! Role: Rebuild the paint list by walking the tree in paint order within the view.
//!
//! A layer node (see `layers.rs`) is walked in its own space: its box at the
//! origin, its children's origins and clips relative to it, its transform
//! kept as a [`LayerSpec`]. Any other node is walked in the space of the
//! transforms pushed around it, as before.

use super::ink::{drawn, grow, ink_local, local_transform, overlaps, placed, reach};
use crate::contract::NodeType;
use crate::layer::LayerView;
use crate::model::Model;
use crate::model::transform::map_rect;
use crate::paint::state::{
    ChildIndex, ClipShape, Entry, Grouping, LayerSpec, LayerTraits, ListState, MovingLayer,
    PaintList, ScrollerWalk, StaleFacts, Step, SubtreeFacts,
};
use crate::tree::node::{HostNode, NodeIndex};
use arvane_protocol::{Plane, RasterGrid};
use kivact_types::cell::Generation;
use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::BackfaceVisibility;
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

/// Whether the transparent (`display: contents`) nodes between `child` and
/// its walk parent (at `parent`) are drawn. The walk reaches a child through
/// them when it flattens them, so their own hiding decides for the child.
fn through_drawn(host: &Model, child: &HostNode, parent: NodeIndex) -> bool {
    host.tree()
        .ancestors(child.index)
        .skip(1)
        .take_while(|(i, _)| *i != parent)
        .all(|(_, n)| !n.hidden() && !n.visibility_hidden())
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
    /// The layer moves every frame (see `Model::refresh_moving_layers`).
    moves: bool,
    /// The box's origin in the parent's space, as walked.
    origin: Point,
    /// The parent's perspective it was walked with.
    perspective: Option<Matrix4>,
}

struct Walk<'a> {
    host: &'a Model,
    margin: f64,
    indices: &'a mut HashMap<NodeId, ChildIndex>,
    /// What each subtree holds (decides the layers).
    facts: &'a HashMap<NodeId, SubtreeFacts>,
    /// The generation of the nodes' inks.
    generation: Generation,
    /// The containers whose index this walk used.
    used: HashSet<NodeId>,
    out: Vec<Entry>,
    /// The listed transformed nodes' transforms.
    transforms: Vec<Transform>,
    /// The listed layers.
    layers: Vec<LayerSpec>,
    /// The listed layers that move every frame.
    moving: Vec<MovingLayer>,
    /// The layers the walk is inside, outermost first, with their indices.
    outer: Vec<(NodeId, u32)>,
    /// The listed scroll containers and where they were walked from.
    scrollers: HashMap<NodeId, ScrollerWalk>,
    /// The look each listed node with state styles or a transition was walked with.
    looks: HashMap<NodeId, LayerTraits>,
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
        let walked = ScrollerWalk {
            origin,
            alpha: parent_alpha,
            clip,
            perspective: parent_perspective,
        };
        self.keep_scroller(host, id, walked);
        // The walk reaches a node only from a drawn parent (through the
        // transparent nodes between them, see `through_drawn`), or as a root
        // or layer `Model::layers` found drawn: its own visibility decides.
        if n.hidden() || n.visibility_hidden() {
            return;
        }
        debug_assert!(
            host.tree().drawn(id),
            "invariant: the paint walk reaches only drawn nodes"
        );
        let layout = host.layout_at(n.index).unwrap_or_default();
        let look = LayerTraits::now(host, id, n);
        if n.states().is_some() || host.motion.get(id).is_some() {
            self.looks.insert(id, look);
        }
        let local = local_transform(host, id, n, &layout);
        let own = reach(host, id, n, &layout, &look);
        // A parent's perspective moves only a node that has a transform of
        // its own; the others stay in the plane of their parent.
        let perspective = parent_perspective.filter(|_| local.is_some());
        let plan = self.plan(id, n, &layout, &look, origin, perspective);
        // A moving layer is listed whole: its matrix changes every frame
        // without a new list, so it is not culled.
        let moving = plan.as_ref().is_some_and(|p| p.moves);
        let visible = moving
            || match &plan {
                // A layer with a non-affine map is culled by its projected content.
                Some(p) if p.affine.is_none() => projected_overlaps(&p.matrix, p.cull, clip),
                _ => overlaps(placed(local.as_ref(), own, origin), clip),
            };
        if !visible && !self.subtree_shows(id, n, &layout, origin, clip) {
            return;
        }
        let Some((transform, clip, o, layer)) = self.place(id, local, clip, origin, plan) else {
            return;
        };
        let clips = n.clips();
        let presence_clip = presence_clip(host, id, clip, o, layout.size);
        let (grouping, alpha) = opacity(host, id, n, &look, parent_alpha, moving);
        // A layer applies its node's opacity itself.
        let alpha = if layer.is_some() { parent_alpha } else { alpha };
        let clip_shape = match (clips, look.rounded()) {
            (false, _) => ClipShape::None,
            (true, false) => ClipShape::Rect,
            (true, true) => ClipShape::Rounded,
        };
        let step = Step {
            id,
            origin: o,
            transform,
            alpha,
            clip_shape,
            presence_clip,
            own: visible,
            grouping,
            clip,
            layer,
        };
        let listed =
            visible || grouping != Grouping::None || transform.is_some() || layer.is_some();
        if listed {
            self.out.push(Entry::Open(step));
        }
        let child_clip = self.child_clip(&look, o, layout.size, clip, clips, presence_clip);
        let (base, perspective) = children_frame(host, id, n, o, &look);
        let opened = layer.filter(|_| host.is_layer_node(id));
        if let Some(index) = opened {
            self.outer.push((id, index));
        }
        self.children(id, n, base, alpha, child_clip, perspective);
        if opened.is_some() {
            self.outer.pop();
        }
        if listed && closes(&step, n, host) {
            self.out.push(Entry::Close(step));
        }
    }

    /// Keep how `id` was walked when it is a scroll container: its subtree
    /// is walked again when it scrolls (`Model::rewalk_scrolled`).
    fn keep_scroller(&mut self, host: &Model, id: NodeId, walked: ScrollerWalk) {
        if host.tree().scrollers.contains(&id) {
            self.scrollers.insert(id, walked);
        }
    }

    /// Whether a node that does not show itself shows through its subtree:
    /// a box of the subtree is drawn outside its ink (projected, or placed
    /// by a drawing).
    fn subtree_shows(
        &self,
        id: NodeId,
        n: &HostNode,
        layout: &kivact_layout::NodeLayout,
        origin: Point,
        clip: Rect,
    ) -> bool {
        drawn(self.host, id, n, layout, self.generation).is_some_and(|d| {
            let at = Rect {
                x: d.rect.x + origin.x,
                y: d.rect.y + origin.y,
                ..d.rect
            };
            d.beyond || overlaps(at, clip)
        })
    }

    /// The area a node's children may draw in: its own area grown by its
    /// filters, cut to its padding box when it clips and to its presence
    /// clip (`clip` is in its space, `o` its box's origin).
    fn child_clip(
        &self,
        look: &LayerTraits,
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
        look: &LayerTraits,
        origin: Point,
        parent_perspective: Option<Matrix4>,
    ) -> Option<LayerPlan> {
        if !self.facts.get(&id).is_some_and(|f| f.layer()) {
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
            backface: if look.backface_hidden() {
                BackfaceVisibility::Hidden
            } else {
                BackfaceVisibility::Visible
            },
            grid: if moves || look.will_change == WillChange::Transform {
                RasterGrid::Layer
            } else {
                RasterGrid::Target
            },
            moves,
            origin,
            perspective: parent_perspective,
        })
    }

    /// Place a node: its transform index (a transform pushed around it),
    /// the clip its subtree is culled in, its origin and its layer. A layer
    /// is placed at its own origin, in its own space.
    fn place(
        &mut self,
        id: NodeId,
        local: Option<Transform>,
        clip: Rect,
        origin: Point,
        plan: Option<LayerPlan>,
    ) -> Option<(Option<u32>, Rect, Point, Option<u32>)> {
        if let Some(plan) = plan {
            // A layer's subtree is culled in its space: through the inverse of
            // an affine map, else within its bounds (a moving layer's, whose
            // map changes every frame).
            let clip = match plan.affine.and_then(|t| t.invert()).filter(|_| !plan.moves) {
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
            if plan.moves {
                self.moving.push(MovingLayer {
                    id,
                    index,
                    origin: plan.origin,
                    perspective: plan.perspective,
                    outer: self.outer.iter().rev().copied().collect(),
                });
            }
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
                let Some(c) = host.node(child).filter(|c| {
                    !c.is_layer()
                        && c.properties.drawable.is_none()
                        && through_drawn(host, c, n.index)
                }) else {
                    continue;
                };
                let at = host.location(c);
                self.node(child, place(at), alpha, clip, perspective);
            }
            return;
        }
        self.used.insert(id);
        let generation = self.generation;
        let index = self.indices.entry(id).or_insert_with(|| {
            ChildIndex::build(
                host,
                host.tree().flat_children(host.tree().nodes.id_at(n.index)),
                generation,
            )
        });
        index.refresh(host, generation);
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
            if !host
                .node(child)
                .is_some_and(|c| through_drawn(host, c, n.index))
            {
                continue;
            }
            let at = match host
                .node(child)
                .filter(|c| c.place.positioning == crate::tree::node::Positioning::Sticky)
            {
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
    /// state deltas) that its paint list steps are made of ([`LayerTraits::stacking`]).
    pub(crate) fn paint_stacking(&self, id: NodeId) -> impl PartialEq + use<> {
        LayerTraits::of(&self.target_paint(id)).stacking()
    }

    /// Bring the paint list up to date for the next frame: rebuilt if
    /// anything restacked since the last one (see the module invariants).
    /// Returns whether it was rebuilt.
    pub(crate) fn paint_list(&mut self) -> bool {
        let mut list = std::mem::take(&mut self.paint.list);
        let key = ListState::Built {
            size: self.window(),
            scale: self.paint.scale,
        };
        // A layer root placed by another node's motion (a submenu anchored
        // in a moving menu) moves without a restack: the list follows it.
        let rebuilt = list.state != key
            || self.layers() != list.layer_origins
            || (!list.rewalk.is_empty() && !self.rewalk_scrolled(&mut list));
        if rebuilt {
            self.build_list(&mut list, key);
        }
        list.rewalk.clear();
        self.paint.list = list;
        rebuilt
    }

    /// Walk every subtree of the root and the layers anew into `list`, which
    /// is then of the size and scale `key` names.
    fn build_list(&mut self, list: &mut PaintList, key: ListState) {
        let window = self.window();
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
        let roots = self.layers();
        let mut walk = Walk {
            host: self,
            margin,
            indices: &mut list.indices,
            facts: &list.facts,
            generation: list.ink_gen,
            used: HashSet::new(),
            out: Vec::with_capacity(list.entries.len()),
            transforms: Vec::new(),
            layers: Vec::new(),
            moving: Vec::new(),
            outer: Vec::new(),
            scrollers: HashMap::new(),
            looks: HashMap::new(),
        };
        if let Some(root) = self.tree().root() {
            walk.node(root, Point::default(), 1.0, viewport, None);
        }
        for &(layer, origin) in &roots {
            walk.node(layer, origin, 1.0, viewport, None);
        }
        let Walk {
            out,
            used,
            transforms,
            layers,
            moving,
            scrollers,
            looks,
            ..
        } = walk;
        list.transforms = transforms;
        list.layers = layers;
        list.moving = moving;
        list.scrollers = scrollers;
        list.looks = looks;
        list.layer_origins = roots;
        // Indices of containers this walk did not reach (removed, or
        // culled with an ancestor) are built again when reached.
        list.indices.retain(|id, _| used.contains(id));
        list.entries = out;
        list.state = key;
    }

    /// Walk the subtrees of the scrolled containers again in place of
    /// their old steps; `false` when one cannot be (it is not listed with a
    /// close, or a layer's window origin moved): the caller builds the list.
    /// The old steps' transforms and layers are dropped (`compact`); the
    /// nodes that left the list lose their walk records and kept drawings.
    fn rewalk_scrolled(&self, list: &mut PaintList) -> bool {
        if self.layers() != list.layer_origins {
            return false;
        }
        let tree = self.tree();
        // Outermost first: a container inside another is walked with it.
        let ids: Vec<NodeId> = list
            .rewalk
            .iter()
            .copied()
            .filter(|id| {
                !list
                    .rewalk
                    .iter()
                    .any(|o| o != id && tree.is_in_subtree(*id, *o))
            })
            .collect();
        let margin = MARGIN.max(1.0 / self.paint.scale.max(f64::MIN_POSITIVE));
        for id in ids {
            let Some(at) = list.scrollers.get(&id).copied() else {
                return false;
            };
            let Some(open) = list
                .entries
                .iter()
                .position(|e| matches!(e, Entry::Open(s) if s.id == id))
            else {
                return false;
            };
            let Some(close) = list.entries[open..]
                .iter()
                .position(|e| matches!(e, Entry::Close(s) if s.id == id))
                .map(|k| open + k)
            else {
                return false;
            };
            let transforms = std::mem::take(&mut list.transforms);
            let layers = std::mem::take(&mut list.layers);
            let mut walk = Walk {
                host: self,
                margin,
                indices: &mut list.indices,
                facts: &list.facts,
                generation: list.ink_gen,
                used: HashSet::new(),
                out: Vec::new(),
                transforms,
                layers,
                moving: Vec::new(),
                outer: Vec::new(),
                scrollers: HashMap::new(),
                looks: HashMap::new(),
            };
            walk.node(id, at.origin, at.alpha, at.clip, at.perspective);
            let Walk {
                out,
                transforms,
                layers,
                moving,
                scrollers,
                looks,
                ..
            } = walk;
            list.transforms = transforms;
            list.layers = layers;
            let left = left_ids(&list.entries[open..=close], &out);
            list.moving.retain(|m| !tree.is_in_subtree(m.id, id));
            list.moving.extend(moving);
            list.scrollers.retain(|node, _| !left.contains(node));
            list.scrollers.extend(scrollers);
            list.looks.retain(|node, _| !left.contains(node));
            list.looks.extend(looks);
            list.entries.splice(open..=close, out);
            self.forget_left_draws(&left);
        }
        compact(list);
        true
    }

    /// Drop the kept drawings of the nodes `left` (they left the list).
    fn forget_left_draws(&self, left: &HashSet<NodeId>) {
        if left.is_empty() {
            return;
        }
        self.paint.draw_owners.borrow_mut().retain(|(id, slot)| {
            let keep = !left.contains(id);
            if !keep {
                self.slots.draws[*slot].set(None);
            }
            keep
        });
    }

    /// Set the matrices of the moving layers for this frame.
    pub(crate) fn refresh_moving_layers(&mut self) {
        let mut list = std::mem::take(&mut self.paint.list);
        for m in &list.moving {
            let Some(n) = self.node(m.id) else {
                continue;
            };
            let size = self.layout_at(n.index).unwrap_or_default().size;
            let own = self.local_matrix(m.id, size).unwrap_or(Matrix4::IDENTITY);
            let matrix = own
                .then(Matrix4::translate(m.origin.x, m.origin.y, 0.0))
                .then(m.perspective.unwrap_or(Matrix4::IDENTITY));
            if let Some(spec) = list.layers.get_mut(m.index as usize) {
                spec.matrix = matrix;
            }
        }
        self.refresh_outer_bounds(&mut list);
        self.paint.list = list;
    }

    /// The bounds of the layers that hold a moving one: their ink holds its
    /// moving box, so the inks from it up to them are taken again.
    fn refresh_outer_bounds(&self, list: &mut PaintList) {
        for m in &list.moving {
            let Some(&(outermost, _)) = m.outer.last() else {
                continue;
            };
            let tree = self.tree();
            let mut at = Some(m.id);
            while let Some(id) = at {
                if let Some(n) = tree.get(id) {
                    self.slots.ink[n.index.0].set(None);
                }
                if id == outermost {
                    break;
                }
                at = tree.get(id).and_then(|n| tree.flat_parent(n));
            }
            for &(id, index) in &m.outer {
                let Some(n) = self.node(id) else {
                    continue;
                };
                let layout = self.layout_at(n.index).unwrap_or_default();
                if let Some(spec) = list.layers.get_mut(index as usize) {
                    spec.bounds = ink_local(self, id, n, &layout, list.ink_gen);
                }
            }
        }
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
            generation: self.paint.list.ink_gen,
            used: HashSet::new(),
            out: Vec::new(),
            transforms: Vec::new(),
            layers: Vec::new(),
            moving: Vec::new(),
            outer: Vec::new(),
            scrollers: HashMap::new(),
            looks: HashMap::new(),
        };
        if self.tree().drawn(id) {
            walk.node(id, Point::default(), 1.0, clip, None);
        }
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

/// The node of a list entry.
fn entry_id(entry: &Entry) -> NodeId {
    match entry {
        Entry::Open(step) | Entry::Close(step) => step.id,
    }
}

/// The nodes the old steps `old` list and the new steps `new` do not.
fn left_ids(old: &[Entry], new: &[Entry]) -> HashSet<NodeId> {
    let listed: HashSet<NodeId> = new.iter().map(entry_id).collect();
    old.iter()
        .map(entry_id)
        .filter(|id| !listed.contains(id))
        .collect()
}

/// Drop the transforms and layers no listed step refers to (a walk
/// again of a scroll container leaves its old ones behind), renumbering
/// them in the order the steps first refer to them: the order a walk
/// pushes them, so the list equals one built anew. A moving layer no
/// step refers to any more is dropped too.
fn compact(list: &mut PaintList) {
    let PaintList {
        entries,
        transforms,
        layers,
        moving,
        ..
    } = list;
    let mut transform_at = vec![None; transforms.len()];
    let mut layer_at = vec![None; layers.len()];
    let mut kept_transforms = Vec::new();
    let mut kept_layers = Vec::new();
    for entry in entries.iter_mut() {
        let (Entry::Open(step) | Entry::Close(step)) = entry;
        if let Some(t) = &mut step.transform {
            let old = *t as usize;
            *t = *transform_at[old].get_or_insert_with(|| {
                kept_transforms.push(transforms[old]);
                (kept_transforms.len() - 1) as u32
            });
        }
        if let Some(l) = &mut step.layer {
            let old = *l as usize;
            *l = *layer_at[old].get_or_insert_with(|| {
                kept_layers.push(layers[old]);
                (kept_layers.len() - 1) as u32
            });
        }
    }
    moving.retain_mut(
        |m| match layer_at.get(m.index as usize).copied().flatten() {
            Some(index) => {
                m.index = index;
                true
            }
            None => false,
        },
    );
    *transforms = kept_transforms;
    *layers = kept_layers;
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

/// The grouping and the alpha of a node drawn with `look`. A moving layer
/// (`moving`) applies its opacity itself, and its steps are not listed
/// again for it (`relooked`): they carry the opacity at rest.
fn opacity(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    look: &LayerTraits,
    parent_alpha: f64,
    moving: bool,
) -> (Grouping, f64) {
    let opacity = if moving { 1.0 } else { look.opacity };
    // Group opacity over more than one thing it draws.
    let content = match n.kind() {
        _ if host.slots.editors[n.index.0].is_some() => 2,
        NodeType::Text | NodeType::Image => 1,
        _ => 0,
    };
    let more = host.tree().flat_children(id).len() != 0
        || n.scenes().iter().any(Option::is_some)
        || look.items + content > 1
        || look.grouped();
    let group = opacity < 1.0 && more;
    let grouping = match (group, look.grouped()) {
        (true, _) => Grouping::Opacity,
        (false, true) => Grouping::Effects,
        (false, false) => Grouping::None,
    };
    let alpha = if group {
        parent_alpha
    } else {
        parent_alpha * opacity
    };
    (grouping, alpha)
}

/// Where the children of `id` (whose box starts at `o`) are walked from:
/// the origin their layout locations are relative to (scrolled), and the
/// perspective `id` gives them.
fn children_frame(
    host: &Model,
    id: NodeId,
    n: &HostNode,
    o: Point,
    look: &LayerTraits,
) -> (Point, Option<Matrix4>) {
    let scroll = n.scroll();
    let base = Point {
        x: o.x - scroll.x,
        y: o.y - scroll.y,
    };
    let perspective = if look.perspective() {
        host.perspective_at(id, o)
    } else {
        None
    };
    (base, perspective)
}
