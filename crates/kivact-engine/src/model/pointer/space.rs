//! Role: Where one event's pointer is relative to nodes, regions and text.

use crate::model::Model;
use kivact_types::{NodeId, Point, RegionId};
use std::cell::RefCell;

/// Where the pointer of one input event is, relative to the nodes: a
/// window position, and the engine that finds the caret under it.
/// One concrete type, as `measure::leaf::Measure` is: window positions,
/// scene regions and shaped text offsets need no trait.
pub(crate) struct PointerSpace<'a> {
    at: Point,
    engine: RefCell<&'a mut arvane::Engine>,
}

impl<'a> PointerSpace<'a> {
    /// The pointer at the window position `at`.
    pub(crate) fn new(at: Point, engine: &'a mut arvane::Engine) -> Self {
        Self {
            at,
            engine: RefCell::new(engine),
        }
    }

    /// The pointer position in `node`'s local coordinates: relative to its
    /// untransformed border box ([`Model::rect`]), mapped back through
    /// its transforms (`Model::to_local`).
    pub(crate) fn local(&self, model: &Model, node: NodeId) -> Point {
        model.pointer_local(node, self.at)
    }

    /// The pointer position in the window.
    pub(crate) fn window(&self) -> Point {
        self.at
    }

    /// The scene region of `node` under the pointer: the topmost region of
    /// its scenes containing `local`.
    pub(crate) fn region(&self, model: &Model, node: NodeId, local: Point) -> Option<RegionId> {
        model.node(node)?.region_at(local)
    }

    /// The byte offset in the value of the text input `node` under the
    /// pointer (Arvane's shaping, `text_geometry`; `None`: the caret
    /// stays).
    pub(crate) fn text_offset(&self, model: &Model, node: NodeId) -> Option<usize> {
        let mut engine = self.engine.borrow_mut();
        crate::model::interact::text_geometry::offset_at(model, node, self.at, &mut engine)
    }
}
