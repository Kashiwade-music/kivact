//! Role: The paint group's retained paint list and derived node caches.
//!
//! The fields are the paint modules' alone; the model, the host and the
//! layout pass use the methods (`PaintState`).
//!
//! The caches have one role each: a node's normalized base paint (`Look`)
//! and its ink, indexed by structural slot beside the model's
//! placement (`Place`); none copies the
//! typed props into a general record of facts. While there is no surface
//! no redraw is asked for (`Host::paint_due`); `needs_redraw` stays set
//! and the restored surface draws the frame.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashMap};

use kivact_types::protocol::ColorMatrix;
use kivact_wire::{Edges, NodeId, Point, Rect, Size};

/// The paint group of the model: derived caches and whether a
/// frame must be redrawn. Only the paint modules read the caches.
pub(crate) struct PaintState {
    pub(crate) ink: Vec<Cell<Option<Ink>>>,
    pub(crate) look: Vec<Option<Look>>,
    pub(super) scale: f32,
    pub(super) raster: Cell<f32>,
    pub(crate) scene_cache: RefCell<kivact_lower::SceneCache>,
    pub(super) dropped_reported: RefCell<BTreeSet<(NodeId, u8)>>,
    pub(super) dropped_draws: RefCell<Vec<(NodeId, u32)>>,
    pub(super) list: PaintList,
    pub(super) needs_redraw: bool,
}

impl PaintState {
    /// Initialize an empty retained list that needs its first frame.
    pub(crate) fn new() -> Self {
        Self {
            ink: Vec::new(),
            look: Vec::new(),
            scale: 1.0,
            raster: Cell::new(1.0),
            scene_cache: RefCell::default(),
            dropped_reported: RefCell::default(),
            dropped_draws: RefCell::default(),
            list: PaintList::new(),
            needs_redraw: true,
        }
    }

    /// The window's scale factor the next frame is drawn at.
    pub(crate) fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Whether the next frame must be drawn.
    pub(crate) fn needs_redraw(&self) -> bool {
        self.needs_redraw
    }

    /// The frame was drawn.
    pub(crate) fn drawn(&mut self) {
        self.needs_redraw = false;
    }

    /// Forget what was kept for the nodes `live` says are gone.
    pub(crate) fn forget_missing(&mut self, live: impl Fn(NodeId) -> bool) {
        self.dropped_reported.get_mut().retain(|(id, _)| live(*id));
    }

    /// A node's paint reads around its box by `reach` (backdrop filters).
    pub(crate) fn note_backdrop(&mut self, reach: f32) {
        self.list.note_backdrop(reach);
    }
}

/// What a node's paint says about where it draws, sanitized like
/// `paint::sanitize_paint` (read in place: most nodes have no state
/// styles, and their paint is not copied).
#[derive(Clone, Copy, Debug)]
pub(crate) struct Look {
    pub(super) opacity: f32,
    /// How far its outer shadows and focus ring reach outside its box.
    pub(super) overflow: f32,
    /// The widths of its visible border sides (canvas scenes start inside
    /// them).
    pub(super) border: Edges<f32>,
    /// How far its layer filters draw beyond what they filter.
    pub(super) fx: f32,
    /// The layers its effects need (layer filters, clip path, mask,
    /// blend), whatever its opacity: 0 or at least 1.
    pub(super) layers: u8,
    /// Its color filters as one matrix (`None`: it has none).
    pub(super) recolor: Option<ColorMatrix>,
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
    pub(super) generation: u64,
}

/// How one listed node is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Step {
    pub(super) id: NodeId,
    /// Its untransformed layout box's origin, in the space of the
    /// transforms pushed around it (its own and its ancestors'
    /// translations below them applied).
    pub(super) origin: Point,
    /// The transform it is drawn through (an index into
    /// [`PaintList::transforms`], or `u32::MAX`).
    pub(super) transform: u32,
    /// The alpha its colors are faded by (outside a group: its opacity
    /// times its ancestors').
    pub(super) alpha: f32,
    pub(super) clips: bool,
    /// Its corners are rounded (a clip is a layer clipped to its rounded
    /// padding box, not a rect clip).
    pub(super) rounded: bool,
    /// It draws itself (else it is listed for its layers only).
    pub(super) own: bool,
    /// The layers it is composited in (0: none).
    pub(super) layers: u8,
    /// Its opacity is a group's (applied by its outer layer).
    pub(super) group: bool,
    /// Its colors' matrix and its parent's (indices into
    /// [`PaintList::matrices`], or `u32::MAX`).
    pub(super) matrix: u32,
    pub(super) parent_matrix: u32,
    /// The area its ancestors let it draw in (grown by the culling
    /// margin), in the space of its origin: a text input draws only the
    /// lines that reach it.
    pub(super) clip: Rect,
}

/// One step of the paint list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Entry {
    /// Open the node's layers, draw its box, then (inside its clip when it
    /// clips) its canvas scene and its content.
    Open(Step),
    /// After its descendants: its `DrawOver` scene, then pop its clip and
    /// its layers.
    Close(Step),
}

/// The paint list and what it was built for.
#[derive(Clone, Debug, Default)]
pub(crate) struct PaintList {
    pub(super) entries: Vec<Entry>,
    /// The color matrices of the listed nodes with color filters.
    pub(super) matrices: Vec<ColorMatrix>,
    /// The transforms of the listed transformed nodes, each in the space
    /// of the transforms around it.
    pub(super) transforms: Vec<kivact_types::protocol::Transform>,
    /// How far the farthest-reaching backdrop filter seen reads around its
    /// box (the viewport grows by it).
    pub(super) backdrop_reach: f32,
    /// Rebuild before the next frame.
    pub(super) stale: bool,
    /// The window size and raster scale it was culled for.
    pub(super) built_for: Option<(Size, f32)>,
    /// The child indices of the containers with many children.
    pub(super) indices: HashMap<NodeId, ChildIndex>,
    /// Layout passes so far (the indices' geometry is from one of them).
    pub(super) pass: u64,
    /// The generation of the nodes' inks: an ink of another is forgotten.
    pub(super) ink_gen: u64,
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
    pub(super) starts: Vec<f32>,
    pub(super) furthest: Vec<f32>,
    /// The entries of the sticky children.
    pub(super) sticky: Vec<usize>,
    /// The layout pass the entries' geometry is from.
    pub(super) pass: u64,
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
