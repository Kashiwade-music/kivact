//! Role: Write and read API through which kivact changes the engine's tree in a turn.
//!
//! The engine's write API: how kivact changes the engine's tree during a
//! turn ([`Writer`]), and reads it back ([`Reader`]).
//!
//! The host gives kivact owned cloneable write access at startup. Each
//! mutation borrows the shared model for that operation only. Commit checks
//! focus once, records a private event boundary, and synchronizes changes.
//! The host releases all borrows and completes layout before notifying core.
//! Platform effects, window settings and the imports and drops of shared
//! textures (feature `gpu-interop`) run after the turn; fonts register
//! immediately. Trusted structural operations retain local debug assertions.
//!
//! [`Writer`] and [`Reader`] are concrete types owning the same model and
//! text engine through `Rc`, borrowing them for each call only; there is no
//! backend trait per write or deferred model mutation. The active sibling order
//! changes immediately and is packed at revision and geometry-read boundaries.
//! The effects kivact runs
//! write through them directly (create, the typed setters, insert,
//! move_before, remove, set_scene, focus, blur, scroll, effect, set_window,
//! request_frame, observe_layout).
//!
//! Invariants:
//! - The engine trusts kivact's writes: payloads and setters are typed by
//!   node kind, and only local invariants are checked, by `debug_assert`
//!   (a unique nonzero node id, no cycle, an insert of a detached node, a
//!   `before` among the parent's children). There is no second tree to
//!   validate against and no report of an invalid write; the layout lint
//!   is a feature of its own (`check`).
//! - A commit checks the focus, moves the revision on, records the private
//!   boundary of the events (`Model::commit_revision`), then synchronizes
//!   (`Model::sync`).
//! - [`Reader`] lends the typed properties and payload of a node to a
//!   closure ([`Reader::with_node`]) and reads scenes, geometry and input
//!   values: kivact never copies a prop. [`Reader::text_measurer`] gives a
//!   draw callback's `Draw::measure_text` the text engine and the node's
//!   inherited text style (the same measurer type image output measures
//!   with), kept for as long as kivact holds it; the `LeafTextMeasurer` a
//!   measure function is lent lives during one layout pass only.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use crate::model::Model;
use crate::mutation::WriteStats;
use kivact_wire::{
    FocusTarget, HostEvent, NodeId, NodeKind, NodeProperties, PlatformRequest, PlatformResult,
    Point, PointerId, RegionId, RequestId, Revision, Scene, SceneLayer, Size,
};

/// Owned, cloneable write access to the engine's tree, given to kivact for
/// startup. Every write applies at once; [`Writer::commit`] ends a revision.
#[derive(Clone)]
pub struct Writer {
    model: Rc<RefCell<Model>>,
    engine: Rc<RefCell<arvane::Engine>>,
}

impl Writer {
    pub(crate) fn new(model: &Rc<RefCell<Model>>, engine: &Rc<RefCell<arvane::Engine>>) -> Self {
        Writer {
            model: Rc::clone(model),
            engine: Rc::clone(engine),
        }
    }

    /// Read access to the engine's tree, and its text measurement, for as
    /// long as kivact keeps it.
    pub fn reader(&self) -> Reader {
        Reader {
            model: Rc::clone(&self.model),
            engine: Rc::clone(&self.engine),
        }
    }

    /// Create a detached node whose payload owns its kind-specific values.
    pub fn create(&self, id: NodeId, kind: NodeKind) {
        let mut model = self.model.borrow_mut();
        debug_assert!(!model.tree.contains(id), "node ids are unique");
        model.create(id, kind);
        model.writes.create(id);
    }

    /// Record that `pointer`'s current primary gesture was adopted at
    /// `region` of `node` (a canvas gesture took the press over): the
    /// activation its click produces, `HostEvent::Activate`, is then
    /// skipped (`Reader::take_adopted_activation`). The adoption ends at
    /// that pointer's next primary `Down` or its `Cancel` (as dispatched,
    /// see [`Reader::admit_input`]), and when its activation is answered.
    pub fn adopt_pointer(&self, node: NodeId, region: RegionId, pointer: PointerId) {
        self.model.borrow().admission.adopt(pointer, node, region);
    }

    /// End the adoption of `pointer` (its gesture ended another way than a
    /// release or a `Cancel` of the engine's).
    pub fn forget_adopted_pointer(&self, pointer: PointerId) {
        self.model.borrow().admission.forget_adoption(pointer);
    }

    /// Totals of the client's mutations and the last committed revision.
    pub fn stats(&self) -> WriteStats {
        let model = self.model.borrow();
        let stats = model.writes.stats;
        #[cfg(feature = "testing")]
        let stats = WriteStats {
            a11y_constructed: model.a11y.constructed,
            a11y_published: model.a11y.published,
            ..stats
        };
        stats
    }
    /// Operations awaiting a revision, grouped by changed node.
    pub fn pending_writes(&self) -> usize {
        self.model.borrow().writes.pending()
    }

    /// Attach the detached node `id` under `parent`, before `before`
    /// (append if `None`).
    pub fn insert(&self, parent: NodeId, id: NodeId, before: Option<NodeId>) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        debug_assert!(m.tree.contains(parent) && m.tree.contains(id));
        debug_assert!(
            m.tree
                .get(id)
                .is_some_and(|node| m.tree.parent_id(node).is_none())
        );
        debug_assert!(!m.tree.is_in_subtree(parent, id));
        debug_assert!(before.is_none_or(|before| {
            m.tree
                .get(before)
                .is_some_and(|node| m.tree.parent_id(node) == Some(parent))
        }));
        m.insert(parent, id, before, true);
        m.writes.structural(id);
    }

    /// Move the attached node `id` before a sibling (append if `None`).
    pub fn move_before(&self, id: NodeId, before: Option<NodeId>) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.move_before(id, before);
        m.writes.structural(id);
    }

    /// Remove and destroy `id` and its whole subtree.
    pub fn remove(&self, id: NodeId) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.remove(id);
        m.writes.structural(id);
    }

    /// Set the scene of one draw layer of the `Box` `id`.
    pub fn set_scene(&self, id: NodeId, layer: SceneLayer, scene: Arc<Scene>) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;

        if m.node(id)
            .and_then(|n| n.scene(layer))
            .is_some_and(|old| old.as_ref() == scene.as_ref())
        {
            return;
        }
        m.set_scene(id, layer, scene);
        m.writes.scene(id);
    }

    /// Focus `id`; the result is reported with `request`
    /// (`HostEvent::FocusResult`).
    pub fn focus(&self, id: kivact_wire::FocusTarget, request: RequestId, visible: bool) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(Some(id.node));
        m.focus_request(id, request, visible);
    }

    /// Take the keyboard focus away.
    pub fn blur(&self) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(None);
        m.set_focus(None);
    }

    /// Scroll one container with optional smooth interpolation.
    pub fn scroll(&self, id: NodeId, point: Point, relative: bool, smooth: bool) {
        let mut m = self.model.borrow_mut();
        m.writes.command(Some(id));
        let Some(n) = m.node(id) else { return };
        let start = n.scroll();
        let target = if relative {
            Point {
                x: start.x + point.x,
                y: start.y + point.y,
            }
        } else {
            point
        };
        if !target.x.is_finite() || !target.y.is_finite() {
            return;
        }
        let axes = n.scroll_axes();
        let Some(layout) = m.computed_layout(id) else {
            return;
        };
        let target = Point {
            x: if axes.0 {
                target.x.clamp(
                    0.0,
                    (layout.content_size.width - layout.size.width).max(0.0),
                )
            } else {
                0.0
            },
            y: if axes.1 {
                target.y.clamp(
                    0.0,
                    (layout.content_size.height - layout.size.height).max(0.0),
                )
            } else {
                0.0
            },
        };
        m.scroll_smoothly(id, start, target, smooth);
    }
    /// Select every character of a text input.
    pub fn select_all(&self, id: NodeId) {
        let mut model = self.model.borrow_mut();
        model.writes.command(Some(id));
        model.edit_input(id, |e| {
            e.move_to(0, false);
            e.move_to(e.value().len(), true);
            false
        });
    }
    /// Scroll the container `id` to `offset`.
    pub fn scroll_to(&self, id: NodeId, offset: Point) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(Some(id));
        m.scroll_to(id, offset);
    }

    /// The default action of an unhandled wheel event: scroll the nearest
    /// scrollable container at or above `id` that can still move by
    /// `delta` (edges propagate further up). The engine never scrolls
    /// above a wheel-listening node on its own; kivact answers each
    /// `Wheel` no handler handled with this write.
    pub fn scroll_by(&self, id: NodeId, delta: Point) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(Some(id));
        m.scroll_from(id, delta, None);
    }

    /// Scroll every scrollable ancestor of `id` so that it is visible.
    pub fn scroll_into_view(&self, id: NodeId) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(Some(id));
        m.reveal(id);
    }

    /// Report `LayoutChanged` for `id` while `on`. Asking again for what
    /// is already so changes nothing and counts no write.
    pub fn observe_layout(&self, id: NodeId, on: bool) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        if m.observe_layout(id, on) {
            m.writes.command(Some(id));
        }
    }

    /// Ask the platform for `effect`; the result is reported with
    /// `request` (`HostEvent::EffectResult`). A font is registered at
    /// once (the next layout measures with it); the other effects are
    /// carried out by the window after the turn.
    pub fn effect(&self, request: RequestId, effect: PlatformRequest) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(None);
        match effect {
            PlatformRequest::RegisterFont(font) => {
                let result = m
                    .assets
                    .borrow_mut()
                    .register_font(&mut self.engine.borrow_mut(), &font);
                m.fonts_pending |= matches!(result, PlatformResult::Fonts(_));
                m.push(HostEvent::EffectResult { request, result });
            }
            effect => m.effects.push((request, effect)),
        }
    }

    /// Set the window's title and minimum size (after the turn).
    pub fn set_window(&self, title: String, min_size: Option<Size>) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(None);
        m.window_request = Some((title, min_size));
    }

    /// Import the textures an external render engine shares as the image
    /// `image` of the window (`Window::import_external_image`), after the
    /// turn; the answer is `HostEvent::ExternalImported`. The client names
    /// each image once.
    #[cfg(feature = "gpu-interop")]
    pub fn import_external(
        &self,
        image: kivact_wire::ExternalImageId,
        textures: kivact_wire::SharedTextures,
    ) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(None);
        m.external_writes
            .push(crate::model::ExternalWrite::Import(image, textures));
    }

    /// Drop the image `image` of the window
    /// (`Window::drop_external_image`), after the turn; nothing when it is
    /// not imported.
    #[cfg(feature = "gpu-interop")]
    pub fn drop_external(&self, image: kivact_wire::ExternalImageId) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(None);
        m.external_writes
            .push(crate::model::ExternalWrite::Drop(image));
    }

    /// Ask for a `FrameTick` at or after `at_ns` (host clock).
    pub fn request_frame(&self, at_ns: u64) {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        m.writes.command(None);
        m.frame.request(at_ns);
    }

    /// End the writes of one revision and return it: the focus is checked
    /// once and its private post-layout boundary recorded.
    pub fn commit(&self) -> Revision {
        let mut guard = self.model.borrow_mut();
        let m = &mut *guard;
        let revision = m.commit_revision(&mut self.engine.borrow_mut());
        m.writes.commit(revision);
        if std::mem::take(&mut m.fonts_pending) {
            m.fonts_added();
        }
        m.sync();
        revision
    }
}

/// Read access to the engine's tree (kivact's view of the props and
/// scenes it wrote): the engine's tree is the one store of node props, so
/// kivact asks it for a prop's current value instead of keeping a copy.
/// Reading while a write is applied is a bug (the model is borrowed for
/// each write only, never across a call into kivact).
#[derive(Clone)]
pub struct Reader {
    model: Rc<RefCell<Model>>,
    engine: Rc<RefCell<arvane::Engine>>,
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
        let mut state = kivact_editing::Editor::new(self.model.borrow().edit(id).cloned()?);
        state.start_log();
        Some(state.into_state())
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

pub(crate) mod typed;
