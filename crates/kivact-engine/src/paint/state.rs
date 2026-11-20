//! Role: The paint group's retained paint list and derived node caches.
//!
//! The fields are the paint modules' alone; the model, the host and the
//! layout pass use the methods (`PaintState`).
//!
//! The caches have one role each: a node's normalized base paint (`LayerTraits`)
//! and its ink, indexed by structural slot beside the model's
//! placement (`Place`); none copies the
//! typed props into a general record of facts. While there is no surface
//! no redraw is asked for (`Host::paint_due`); `Redraw::Needed` stays set
//! and the restored surface draws the frame.

use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::rc::Rc;

use arvane_protocol::{Composition, LayerKey, Plane, RasterGrid, RenderCmd};
use kivact_types::cell::Generation;
use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::BackfaceVisibility;
use kivact_types::{NodeId, PaintProps, Point, Rect, Size};

use crate::lower::LayerChange;

pub(crate) use crate::model::{LayerTraits, SubtreeFacts};

/// The parameters of one walk over a paint list; nothing in [`PaintState`]
/// is changed for its duration (it replaces temporary scale and list swaps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaintPass {
    /// Device pixels per logical pixel the walk draws at (inside a
    /// transform, enlarged by its zoom).
    pub(crate) scale: f64,
    /// The logical size of the root layer.
    pub(crate) size: Size,
    /// Whether the walk reads and keeps the nodes' drawings (the window's
    /// frame only; exports and snapshots draw afresh).
    pub(crate) cache: bool,
}

/// What one listed node drew in the last frame it was drawn, kept while
/// nothing it is drawn from changes (`paint::frame_scene`). Only listed
/// nodes keep one; the list's rebuild drops the others (`Model::evict_draws`).
#[derive(Debug)]
pub(crate) struct NodeDraw {
    /// Where and at which scale it was drawn.
    pub(super) key: DrawKey,
    /// Its paint as drawn: state deltas, transition and layout-run radii
    /// applied, sanitized (`interact::visual::paint`).
    pub(super) paint: Rc<PaintProps>,
    pub(super) recorded: Recorded,
}

/// The place, size and scale a node's drawing was made for, and whether
/// the step drew the node itself: a step with another one draws it again
/// (a layout may resize a node in place, and one node may be listed by a
/// step that draws it and by one that only opens its group).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct DrawKey {
    pub(super) origin: Point,
    pub(super) size: Size,
    pub(super) own: bool,
    pub(super) alpha: f64,
    pub(super) clip: Rect,
    pub(super) scale: f64,
}

impl DrawKey {
    /// The key of `step`'s node, of `size`, drawn in `pass`.
    pub(crate) fn of(step: &Step, size: Size, pass: PaintPass) -> Self {
        DrawKey {
            origin: step.origin,
            size,
            own: step.own,
            alpha: step.alpha,
            clip: step.clip,
            scale: pass.scale,
        }
    }
}

/// What a node's drawing left to replay.
#[derive(Debug)]
pub(crate) enum Recorded {
    /// Its own commands: the box (between the outer and inner openings),
    /// the content (after the inner openings) and, once its close was
    /// drawn, the draw-over scene and scrollbars.
    Commands {
        boxed: Vec<RenderCmd>,
        content: Vec<RenderCmd>,
        over: Option<Vec<RenderCmd>>,
    },
    /// It opens layers or items, or draws pixels (images, patterns): it
    /// is lowered every frame (its paint is still kept).
    Live,
}

impl Recorded {
    /// The record of a box and a content part (`None`: a part could not be recorded).
    pub(crate) fn of(boxed: Option<Vec<RenderCmd>>, content: Option<Vec<RenderCmd>>) -> Self {
        match (boxed, content) {
            (Some(boxed), Some(content)) => Recorded::Commands {
                boxed,
                content,
                over: None,
            },
            _ => Recorded::Live,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Redraw {
    Clean,
    Needed,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) enum ListState {
    #[default]
    Stale,
    Built {
        size: Size,
        scale: f64,
    },
}

/// The nodes whose subtree facts must be derived again before the next
/// paint list (a work queue, not a phase).
#[derive(Default)]
pub(crate) enum StaleFacts {
    /// Every fact is derived again (first list, new root, layer roots changed).
    #[default]
    All,
    /// Only these nodes (and, through propagation, their ancestors).
    Nodes(BTreeSet<NodeId>),
}

// xtask-allow(state): legacy: scene_cache, dropped_reported, dropped_draws, drawables, misplaced_reported, twice_reported; draw_owners lists the slots holding a kept drawing (keep_draw, evict_draws, forget_draws); rewalked says the list was walked again in part since the last frame (paint_list), taken by stamp_versions
/// The paint group of the model: derived caches and whether a
/// frame must be redrawn. Only the paint modules read the caches.
pub(crate) struct PaintState {
    pub(super) scale: f64,
    pub(crate) scene_cache: RefCell<crate::lower::SceneCache>,
    pub(super) dropped_reported: RefCell<BTreeSet<(NodeId, u8)>>,
    pub(super) dropped_draws: RefCell<Vec<(NodeId, u32)>>,
    /// The canvas drawables of the frame being composed (`canvas.rs`).
    pub(crate) drawables: RefCell<crate::lower::drawables::DrawablePlan>,
    /// The misplaced drawables already reported, by node.
    pub(super) misplaced_reported: RefCell<BTreeSet<(NodeId, kivact_types::DrawableId)>>,
    /// The drawables drawn twice (in both slots) already reported, by node.
    pub(super) twice_reported: RefCell<BTreeSet<(NodeId, kivact_types::DrawableId)>>,
    pub(super) list: PaintList,
    pub(super) redraw: Redraw,
    /// The nodes whose facts the next list derives again.
    pub(crate) stale_facts: StaleFacts,
    /// The nodes (and their slots) that keep a drawing in `Slots::draws`.
    pub(crate) draw_owners: RefCell<HashSet<(NodeId, usize)>>,
    /// The content version each layer of the window's last frame had.
    pub(super) versions: LayerVersions,
    /// The list was walked again in part since the last frame (see `rewalked`).
    pub(super) rewalked: bool,
}

/// The content version each layer of the window's last frame had.
#[derive(Debug, Default)]
pub(crate) struct LayerVersions(HashMap<LayerKey, u64>);

#[cfg(feature = "testing")]
thread_local! {
    /// A node whose kept drawing a redraw mark does not forget: the test
    /// driver's injected invalidation fault (`ArvaneTestHost::keep_stale_draw`),
    /// which the debug check of kept drawings must catch.
    pub(crate) static KEEP_STALE_DRAW: std::cell::Cell<Option<NodeId>> = const { std::cell::Cell::new(None) };
}

#[cfg(feature = "testing")]
thread_local! {
    /// A child the hit test's candidates leave out: the test driver's
    /// injected fault (`ArvaneTestHost::drop_hit_candidate`), which the
    /// debug self-check's complete walk must catch.
    pub(crate) static DROP_HIT_CANDIDATE: std::cell::Cell<Option<NodeId>> = const { std::cell::Cell::new(None) };
}

/// The sizes of a window's paint list and draw cache (the test driver reads them).
#[cfg(feature = "testing")]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PaintListSizes {
    /// The list's entries (opens and closes).
    pub entries: usize,
    /// The transforms the steps refer to.
    pub transforms: usize,
    /// The compositor layers the steps refer to.
    pub layers: usize,
    /// The nodes that keep a drawing.
    pub kept_draws: usize,
}

impl PaintState {
    /// Initialize an empty retained list that needs its first frame.
    pub(crate) fn new() -> Self {
        Self {
            scale: 1.0,
            scene_cache: RefCell::default(),
            dropped_reported: RefCell::default(),
            dropped_draws: RefCell::default(),
            drawables: RefCell::default(),
            misplaced_reported: RefCell::default(),
            twice_reported: RefCell::default(),
            list: PaintList::new(),
            redraw: Redraw::Needed,
            stale_facts: StaleFacts::All,
            draw_owners: RefCell::default(),
            versions: LayerVersions::default(),
            rewalked: false,
        }
    }

    /// The window's scale factor the next frame is drawn at.
    pub(crate) fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    /// The window's scale factor the next frame is drawn at, as last set.
    pub(crate) fn scale(&self) -> f64 {
        self.scale
    }

    /// Ask for the next frame to be drawn (a skipped export is due).
    pub(crate) fn request_redraw(&mut self) {
        self.redraw = Redraw::Needed;
    }

    /// Whether the next frame must be drawn.
    pub(crate) fn needs_redraw(&self) -> bool {
        self.redraw == Redraw::Needed
    }

    /// The frame was drawn.
    pub(crate) fn drawn(&mut self) {
        self.redraw = Redraw::Clean;
    }

    /// Forget what was kept for the nodes `live` says are gone.
    pub(crate) fn forget_missing(&mut self, live: impl Fn(NodeId) -> bool) {
        self.dropped_reported.get_mut().retain(|(id, _)| live(*id));
    }

    /// Derive `id`'s subtree facts again before the next list (and its ancestors as they change).
    pub(crate) fn stale_fact(&mut self, id: NodeId) {
        if let StaleFacts::Nodes(set) = &mut self.stale_facts {
            set.insert(id);
        }
    }

    /// Derive every subtree fact again before the next list.
    pub(crate) fn stale_all_facts(&mut self) {
        self.stale_facts = StaleFacts::All;
    }

    /// Forget the recorded facts of `id` (the node is gone).
    pub(crate) fn forget_fact(&mut self, id: NodeId) {
        self.list.facts.remove(&id);
    }

    /// Give each layer of the frame its content version: the last frame's
    /// when nothing in it changed and the list stayed, a fresh one otherwise
    /// (an untracked layer always).
    pub(crate) fn stamp_versions(
        &mut self,
        composition: &mut Composition,
        changes: &[LayerChange],
        rebuilt: bool,
    ) {
        let list_changed = rebuilt || std::mem::take(&mut self.rewalked);
        let mut seen = HashMap::with_capacity(composition.layers.len());
        for (layer, change) in composition.layers.iter_mut().zip(changes) {
            let kept = self.versions.0.get(&layer.key).copied();
            let version = match (change, kept) {
                (LayerChange::Kept, Some(version)) if !list_changed => version,
                _ => crate::lower::content_versions::fresh_content_version(),
            };
            layer.content_version = version;
            if *change != LayerChange::Untracked {
                seen.insert(layer.key, version);
            }
        }
        self.versions.0 = seen;
    }

    /// The content versions of the last frame's tracked layers, by key (the test driver reads them).
    #[cfg(feature = "testing")]
    pub(crate) fn layer_versions(&self) -> Vec<(LayerKey, u64)> {
        self.versions
            .0
            .iter()
            .map(|(key, version)| (*key, *version))
            .collect()
    }

    /// The sizes of the list and the draw cache (the test driver reads them).
    #[cfg(feature = "testing")]
    pub(crate) fn list_sizes(&self) -> PaintListSizes {
        PaintListSizes {
            entries: self.list.entries.len(),
            transforms: self.list.transforms.len(),
            layers: self.list.layers.len(),
            kept_draws: self.draw_owners.borrow().len(),
        }
    }
}

/// Everything a node's subtree can draw on, relative to its layout
/// location, and the layout and generation it was found for (see the
/// module invariants).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ink {
    /// `None`: nothing (hidden).
    pub(super) rect: Option<Drawn>,
    pub(super) size: Size,
    pub(super) content: Size,
    pub(super) generation: Generation,
}

/// Where a drawn subtree draws (an [`Ink`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Drawn {
    /// Everything it draws on as the paint list places it: each box mapped
    /// by the plane map of its transform.
    pub(super) rect: Rect,
    /// A box in it lands outside `rect` once drawn: one whose transform is
    /// not 2D (a perspective or a 3D rendering context projects it) or a
    /// drawable (its host's drawing places it). Its hits are not bounded
    /// by `rect`.
    pub(super) beyond: bool,
}

/// The compositor layer one listed node is drawn in.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LayerSpec {
    /// Layer space (the node's box at its origin) → the parent's space.
    pub(super) matrix: Matrix4,
    /// Its content bounds in layer space.
    pub(super) bounds: Rect,
    pub(super) plane: Plane,
    pub(super) backface: BackfaceVisibility,
    pub(super) grid: RasterGrid,
}

/// One thing a listed node opens, in the order it is opened.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Opening {
    /// Its compositor layer.
    Layer,
    /// Its transform (inline, not a layer).
    Transform,
    /// Its group (opacity, clip path, blend; inline, not a layer).
    Group,
    /// Its presence clip.
    Presence,
    /// Its padding box as a rect clip.
    Clip,
    /// Its padding box as a rounded group.
    ClipGroup,
}

/// How one listed node is drawn with its group.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum Grouping {
    /// Neither composited in a group nor faded by the group's opacity.
    #[default]
    None,
    /// Composited in a group (a clip, a filter, a blend): the group opens.
    Effects,
    /// Its opacity is the group's (applied by its group).
    Opacity,
}

/// How a listed node's padding box clips its content.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ClipShape {
    /// Nothing clips the node.
    #[default]
    None,
    /// A rect clip to the padding box.
    Rect,
    /// A rounded group: a layer clipped to the rounded padding box.
    Rounded,
}

/// How one listed node is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Step {
    pub(super) id: NodeId,
    /// Its untransformed layout box's origin, in the space of the
    /// transforms pushed around it (its own and its ancestors'
    /// translations below them applied). A layer's origin is its own box.
    pub(super) origin: Point,
    /// The transform it is drawn through (an index into
    /// [`PaintList::transforms`]).
    pub(super) transform: Option<u32>,
    /// The alpha its colors are faded by (outside a group: its opacity
    /// times its ancestors'; inside a layer: its ancestors' only).
    pub(super) alpha: f64,
    /// The clip of its padding box (none, a rect, or a rounded group).
    pub(super) clip_shape: ClipShape,
    pub(super) presence_clip: Option<Rect>,
    /// It draws itself (else it is listed for its group only).
    pub(super) own: bool,
    /// How it is composited in a group.
    pub(super) grouping: Grouping,
    /// The area its ancestors let it draw in (grown by the culling
    /// margin), in the space of its origin: a text input draws only the
    /// lines that reach it.
    pub(super) clip: Rect,
    /// Its compositor layer (an index into [`PaintList::layers`]).
    pub(super) layer: Option<u32>,
}

impl Step {
    /// What opening this step pushes, in order. Its layer comes
    /// first; its transform and group are not opened inside a layer,
    /// whose transform and effects are its own.
    pub(crate) fn opened(&self) -> Vec<Opening> {
        let mut out = Vec::with_capacity(5);
        if self.layer.is_some() {
            out.push(Opening::Layer);
        } else {
            if self.transform.is_some() {
                out.push(Opening::Transform);
            }
            if self.grouping != Grouping::None {
                out.push(Opening::Group);
            }
        }
        if self.presence_clip.is_some() {
            out.push(Opening::Presence);
        }
        match self.clip_shape {
            ClipShape::None => {}
            ClipShape::Rect => out.push(Opening::Clip),
            ClipShape::Rounded => out.push(Opening::ClipGroup),
        }
        out
    }
}

/// One step of the paint list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Entry {
    /// Open the node's group, draw its box, then (inside its clip when it
    /// clips) its canvas scene and its content.
    Open(Step),
    /// After its descendants: its `DrawOver` scene, then pop its clip and
    /// its group.
    Close(Step),
}

/// A listed layer that moves every frame (a running opacity or transform
/// transition, or a layout run): its matrix is set again before each
/// frame without a new list (`Model::refresh_moving_layers`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MovingLayer {
    pub(super) id: NodeId,
    /// Its index in `PaintList::layers`.
    pub(super) index: u32,
    /// Its box's origin in its parent's space, as walked.
    pub(super) origin: Point,
    /// The parent's perspective it was walked with.
    pub(super) perspective: Option<Matrix4>,
    /// The layers it is inside, nearest first, with their indices: their
    /// ink holds its moving box, so their bounds follow its motion.
    pub(super) outer: Vec<(NodeId, u32)>,
}

/// Where a listed scroll container was walked from, to walk its subtree
/// again after it scrolls (`Model::rewalk_scrolled`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollerWalk {
    pub(super) origin: Point,
    pub(super) alpha: f64,
    pub(super) clip: Rect,
    pub(super) perspective: Option<Matrix4>,
}

/// The paint list and what it was built for.
#[derive(Clone, Debug, Default)]
pub(crate) struct PaintList {
    pub(super) entries: Vec<Entry>,
    /// The transforms of the listed transformed nodes, each in the space
    /// of the transforms around it.
    pub(super) transforms: Vec<kivact_types::Affine>,
    /// The compositor layers of the listed layer nodes.
    pub(super) layers: Vec<LayerSpec>,
    /// What each subtree holds that decides its layers (only the nodes
    /// with something to say are kept).
    pub(super) facts: HashMap<NodeId, SubtreeFacts>,
    /// Whether the list is reusable for its window size and raster scale.
    pub(super) state: ListState,
    /// The child indices of the containers with many children.
    pub(super) indices: HashMap<NodeId, ChildIndex>,
    /// The generation of the nodes' inks: an ink of another is forgotten.
    pub(super) ink_gen: Generation,
    /// The listed layers that move every frame.
    pub(super) moving: Vec<MovingLayer>,
    /// The listed scroll containers and where they were walked from.
    pub(super) scrollers: HashMap<NodeId, ScrollerWalk>,
    /// The window origins of the layers (`Model::layers`) the list was built with.
    pub(super) layer_origins: Vec<(NodeId, Point)>,
    /// The look each listed node with state styles or a transition was walked with.
    pub(super) looks: HashMap<NodeId, LayerTraits>,
    /// The scroll containers whose subtrees are walked again before the next frame.
    pub(super) rewalk: BTreeSet<NodeId>,
    /// The children of each listed container that sorts them, in stacked order (paint and hit test share it).
    pub(super) stack_orders: HashMap<NodeId, Rc<[NodeId]>>,
}

/// What one child can draw on, relative to its layout location.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Extent {
    /// Hidden: nothing.
    Nothing,
    /// Its subtree may draw anywhere (its extent is not finite, or a box
    /// of it is drawn outside its ink).
    Anywhere,
    /// Within this rectangle.
    Within(Rect),
}

/// One child of an indexed container.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Indexed {
    pub(super) id: NodeId,
    /// Its place among the entries (the children's order).
    pub(super) order: usize,
    /// It is sticky.
    pub(super) sticky: bool,
    /// Its layout location in the container.
    pub(super) location: Point,
    pub(super) extent: Extent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Axis {
    X,
    Y,
}

/// The children of a container with many children.
#[derive(Clone, Debug)]
pub(crate) struct ChildIndex {
    pub(super) entries: Vec<Indexed>,
    /// The axis every reach is sorted along (by its start), if any.
    pub(super) axis: Option<Axis>,
    /// Along `axis`, per entry: where its reach starts (a hidden entry
    /// repeats the one before it), and the furthest end of the reaches up
    /// to it.
    pub(super) starts: Vec<f64>,
    pub(super) furthest: Vec<f64>,
    /// The entries of the sticky children.
    pub(super) sticky: Vec<usize>,
    /// Children whose own drawing or layout changed since.
    pub(super) touched: Vec<NodeId>,
}

impl PaintState {
    /// Whether the current list draws `id` itself (an `Open` step of its
    /// own); `false` while the list waits to be built again.
    pub(crate) fn lists_own(&self, id: NodeId) -> bool {
        self.list.state != ListState::Stale
            && self
                .list
                .entries
                .iter()
                .any(|e| matches!(e, Entry::Open(step) if step.id == id && step.own))
    }

    /// `child` of `parent` moved or resized in a layout pass: its entry in
    /// the parent's child index (if it has one) is looked at again.
    pub(crate) fn moved_child(&mut self, parent: NodeId, child: NodeId) {
        if let Some(index) = self.list.indices.get_mut(&parent) {
            index.touch(child);
        }
    }

    /// The look `id` was listed with, if it has state styles or a transition.
    pub(crate) fn listed_look(&self, id: NodeId) -> Option<LayerTraits> {
        self.list.looks.get(&id).copied()
    }

    /// The scroll containers `scrolled` moved their content: their subtrees
    /// are walked again before the next frame (a list built anew does it all).
    pub(crate) fn rewalk_after(&mut self, scrolled: impl IntoIterator<Item = NodeId>) {
        self.list.rewalk.extend(scrolled);
    }
}
