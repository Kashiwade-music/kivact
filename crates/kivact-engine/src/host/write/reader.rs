//! Role: `Reader`: the turn's read API over the engine's tree and layout.

use crate::model::Model;
use kivact_wire::{
    FocusTarget, HostEvent, NodeId, NodeProperties, Point, PointerId, RegionId, Scene, SceneLayer,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Read access to the engine's tree (kivact's view of the props and
/// scenes it wrote): the engine's tree is the one store of node props, so
/// kivact asks it for a prop's current value instead of keeping a copy.
/// Reading while a write is applied is a bug (the model is borrowed for
/// each write only, never across a call into kivact).
#[derive(Clone)]
pub struct Reader {
    pub(super) model: Rc<RefCell<Model>>,
    pub(super) engine: Rc<RefCell<arvane::Engine>>,
}

impl Reader {
    /// Repack at an external geometry read when no outer read callback is active.
    /// Nested reads from with_node already hold an immutable model borrow; every
    /// semantic order reader supports the active representation in that case.
    fn prepare_geometry_read(&self) {
        if let Ok(mut model) = self.model.try_borrow_mut() {
            model.tree.pack_child_orders();
        }
    }

    /// Registered and system font family names, sorted and unique, preserving case.
    pub fn font_families(&self) -> Vec<String> {
        self.engine.borrow().font_families()
    }

    /// A copy of the editing state of the text input `id` (its text
    /// included), recording the changes made to it from here.
    pub fn edit_state(&self, id: NodeId) -> Option<kivact_editing::EditState> {
        Some(self.model.borrow().editor_node(id)?.editor.snapshot())
    }

    /// The editing session of the text input `id`, without its text.
    pub fn edit_session(&self, id: NodeId) -> Option<kivact_editing::TextSession> {
        self.model.borrow().edit_session(id)
    }

    /// The geometry of the lines `lines` of the text input `id` (only
    /// those are shaped) and its caret.
    pub fn text_layout(
        &self,
        id: NodeId,
        lines: std::ops::Range<usize>,
    ) -> Option<kivact_editing::TextLayout> {
        self.prepare_geometry_read();
        let model = self.model.borrow();
        crate::model::interact::text_geometry::text_layout(
            &model,
            id,
            lines,
            &mut self.engine.borrow_mut(),
        )
    }

    /// The value of the text input `id` (`None`: it is none, or a
    /// password). A text input publishes no accessible value (only its text
    /// runs), so this is where its value is read.
    pub fn edit_value(&self, id: NodeId) -> Option<String> {
        let model = self.model.borrow();
        let n = model.node(id)?;
        (!n.password()).then(|| model.edit(id).map(|e| e.value().to_owned()))?
    }

    /// A window point in a node's untransformed local coordinates.
    pub fn local_point(&self, id: NodeId, point: Point) -> Point {
        self.prepare_geometry_read();
        self.model.borrow().to_local(id, point)
    }

    /// Map node-local region bounds into logical window coordinates.
    pub fn region_bounds(&self, id: NodeId, rect: kivact_wire::Rect) -> Option<kivact_wire::Rect> {
        self.prepare_geometry_read();
        let model = self.model.borrow();
        let bounds = model.rect(id)?;
        let own = kivact_wire::Rect {
            x: bounds.x + rect.x,
            y: bounds.y + rect.y,
            ..rect
        };
        let transform = model.transform_between(id, None);
        Some(crate::model::transform::map_rect(transform.as_ref(), own))
    }

    /// Whether the frontmost hit belongs to this subtree.
    pub fn hit_test(&self, id: NodeId, point: Point) -> bool {
        self.prepare_geometry_read();
        let m = self.model.borrow();
        m.hit(point)
            .is_some_and(|hit| m.tree().is_in_subtree(hit, id))
    }

    /// The topmost shown modal overlay, if one is shown: input, Tab order
    /// and focus are confined to its subtree.
    pub fn active_modal(&self) -> Option<NodeId> {
        self.model.borrow().modal()
    }

    /// Whether input may reach `id`: no modal overlay is shown, or `id` lies
    /// in the topmost one (overlays nested in it included).
    pub fn in_modal_scope(&self, id: NodeId) -> bool {
        self.model.borrow().in_scope(id)
    }

    /// The topmost shown overlay that offers `Dismiss`, inside the active
    /// modal when one is shown: where an Escape no key handler took goes.
    pub fn dismiss_target(&self) -> Option<NodeId> {
        self.model.borrow().dismiss_target()
    }

    /// Whether `target` may take the focus now, with the props written so
    /// far: it exists and is shown, focusable by its own props and not
    /// disabled, its region (if any) exists and is focusable, and it lies
    /// in the modal that stays open. The one rule every focus restore is
    /// checked by before it is sent.
    pub fn can_take_focus(&self, target: FocusTarget) -> bool {
        self.model.borrow().can_take_focus(target)
    }

    /// Whether `event` may be dispatched while overlays are shown. Ask once
    /// per event, in event order, immediately before dispatching it (a
    /// modal an earlier event of the turn opened already applies): it also
    /// records the pointer drags it admits. An event not aimed at a node
    /// passes. The model is borrowed for the call only.
    pub fn admit_input(&self, event: &HostEvent) -> bool {
        self.model.borrow().admit_input(event)
    }

    /// Whether the activation of `region` of `node` that a click of
    /// `pointer` produced must be skipped, because `pointer`'s gesture was
    /// adopted at that region (`Writer::adopt_pointer`). Answers once: it
    /// ends the pointer's adoption either way. An activation of the node's
    /// own action (no region) is never skipped.
    pub fn take_adopted_activation(
        &self,
        node: NodeId,
        region: Option<RegionId>,
        pointer: PointerId,
    ) -> bool {
        self.model
            .borrow()
            .admission
            .take_adoption(pointer, node, region)
    }

    /// Borrow a node's typed values for one read callback.
    pub fn with_node<R>(&self, id: NodeId, f: impl FnOnce(Option<&NodeProperties>) -> R) -> R {
        let model = self.model.borrow();
        f(model.node(id).map(|node| &node.properties))
    }

    /// The scene of node `id`'s draw `layer`, if it has one.
    pub fn scene(&self, id: NodeId, layer: SceneLayer) -> Option<Arc<Scene>> {
        let model = self.model.borrow();
        model.node(id).and_then(|n| n.scene(layer)).cloned()
    }

    /// A text measurer independent of a mounted node, using caller-supplied
    /// inherited defaults. Each shaping borrow ends before returning its result.
    pub fn paragraph_measurer(
        &self,
        inherited: kivact_wire::ComputedText,
    ) -> crate::text_measure::TextMeasurer {
        crate::text_measure::TextMeasurer::new(Rc::clone(&self.engine), inherited)
    }

    /// The measurer for a draw callback of node `id`'s scene: runs without
    /// a family or a line height take the node's inherited ones. `None`
    /// while the model is borrowed (the engine measures or lays out; never
    /// so in kivact's turn), when nothing can be measured.
    pub fn text_measurer(&self, id: NodeId) -> Option<crate::text_measure::TextMeasurer> {
        let inherited =
            kivact_wire::ComputedText::clone(&self.model.try_borrow().ok()?.text_props(id));
        Some(crate::text_measure::TextMeasurer::new(
            Rc::clone(&self.engine),
            inherited,
        ))
    }
}

impl std::fmt::Debug for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reader").finish_non_exhaustive()
    }
}
