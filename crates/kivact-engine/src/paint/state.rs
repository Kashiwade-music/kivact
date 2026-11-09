//! Role: The paint group's retained paint list and derived node caches.
//!
//! The fields are the paint modules' alone; the model, the host and the
//! layout pass use the methods (`PaintState`).
//!
//! The caches have one role each: a node's normalized base paint (`Look`)
//! and its ink, indexed by structural slot beside the model's
//! placement (`Place`); none copies the
//! typed props into a general record of facts. While there is no surface
//! no redraw is asked for (`Host::paint_due`); `Redraw::Needed` stays set
//! and the restored surface draws the frame.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};

use kivact_types::cell::Generation;
use kivact_types::geometry3d::Matrix4;
use kivact_types::paint::BackfaceVisibility;
use kivact_types::render::{Plane, RasterGrid};
use kivact_types::{NodeId, Point, Rect, Size};

pub(crate) use crate::model::{Look, SubtreeFacts};

/// The parameters of one walk over a paint list; nothing in [`PaintState`]
/// is changed for its duration (it replaces temporary scale and list swaps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaintPass {
    /// Device pixels per logical pixel the walk draws at (inside a
    /// transform, enlarged by its zoom).
    pub(crate) scale: f64,
    /// The logical size of the root layer.
    pub(crate) size: Size,
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

/// The paint group of the model: derived caches and whether a
/// frame must be redrawn. Only the paint modules read the caches.
pub(crate) struct PaintState {
    pub(crate) ink: Vec<Cell<Option<Ink>>>,
    pub(crate) look: Vec<Option<Look>>,
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
}

impl PaintState {
    /// Initialize an empty retained list that needs its first frame.
    pub(crate) fn new() -> Self {
        Self {
            ink: Vec::new(),
            look: Vec::new(),
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
}

/// Everything a node's subtree can draw on, relative to its layout
/// location, and the layout and generation it was found for (see the
/// module invariants).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Ink {
    /// `None`: nothing (hidden).
    pub(super) rect: Option<Rect>,
    pub(super) size: Size,
    pub(super) content: Size,
    pub(super) generation: Generation,
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

/// How one listed node is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Step {
    pub(super) id: NodeId,
    /// Its untransformed layout box's origin, in the space of the
    /// transforms pushed around it (its own and its ancestors'
    /// translations below them applied). A layer's origin is its own box.
    pub(super) origin: Point,
    // xtask-allow(state-fields): passive record - The transform it is drawn through (an index into [`PaintList::transforms`])
    /// The transform it is drawn through (an index into
    /// [`PaintList::transforms`]).
    pub(super) transform: Option<u32>,
    /// The alpha its colors are faded by (outside a group: its opacity
    /// times its ancestors'; inside a layer: its ancestors' only).
    pub(super) alpha: f64,
    // xtask-allow(state-fields): passive record - clips
    pub(super) clips: bool,
    // xtask-allow(state-fields): passive record - presence clip
    pub(super) presence_clip: Option<Rect>,
    // xtask-allow(state-fields): passive record - Its corners are rounded (a clip is a layer clipped to its rounded padding box, not a rect clip)
    /// Its corners are rounded (a clip is a layer clipped to its rounded
    /// padding box, not a rect clip).
    pub(super) rounded: bool,
    // xtask-allow(state-fields): passive record - It draws itself (else it is listed for its group only)
    /// It draws itself (else it is listed for its group only).
    pub(super) own: bool,
    // xtask-allow(state-fields): passive record - It is composited in a group
    /// It is composited in a group.
    pub(super) grouped: bool,
    // xtask-allow(state-fields): passive record - Its opacity is the group's (applied by its group)
    /// Its opacity is the group's (applied by its group).
    pub(super) group: bool,
    /// The area its ancestors let it draw in (grown by the culling
    /// margin), in the space of its origin: a text input draws only the
    /// lines that reach it.
    pub(super) clip: Rect,
    // xtask-allow(state-fields): passive record - Its compositor layer (an index into [`PaintList::layers`])
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
            if self.grouped {
                out.push(Opening::Group);
            }
        }
        if self.presence_clip.is_some() {
            out.push(Opening::Presence);
        }
        if self.clips {
            out.push(if self.rounded {
                Opening::ClipGroup
            } else {
                Opening::Clip
            });
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

/// The paint list and what it was built for.
#[derive(Clone, Debug, Default)]
pub(crate) struct PaintList {
    pub(super) entries: Vec<Entry>,
    /// The transforms of the listed transformed nodes, each in the space
    /// of the transforms around it.
    pub(super) transforms: Vec<kivact_types::Transform>,
    /// The compositor layers of the listed layer nodes.
    pub(super) layers: Vec<LayerSpec>,
    /// What each subtree holds that decides its layers (only the nodes
    /// with something to say are kept).
    pub(super) facts: HashMap<NodeId, SubtreeFacts>,
    /// Whether the list is reusable for its window size and raster scale.
    pub(super) state: ListState,
    /// The child indices of the containers with many children.
    pub(super) indices: HashMap<NodeId, ChildIndex>,
    /// Layout passes so far (the indices' geometry is from one of them).
    pub(super) pass: Generation,
    /// The generation of the nodes' inks: an ink of another is forgotten.
    pub(super) ink_gen: Generation,
}

/// What one child can draw on, relative to its layout location.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Extent {
    /// Hidden: nothing.
    Nothing,
    /// Its subtree may draw anywhere (its extent is not finite).
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
    pub(super) size: Size,
    pub(super) content: Size,
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
    /// The layout pass the entries' geometry is from.
    pub(super) pass: Generation,
    /// Children whose own drawing changed since.
    pub(super) touched: Vec<NodeId>,
}

impl PaintState {
    pub(crate) fn push_slot(&mut self) {
        self.ink.push(Cell::new(None));
        self.look.push(None);
    }
    pub(crate) fn reset_slot(&mut self, slot: usize) {
        self.ink[slot].set(None);
        self.look[slot] = None;
    }
}
