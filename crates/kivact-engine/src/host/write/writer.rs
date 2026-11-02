//! Role: `Writer`: the turn's write API onto the engine's tree.

use super::reader::Reader;
use crate::model::Model;
use crate::mutation::WriteStats;
use kivact_wire::{
    HostEvent, NodeId, NodeKind, PlatformRequest, PlatformResult, Point, PointerId, RegionId,
    RequestId, Revision, Scene, SceneLayer, Size,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// Owned, cloneable write access to the engine's tree, given to kivact for
/// startup. Every write applies at once; [`Writer::commit`] ends a revision.
#[derive(Clone)]
pub struct Writer {
    pub(super) model: Rc<RefCell<Model>>,
    pub(super) engine: Rc<RefCell<arvane::Engine>>,
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
                if matches!(result, PlatformResult::Fonts(_)) {
                    m.fonts_added();
                }
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
        let revision = m.commit_revision();
        m.writes.commit(revision);
        m.sync();
        revision
    }
}
