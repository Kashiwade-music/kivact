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
use kivact_types::{Edges, NodeId, Point, Rect, Size};

/// The parameters of one walk over a paint list; nothing in [`PaintState`]
/// is changed for its duration (it replaces temporary scale and list swaps).
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct PaintPass {
    /// Device pixels per logical pixel the walk draws at (inside a
    /// transform, enlarged by its zoom).
    pub(crate) scale: f64,
    /// What the walk draws for.
    pub(crate) purpose: crate::lower::context::Purpose,
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

// xtask-allow(state-fields): Ink, look, scene and dropped-draw collections are independent derived caches and report deduplication (S-9); redraw has its own enum.
/// The paint group of the model: derived caches and whether a
/// frame must be redrawn. Only the paint modules read the caches.
pub(crate) struct PaintState {
    pub(crate) ink: Vec<Cell<Option<Ink>>>,
    pub(crate) look: Vec<Option<Look>>,
    pub(super) scale: f64,
    pub(crate) scene_cache: RefCell<crate::lower::SceneCache>,
    pub(super) dropped_reported: RefCell<BTreeSet<(NodeId, u8)>>,
    pub(super) dropped_draws: RefCell<Vec<(NodeId, u32)>>,
    pub(super) list: PaintList,
    pub(super) redraw: Redraw,
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
            list: PaintList::new(),
            redraw: Redraw::Needed,
        }
    }

    /// The window's scale factor the next frame is drawn at.
    pub(crate) fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
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

    /// A node's paint reads around its box by `reach` (backdrop filters).
    pub(crate) fn note_backdrop(&mut self, reach: f64) {
        self.list.note_backdrop(reach);
    }
}

// xtask-allow(state-fields): Normalized paint cache stores independent optional color and geometric drawing facts (S-9).
/// What a node's paint says about where it draws, sanitized like
/// `paint::sanitize_paint` (read in place: most nodes have no state
/// styles, and their paint is not copied).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Look {
    pub(super) opacity: f64,
    /// How far its outer shadows and focus ring reach outside its box.
    pub(super) overflow: f64,
    /// The widths of its visible border sides (canvas scenes start inside
    /// them).
    pub(super) border: Edges<f64>,
    /// How far its layer filters draw beyond what they filter.
    pub(super) fx: f64,
    /// Its effects need a group (clip path, blend), whatever its opacity.
    pub(super) grouped: bool,
    /// It clips its children to its padding box.
    pub(super) clip: bool,
    /// How many of a background, a border, shadows and a ring it draws.
    pub(super) items: u8,
    /// Its corners are rounded.
    pub(super) rounded: bool,
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

// xtask-allow(state-fields): Immutable paint-list record stores independent drawing and clipping facts, not a lifecycle.
/// How one listed node is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Step {
    pub(super) id: NodeId,
    /// Its untransformed layout box's origin, in the space of the
    /// transforms pushed around it (its own and its ancestors'
    /// translations below them applied).
    pub(super) origin: Point,
    /// The transform it is drawn through (an index into
    /// [`PaintList::transforms`]).
    pub(super) transform: Option<u32>,
    /// The alpha its colors are faded by (outside a group: its opacity
    /// times its ancestors').
    pub(super) alpha: f64,
    pub(super) clips: bool,
    pub(super) presence_clip: Option<Rect>,
    /// Its corners are rounded (a clip is a layer clipped to its rounded
    /// padding box, not a rect clip).
    pub(super) rounded: bool,
    /// It draws itself (else it is listed for its group only).
    pub(super) own: bool,
    /// It is composited in a group.
    pub(super) grouped: bool,
    /// Its opacity is the group's (applied by its group).
    pub(super) group: bool,
    /// The area its ancestors let it draw in (grown by the culling
    /// margin), in the space of its origin: a text input draws only the
    /// lines that reach it.
    pub(super) clip: Rect,
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
    /// How far the farthest-reaching backdrop filter seen reads around its
    /// box (the viewport grows by it).
    pub(super) backdrop_reach: f64,
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
