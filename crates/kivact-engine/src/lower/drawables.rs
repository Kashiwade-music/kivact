//! Role: The canvas drawables of a frame and their hosts' drawings, as data for lowering.
//!
//! The model builds a [`DrawablePlan`] for each frame, before the hosts are
//! lowered; the lowering reads it through a [`DrawableSite`] and never sees
//! the model.
//!
//! Invariants:
//! - A drawable is placed by its first drawing in its host (slot 0 before
//!   slot 1, then command order). Its composed layers are taken once,
//!   by that drawing.
//! - A drawing in a node other than the drawable's host is not placed and
//!   is recorded in `misplaced`.

use std::cell::RefCell;
use std::collections::HashMap;

use kivact_types::render::CompositeLayer;
use kivact_types::{DrawableId, NodeId};

/// The canvas drawables of one frame.
#[derive(Default)]
pub(crate) struct DrawablePlan {
    /// The host node of each mounted canvas drawable.
    pub(crate) hosts: HashMap<DrawableId, NodeId>,
    /// The slot and command index of each drawable's placing drawing.
    pub(crate) winners: HashMap<DrawableId, (u8, u32)>,
    /// The composed layers of each placed drawable, taken by its drawing.
    pub(crate) layers: HashMap<DrawableId, Vec<CompositeLayer>>,
    /// The drawings of drawables in nodes that do not host them, not reported yet.
    pub(crate) misplaced: Vec<(NodeId, DrawableId)>,
    /// The second drawings of a drawable its host already draws in the other
    /// slot (the first drawing places it), not reported yet.
    pub(crate) twice: Vec<(NodeId, DrawableId)>,
}

/// Where a host scene's commands are lowered: the frame's plan, the node
/// that owns the scene and its slot.
#[derive(Clone, Copy)]
pub(crate) struct DrawableSite<'a> {
    pub(crate) plan: &'a RefCell<DrawablePlan>,
    pub(crate) host: NodeId,
    pub(crate) slot: u8,
}
