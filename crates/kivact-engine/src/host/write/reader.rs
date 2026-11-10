//! Role: `Reader`: the turn's read API over the engine's trees and layout.

use crate::contract::{
    BoxModelRects, EngineRecords, HostEvent, ImeStateInfo, NodeProperties, SceneLayer,
};
use crate::model::AppModel;
use kivact_types::{
    ComputedTextStyle, DrawList, FocusTarget, LayoutCheck, LayoutFinding, LayoutProps, NodeId,
    PaintProps, Point, Rect, WindowId,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Read access to the engine's trees (kivact's view of the props and
/// scenes it wrote): each window's tree is the one store of its node props,
/// so kivact asks it for a prop's current value instead of keeping a copy.
/// Reading while a write is applied is a bug (a model is borrowed for each
/// write only, never across a call into kivact).
#[derive(Clone)]
pub struct Reader {
    pub(super) app: Rc<RefCell<AppModel>>,
    pub(super) engine: Rc<RefCell<arvane::Engine>>,
}

impl Reader {
    /// The open windows, ascending by id.
    pub fn windows(&self) -> Vec<WindowId> {
        self.app.borrow().windows()
    }

    /// The window whose model holds `id`.
    pub fn window_of(&self, id: NodeId) -> Option<WindowId> {
        self.app.borrow().window_of(id)
    }

    /// The root node of `window`.
    pub fn root(&self, window: WindowId) -> Option<NodeId> {
        self.app
            .borrow()
            .model(window)
            .and_then(|model| model.tree().root())
    }

    /// The window the OS focuses.
    pub fn focused_window(&self) -> Option<WindowId> {
        self.app.borrow().focused_window()
    }

    /// The current image asset result; absent and non-image nodes are loading.
    pub fn image_status(&self, id: NodeId) -> kivact_types::ImageStatus {
        let app = self.app.borrow();
        let Some(model) = app.model(required_window(&app, id)) else {
            return kivact_types::ImageStatus::Loading;
        };
        let Some(asset) = model.tree().get(id).and_then(|node| node.image()) else {
            return kivact_types::ImageStatus::Loading;
        };
        model.assets.borrow().image_status(asset)
    }

    /// The listener retaining this pointer, if its gesture holds capture.
    pub fn pointer_capture(&self, pointer: kivact_types::PointerId) -> Option<NodeId> {
        let app = self.app.borrow();
        app.windows
            .values()
            .find_map(|model| model.pointer_capture(pointer))
    }

    /// Client mutation totals and the last committed revision.
    pub fn stats(&self) -> crate::mutation::WriteStats {
        let app = self.app.borrow();
        let stats = app.writes.stats;
        #[cfg(feature = "testing")]
        let stats = crate::mutation::WriteStats {
            a11y_constructed: app
                .windows
                .values()
                .map(|model| model.a11y.constructed)
                .sum(),
            a11y_published: app.windows.values().map(|model| model.a11y.published).sum(),
            ..stats
        };
        stats
    }

    /// The current display lifetime phase; missing nodes have left.
    pub fn presence(&self, id: NodeId) -> kivact_types::Presence {
        self.with_model(id, |model| {
            let tree = model.tree();
            tree.get(id).map_or(kivact_types::Presence::Leaving, |_| {
                tree.presence_of(id).into()
            })
        })
        .unwrap_or(kivact_types::Presence::Leaving)
    }

    /// The logical parent, including transparent containers.
    pub fn parent(&self, id: NodeId) -> Option<NodeId> {
        self.with_model(id, |model| {
            model
                .tree()
                .get(id)
                .and_then(|node| model.tree().parent_id(node))
        })
        .flatten()
    }

    /// Logical children, excluding nodes already removed by the client.
    pub fn children(&self, id: NodeId) -> Vec<NodeId> {
        self.with_model(id, |model| model.tree().logical_children(id))
            .unwrap_or_default()
    }

    /// Clone the one canonical semantics value stored for this node.
    pub fn semantics(&self, id: NodeId) -> Option<kivact_types::Semantics<FocusTarget>> {
        self.with_model(id, |model| model.tree().get(id)?.semantics().cloned())
            .flatten()
    }

    /// Registered and system font family names, sorted and unique, preserving case.
    pub fn font_families(&self) -> Vec<String> {
        self.engine.borrow().font_families()
    }

    /// A copy of the editing state of the text input `id` (its text
    /// included), recording the changes made to it from here.
    pub fn edit_state(&self, id: NodeId) -> Option<kivact_editing::EditState> {
        self.with_model(id, |model| Some(model.editor_node(id)?.editor.snapshot()))
            .flatten()
    }

    /// The editing session of the text input `id`, without its text.
    pub fn edit_session(&self, id: NodeId) -> Option<kivact_editing::TextSession> {
        self.with_model(id, |model| model.edit_session(id))
            .flatten()
    }

    /// The geometry of the lines `lines` of the text input `id` (only
    /// those are shaped) and its caret.
    pub fn text_layout(
        &self,
        id: NodeId,
        lines: std::ops::Range<usize>,
    ) -> Option<kivact_editing::TextLayout> {
        let app = self.app.borrow();
        let model = app.model(required_window(&app, id))?;
        crate::model::interact::text_geometry::text_layout(
            model,
            id,
            lines,
            &mut self.engine.borrow_mut(),
        )
    }

    /// Editing geometry in the node's untransformed canvas coordinates.
    pub fn editing_canvas_layout(&self, id: NodeId) -> Option<kivact_editing::TextLayout> {
        let app = self.app.borrow();
        let model = app.model(required_window(&app, id))?;
        crate::model::interact::text_geometry::layout_snapshot(
            model,
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
        self.with_model(id, |model| {
            let n = model.node(id)?;
            (!n.password()).then(|| model.edit(id).map(|e| e.value().to_owned()))?
        })
        .flatten()
    }

    /// A window point in a node's untransformed local coordinates.
    pub fn local_point(&self, id: NodeId, point: Point) -> Point {
        self.with_model(id, |model| model.pointer_local(id, point))
            .unwrap_or_default()
    }

    /// Map node-local region bounds into logical window coordinates.
    pub fn region_bounds(
        &self,
        id: NodeId,
        rect: kivact_types::Rect,
    ) -> Option<kivact_types::Rect> {
        self.with_model(id, |model| {
            let bounds = model.rect(id)?;
            // Node-local bounds are relative to the padding box, where the scene is drawn.
            let paint = crate::model::interact::visual::paint(model, id);
            let canvas = crate::model::interact::visual::padding_box(bounds, &paint);
            let own = kivact_types::Rect {
                x: canvas.x + rect.x,
                y: canvas.y + rect.y,
                ..rect
            };
            Some(model.drawn_rect(id, own))
        })
        .flatten()
    }

    /// Whether the frontmost hit belongs to this subtree.
    pub fn hit_test(&self, id: NodeId, point: Point) -> bool {
        self.with_model(id, |m| {
            m.hit(point)
                .is_some_and(|hit| m.tree().is_in_subtree(hit, id))
        })
        .unwrap_or(false)
    }

    /// The topmost shown modal overlay of `window`, if one is shown: input,
    /// Tab order and focus are confined to its subtree.
    pub fn active_modal(&self, window: WindowId) -> Option<NodeId> {
        self.app
            .borrow()
            .model(window)
            .and_then(|model| model.modal())
    }

    /// Whether input may reach `id`: no modal overlay is shown, or `id` lies
    /// in the topmost one (overlays nested in it included).
    pub fn in_modal_scope(&self, id: NodeId) -> bool {
        self.with_model(id, |model| model.in_scope(id))
            .unwrap_or(true)
    }

    /// The topmost shown overlay of `window` that offers `Dismiss`, inside the
    /// active modal when one is shown: where an Escape no key handler took goes.
    pub fn dismiss_target(&self, window: WindowId) -> Option<NodeId> {
        self.app
            .borrow()
            .model(window)
            .and_then(|model| model.dismiss_target())
    }

    /// Whether `target` may take the focus now, with the props written so
    /// far: it exists and is shown, focusable by its own props and not
    /// disabled, its region (if any) exists and is focusable, and it lies
    /// in the modal that stays open. The one rule every focus restore is
    /// checked by before it is sent.
    pub fn can_take_focus(&self, target: FocusTarget) -> bool {
        self.with_model(target.node, |model| model.can_take_focus(target))
            .unwrap_or(false)
    }

    /// Eligible focus targets for an arrow key inside a roving group.
    pub fn roving_targets(&self, root: NodeId) -> Vec<FocusTarget> {
        self.with_model(root, |model| model.roving_targets(root))
            .unwrap_or_default()
    }

    /// Whether `event` may be dispatched while overlays are shown. Ask once
    /// per event, in event order, immediately before dispatching it (a
    /// modal an earlier event of the turn opened already applies): it also
    /// records the pointer drags it admits. An event not aimed at a node
    /// passes. The model is borrowed for the call only.
    pub fn admit_input(&self, event: &HostEvent) -> bool {
        let window = match event {
            HostEvent::Pointer(routed) => Some(routed.window),
            HostEvent::Wheel(routed) => Some(routed.window),
            HostEvent::File(routed) => Some(routed.window),
            HostEvent::Key(routed) => Some(routed.window),
            HostEvent::Activate { node, .. }
            | HostEvent::TextChanged { node, .. }
            | HostEvent::SelectionChanged { node, .. }
            | HostEvent::Ime { node, .. }
            | HostEvent::A11y { node, .. } => self.app.borrow().window_of(*node),
            _ => None,
        };
        let Some(window) = window else {
            return true;
        };
        self.app
            .borrow()
            .model(window)
            .is_none_or(|model| model.admit_input(event))
    }

    // xtask-allow(internal-contract): the core's DevTools reads the recorded syncs and frames
    #[doc(hidden)]
    /// Take the sync and frame records collected since the last call.
    pub fn take_engine_records(&self) -> EngineRecords {
        self.app.borrow().log.take()
    }

    // xtask-allow(internal-contract): the core's DevTools shows the box model of a node
    #[doc(hidden)]
    /// The margin, border, padding and content rectangles of `id` in window
    /// coordinates.
    pub fn box_model(&self, id: NodeId) -> Option<BoxModelRects> {
        let app = self.app.borrow();
        let model = app.model(app.window_of(id)?)?;
        let layout = model.computed_layout(id)?;
        let border = model.rect(id)?;
        let padding = Rect {
            x: border.x + layout.border.left,
            y: border.y + layout.border.top,
            width: border.width - layout.border.left - layout.border.right,
            height: border.height - layout.border.top - layout.border.bottom,
        };
        let content = Rect {
            x: border.x + layout.content_box.x,
            y: border.y + layout.content_box.y,
            width: layout.content_box.width,
            height: layout.content_box.height,
        };
        Some(BoxModelRects {
            margin: Rect {
                x: border.x - layout.margin.left,
                y: border.y - layout.margin.top,
                width: border.width + layout.margin.left + layout.margin.right,
                height: border.height + layout.margin.top + layout.margin.bottom,
            },
            border,
            padding,
            content,
        })
    }

    // xtask-allow(internal-contract): the core's DevTools reads a node's effective paint
    #[doc(hidden)]
    /// The paint of `id` with its state styles applied.
    pub fn effective_paint(&self, id: NodeId) -> PaintProps {
        self.with_model(id, |model| model.effective_paint(id))
            .unwrap_or_default()
    }

    // xtask-allow(internal-contract): the core's DevTools reads a node's computed text
    #[doc(hidden)]
    /// The computed text style of `id`.
    pub fn computed_text(&self, id: NodeId) -> ComputedTextStyle {
        self.with_model(id, |model| model.text_props(id).as_ref().clone())
            .unwrap_or_default()
    }

    // xtask-allow(internal-contract): the core's DevTools reads a node's layout props
    #[doc(hidden)]
    /// The layout props of `id`.
    pub fn layout_props(&self, id: NodeId) -> LayoutProps {
        self.with_model(id, |model| {
            model.node(id).and_then(|node| node.layout().cloned())
        })
        .flatten()
        .unwrap_or_default()
    }

    // xtask-allow(internal-contract): the core's DevTools picks a node under a point without the modal
    #[doc(hidden)]
    /// The node under `point` in `window`, ignoring modals.
    pub fn hit_any(&self, window: WindowId, point: Point) -> Option<NodeId> {
        self.app
            .borrow()
            .model(window)
            .and_then(|model| model.hit_any(point))
    }

    // xtask-allow(internal-contract): the core's DevTools reports the focused node of a window
    #[doc(hidden)]
    /// The focused target of `window`.
    pub fn focused(&self, window: WindowId) -> Option<FocusTarget> {
        self.app
            .borrow()
            .model(window)
            .and_then(|model| model.focused())
    }

    // xtask-allow(internal-contract): the core's DevTools reports the Tab order of a window
    #[doc(hidden)]
    /// The Tab order of `window`.
    pub fn tab_order(&self, window: WindowId) -> Vec<FocusTarget> {
        self.app
            .borrow()
            .model(window)
            .map(|model| model.tab_stops())
            .unwrap_or_default()
    }

    // xtask-allow(internal-contract): the core's DevTools reports hover and press of a node
    #[doc(hidden)]
    /// Whether `id` is hovered and whether it is pressed.
    pub fn pointer_state(&self, id: NodeId) -> (bool, bool) {
        self.with_model(id, |model| (model.is_hovered(id), model.is_pressed(id)))
            .unwrap_or_default()
    }

    // xtask-allow(internal-contract): the core's DevTools lists a window's layout findings
    #[doc(hidden)]
    /// The layout defects of the texts of `window`.
    pub fn layout_findings(&self, window: WindowId, check: &LayoutCheck) -> Vec<LayoutFinding> {
        let app = self.app.borrow();
        let Some(model) = app.model(window) else {
            return Vec::new();
        };
        let mut engine = self.engine.borrow_mut();
        crate::check::layout_findings(model, &mut engine, check, None, false)
    }

    // xtask-allow(internal-contract): the core's DevTools reports the IME state of a window
    #[doc(hidden)]
    /// The IME state of `window`.
    pub fn ime_state(&self, window: WindowId) -> ImeStateInfo {
        use crate::model::edit::state::ImeTarget;
        let app = self.app.borrow();
        let Some(model) = app.model(window) else {
            return ImeStateInfo::Inactive;
        };
        let purpose = model.ime_purpose();
        let (node, composition) = match model.edits.target {
            ImeTarget::None => return ImeStateInfo::Inactive,
            ImeTarget::Native(slot) => {
                let id = model.tree.nodes.id_at(slot.0);
                let Some(editor) = model.editors[slot.0.0].as_deref() else {
                    return ImeStateInfo::Focused {
                        node: id,
                        purpose,
                        cursor_area: None,
                        composition: None,
                    };
                };
                let state = editor.editor.state();
                let composition = state.marked().map(|range| state.value()[range].to_owned());
                (id, composition)
            }
            ImeTarget::Session(id) => (id, None),
        };
        let cursor_area = if matches!(model.edits.target, ImeTarget::Native(_)) {
            let mut engine = self.engine.borrow_mut();
            let caret = crate::model::interact::text_geometry::caret_rect(model, node, &mut engine);
            Some(model.drawn_rect(node, caret))
        } else {
            model.session_area()
        };
        ImeStateInfo::Focused {
            node,
            purpose,
            cursor_area,
            composition,
        }
    }

    /// Borrow a node's typed values for one read callback.
    pub fn with_node<R>(&self, id: NodeId, f: impl FnOnce(Option<&NodeProperties>) -> R) -> R {
        let app = self.app.borrow();
        let Some(window) = app.window_of(id) else {
            return f(None);
        };
        let Some(model) = app.model(window) else {
            return f(None);
        };
        f(model.node(id).map(|node| &node.properties))
    }

    /// Run `f` on the model that holds `id`.
    fn with_model<R>(&self, id: NodeId, f: impl FnOnce(&crate::model::Model) -> R) -> Option<R> {
        let app = self.app.borrow();
        let window = app.window_of(id)?;
        app.model(window).map(f)
    }

    /// CPU pixels of the current subtree; absent when the node has no layout.
    pub fn snapshot(
        &self,
        id: NodeId,
    ) -> Option<Result<crate::offscreen::OffscreenImage, crate::offscreen::OffscreenError>> {
        let window = self.window_of(id)?;
        let mut app = self.app.borrow_mut();
        app.model_mut(window)?
            .snapshot(id, &mut self.engine.borrow_mut())
    }

    /// The scene of node `id`'s draw `layer`, if it has one.
    pub fn scene(&self, id: NodeId, layer: SceneLayer) -> Option<Arc<DrawList>> {
        self.with_model(id, |model| {
            model.node(id).and_then(|node| node.scene(layer)).cloned()
        })
        .flatten()
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
    /// kivact's turn, never while the engine holds a model.
    pub fn text_measurer(&self, id: NodeId) -> crate::text_measure::TextMeasurer {
        let inherited = {
            let app = self.app.borrow();
            let model = app
                .model(required_window(&app, id))
                .expect("invariant: a drawn node belongs to an open window");
            kivact_types::ComputedTextStyle::clone(&model.text_props(id))
        };
        crate::text_measure::TextMeasurer::new(Rc::clone(&self.engine), inherited)
    }
}

fn required_window(app: &AppModel, node: NodeId) -> WindowId {
    app.window_of(node).unwrap_or(WindowId::MAIN)
}

impl std::fmt::Debug for Reader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reader").finish_non_exhaustive()
    }
}
