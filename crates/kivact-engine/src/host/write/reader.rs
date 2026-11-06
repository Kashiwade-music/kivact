//! Role: `Reader`: the turn's read API over the engine's tree and layout.

use crate::contract::{HostEvent, NodeProperties, SceneLayer};
use crate::model::Model;
use kivact_types::{DrawList, FocusTarget, NodeId, Point};
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
    /// The current image asset result; absent and non-image nodes are loading.
    pub fn image_status(&self, id: NodeId) -> kivact_types::ImageStatus {
        let model = self.model.borrow();
        let Some(asset) = model.tree().get(id).and_then(|node| node.image()) else {
            return kivact_types::ImageStatus::Loading;
        };
        model.assets.borrow().image_status(asset)
    }

    /// The listener retaining this pointer, if its gesture holds capture.
    pub fn pointer_capture(&self, pointer: kivact_types::PointerId) -> Option<NodeId> {
        self.model.borrow().pointer_capture(pointer)
    }

    /// Client mutation totals and the last committed revision.
    pub fn stats(&self) -> crate::mutation::WriteStats {
        let model = self.model.borrow();
        let stats = model.writes.stats;
        #[cfg(feature = "testing")]
        let stats = crate::mutation::WriteStats {
            a11y_constructed: model.a11y.constructed,
            a11y_published: model.a11y.published,
            ..stats
        };
        stats
    }

    /// The current display lifetime phase; missing nodes have left.
    pub fn presence(&self, id: NodeId) -> kivact_types::Presence {
        self.model
            .borrow()
            .tree()
            .get(id)
            .map_or(kivact_types::Presence::Leaving, |node| node.presence.into())
    }

    /// The logical parent, including transparent containers.
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        let model = self.model.borrow();
        model
            .tree()
            .get(id)
            .and_then(|node| model.tree().parent_id(node))
    }

    /// Logical children, excluding nodes already removed by the client.
    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.model.borrow().tree().logical_children(id)
    }

    /// Clone the one canonical semantics value stored for this node.
    pub fn semantics(&self, id: NodeId) -> Option<kivact_types::Semantics<FocusTarget>> {
        self.model.borrow().tree().get(id)?.semantics().cloned()
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
        let model = self.model.borrow();
        crate::model::interact::text_geometry::text_layout(
            &model,
            id,
            lines,
            &mut self.engine.borrow_mut(),
        )
    }

    /// Editing geometry in the node's untransformed canvas coordinates.
    pub fn editing_canvas_layout(&self, id: NodeId) -> Option<kivact_editing::TextLayout> {
        crate::model::interact::text_geometry::layout_snapshot(
            &self.model.borrow(),
            id,
            0..usize::MAX,
            &mut self.engine.borrow_mut(),
            true,
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
        self.model.borrow().to_local(id, point)
    }

    /// Map node-local region bounds into logical window coordinates.
    pub fn region_bounds(
        &self,
        id: NodeId,
        rect: kivact_types::Rect,
    ) -> Option<kivact_types::Rect> {
        let model = self.model.borrow();
        let bounds = model.rect(id)?;
        let own = kivact_types::Rect {
            x: bounds.x + rect.x,
            y: bounds.y + rect.y,
            ..rect
        };
        let transform = model.transform_between(id, None);
        Some(crate::model::transform::map_rect(transform.as_ref(), own))
    }

    /// Whether the frontmost hit belongs to this subtree.
    pub fn hit_test(&self, id: NodeId, point: Point) -> bool {
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

    /// Eligible focus targets for an arrow key inside a roving group.
    pub fn roving_targets(&self, root: NodeId) -> Vec<FocusTarget> {
        self.model.borrow().roving_targets(root)
    }

    /// Whether `event` may be dispatched while overlays are shown. Ask once
    /// per event, in event order, immediately before dispatching it (a
    /// modal an earlier event of the turn opened already applies): it also
    /// records the pointer drags it admits. An event not aimed at a node
    /// passes. The model is borrowed for the call only.
    pub fn admit_input(&self, event: &HostEvent) -> bool {
        self.model.borrow().admit_input(event)
    }

    /// Borrow a node's typed values for one read callback.
    pub fn with_node<R>(&self, id: NodeId, f: impl FnOnce(Option<&NodeProperties>) -> R) -> R {
        let model = self.model.borrow();
        f(model.node(id).map(|node| &node.properties))
    }

    /// CPU pixels of the current subtree; absent when the node has no layout.
    pub fn snapshot(
        &self,
        id: NodeId,
    ) -> Option<Result<crate::offscreen::OffscreenImage, crate::offscreen::OffscreenError>> {
        self.model
            .borrow_mut()
            .snapshot(id, &mut self.engine.borrow_mut())
    }

    /// The scene of node `id`'s draw `layer`, if it has one.
    pub fn scene(&self, id: NodeId, layer: SceneLayer) -> Option<Arc<DrawList>> {
        let model = self.model.borrow();
        model.node(id).and_then(|n| n.scene(layer)).cloned()
    }

    /// A text measurer independent of a mounted node, using caller-supplied
    /// inherited defaults. Each shaping borrow ends before returning its result.
    pub fn paragraph_measurer(
        &self,
        inherited: kivact_types::ComputedTextStyle,
    ) -> crate::text_measure::TextMeasurer {
        crate::text_measure::TextMeasurer::new(Rc::clone(&self.engine), inherited)
    }

    /// The measurer for a draw callback of node `id`'s scene: runs without
    /// a family or a line height take the node's inherited ones. Called in
    /// kivact's turn, never while the engine holds the model.
    pub fn text_measurer(&self, id: NodeId) -> crate::text_measure::TextMeasurer {
        let inherited = {
            let model = self.model.try_borrow().expect(
                "invariant: draw callbacks run in kivact's turn, never while the engine holds the model",
            );
            kivact_types::ComputedTextStyle::clone(&model.text_props(id))
        };
        crate::text_measure::TextMeasurer::new(Rc::clone(&self.engine), inherited)
    }
}

impl std::fmt::Debug for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reader").finish_non_exhaustive()
    }
}
