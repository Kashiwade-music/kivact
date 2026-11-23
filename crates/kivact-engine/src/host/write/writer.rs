//! Role: `Writer`: the turn's write API onto the engine's models.
//!
//! A write names the window it targets, or the engine derives the window
//! from the node it touches. The App's writes, event queue and revision are
//! shared; each window keeps its own model.

use super::reader::Reader;
#[cfg(feature = "devtools")]
use crate::contract::{DevtoolsKeys, DevtoolsOverlay};
use crate::contract::{HostEvent, HostRequest, NodeKind, SceneLayer};
use crate::model::AppModel;
use crate::mutation::WriteStats;
use kivact_types::{
    DrawList, NodeId, PlatformResult, Point, PointerId, RegionId, RequestId, Revision, Size,
    WindowId,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

// xtask-allow(state): independent: app (the window models) and engine (Arvane's text) are shared handles set by Writer::new and never reassigned; each write borrows them for its duration only
/// Owned, cloneable write access to the engine's models, given to kivact for
/// startup. Every write applies at once; [`Writer::commit`] ends a revision.
#[derive(Clone)]
pub struct Writer {
    pub(super) app: Rc<RefCell<AppModel>>,
    pub(super) engine: Rc<RefCell<arvane_text::TextEngine>>,
}

impl Writer {
    pub(crate) fn new(
        app: &Rc<RefCell<AppModel>>,
        engine: &Rc<RefCell<arvane_text::TextEngine>>,
    ) -> Self {
        Writer {
            app: Rc::clone(app),
            engine: Rc::clone(engine),
        }
    }

    /// Read access to the engine's tree, and its text measurement, for as
    /// long as kivact keeps it.
    pub fn reader(&self) -> Reader {
        Reader {
            app: Rc::clone(&self.app),
            engine: Rc::clone(&self.engine),
        }
    }

    /// Open a window for `request`; its model exists from this write on, so
    /// the same turn may create its nodes. The host opens the platform
    /// window after the turn (`WindowChange::Open`).
    pub fn open_window(&self, request: kivact_types::WindowRequest) {
        let viewport = request.size;
        self.app.borrow_mut().open_window(request, viewport);
    }

    /// Close window `id` after the turn (`WindowChange::Close`).
    pub fn close_window(&self, id: WindowId) {
        self.app.borrow_mut().close_window(id);
    }

    /// Create a detached node of `window` whose payload owns its
    /// kind-specific values.
    pub fn create(&self, window: WindowId, id: NodeId, kind: NodeKind) {
        let mut app = self.app.borrow_mut();
        let AppModel {
            windows, writes, ..
        } = &mut *app;
        let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) else {
            return;
        };
        debug_assert!(!model.tree.contains(id), "node ids are unique");
        model.create(id, kind);
        writes.create(id);
        model.log.tree_upserted(model.window, [id]);
    }

    /// Prepare one event immediately before dispatch, after earlier callbacks.
    /// Pointer activation consumes the gesture adoption into its routed sample.
    pub fn prepare_input(&self, event: &mut HostEvent) {
        if let HostEvent::Activate {
            node,
            region,
            pointer: Some(route),
        } = event
        {
            let adopted = self.app.borrow_mut().with_node(*node, |model, _| {
                model
                    .admission
                    .take_adoption(route.input.id, *node, *region)
            });
            route.activation = Some(crate::contract::Activation(adopted.unwrap_or(false)));
        }
    }

    /// Record that `pointer`'s current primary gesture was adopted at
    /// `region` of `node` (a canvas gesture took the press over): the
    /// activation its click produces, `HostEvent::Activate`, is then
    /// skipped through the release sample's routing metadata. The adoption ends at
    /// that pointer's next primary `Down` or its `Cancel` (as dispatched,
    /// see [`Reader::admit_input`]), and when its activation is answered.
    pub fn adopt_pointer(&self, node: NodeId, region: RegionId, pointer: PointerId) {
        self.adopt_pointer_target(node, Some(region), pointer);
    }

    /// Adopt a primary gesture on a node or one of its canvas regions.
    pub fn adopt_pointer_target(&self, node: NodeId, region: Option<RegionId>, pointer: PointerId) {
        self.app.borrow_mut().with_node(node, |model, _| {
            model.admission.adopt(pointer, node, region);
        });
    }

    /// End the adoption of `pointer` in every window (its gesture ended
    /// another way than a release or a `Cancel` of the engine's).
    pub fn forget_adopted_pointer(&self, pointer: PointerId) {
        let app = self.app.borrow();
        for model in app.windows.values().map(|w| &w.model) {
            model.admission.forget_adoption(pointer);
        }
    }

    /// Set or release capture for an admitted, enabled pointer listener.
    pub fn set_pointer_capture(&self, node: NodeId, pointer: PointerId, capture: bool) {
        self.app.borrow_mut().with_node(node, |model, _| {
            model.set_pointer_capture(node, pointer, capture);
        });
    }

    /// Keep an active touch contact routed to its initial control.
    pub fn reserve_touch_gesture(&self, pointer: PointerId) {
        let mut app = self.app.borrow_mut();
        for model in app.windows.values_mut().map(|w| &mut w.model) {
            model.reserve_touch_gesture(pointer);
        }
    }

    /// Totals of the client's mutations and the last committed revision.
    pub fn stats(&self) -> WriteStats {
        super::stats::stats_of(&self.app.borrow())
    }

    /// Operations awaiting a revision, grouped by changed node.
    pub fn pending_writes(&self) -> usize {
        self.app.borrow().writes().pending()
    }

    /// Attach or reparent node `id` under `parent`, before `before`
    /// (append if `None`). Both must live in the same window.
    pub fn insert(&self, parent: NodeId, id: NodeId, before: Option<NodeId>) {
        let mut app = self.app.borrow_mut();
        // A window whose open failed had its model removed before the core
        // saw `WindowFailed`: its nodes have no window any more.
        let (Some(window), Some(parent_window)) = (app.window_of(id), app.window_of(parent)) else {
            return;
        };
        assert_eq!(
            parent_window, window,
            "invariant: a node and its parent live in the same window"
        );
        let AppModel {
            windows, writes, ..
        } = &mut *app;
        let Some(m) = windows.get_mut(&window).map(|w| &mut w.model) else {
            return;
        };
        debug_assert!(m.tree.contains(parent) && m.tree.contains(id));
        debug_assert!(!m.tree.is_in_subtree(parent, id));
        debug_assert!(before.is_none_or(|before| {
            m.tree
                .get(before)
                .is_some_and(|node| m.tree.parent_id(node) == Some(parent))
        }));
        if before == Some(id) {
            return;
        }
        m.insert(parent, id, before, true);
        writes.structural();
        m.log.tree_upserted(m.window, [parent, id]);
    }

    /// Move the attached node `id` before a sibling (append if `None`).
    pub fn move_before(&self, id: NodeId, before: Option<NodeId>) {
        self.app.borrow_mut().with_node(id, |model, writes| {
            let parent = model.tree().get(id).and_then(|n| model.tree().parent_id(n));
            model.move_before(id, before);
            writes.structural();
            model
                .log
                .tree_upserted(model.window, parent.into_iter().chain([id]));
        });
    }

    /// Remove and destroy `id` and its whole subtree.
    pub fn remove(&self, id: NodeId) {
        self.remove_with_exit(id, false, false);
    }

    /// Remove a subtree and return its logical IDs in pre-order, including exits.
    /// Optionally free layout slots and report completion of physical removal.
    pub fn remove_with_exit(&self, id: NodeId, pop_layout: bool, notify: bool) -> Vec<NodeId> {
        self.app
            .borrow_mut()
            .with_node(id, |model, writes| {
                let parent = model.tree().get(id).and_then(|n| model.tree().parent_id(n));
                let removed = model.tree().subtree(id);
                model.remove(id, pop_layout, notify);
                writes.structural();
                model.log.tree_upserted(model.window, parent);
                removed
            })
            .unwrap_or_default()
    }

    /// Set the scene of one draw layer of the `Box` `id`.
    pub fn set_scene(&self, id: NodeId, layer: SceneLayer, scene: Arc<DrawList>) {
        self.app.borrow_mut().with_node(id, |model, writes| {
            if model
                .node(id)
                .and_then(|n| n.scene(layer))
                .is_some_and(|old| old.as_ref() == scene.as_ref())
            {
                return;
            }
            model.set_scene(id, layer, scene);
            writes.scene();
            model.log.tree_upserted(model.window, [id]);
        });
    }

    /// Focus `id`; the result is reported with `request`
    /// (`HostEvent::FocusResult`).
    pub fn focus(&self, id: kivact_types::FocusTarget, request: RequestId, visible: bool) {
        let window = self
            .app
            .borrow()
            .window_of(id.node)
            .unwrap_or(WindowId::MAIN);
        let mut app = self.app.borrow_mut();
        let AppModel {
            windows, writes, ..
        } = &mut *app;
        let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) else {
            return;
        };
        writes.command();
        model.focus_request(id, request, visible);
    }

    /// Take the keyboard focus of `window` away.
    pub fn blur(&self, window: WindowId) {
        let mut app = self.app.borrow_mut();
        let AppModel {
            windows, writes, ..
        } = &mut *app;
        if let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) {
            writes.command();
            model.set_focus(None);
        }
    }

    /// Scroll one container with optional smooth interpolation.
    pub fn scroll(
        &self,
        id: NodeId,
        point: Point,
        relative: bool,
        easing: Option<kivact_types::Easing>,
    ) {
        self.app.borrow_mut().with_node(id, |m, writes| {
            writes.command();
            let Some(n) = m.node(id) else { return };
            let start = m.tree.scroll_of(id);
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
            m.scroll_smoothly(id, start, target, easing);
        });
    }

    /// Select every character of a text input.
    pub fn select_all(&self, id: NodeId) {
        self.app.borrow_mut().with_node(id, |model, writes| {
            writes.command();
            model.edit_input(id, |e| {
                e.move_to(0, false);
                e.move_to(e.value().len(), true);
            });
        });
    }

    /// Scroll the container `id` to `offset`.
    pub fn scroll_to(&self, id: NodeId, offset: Point) {
        self.app.borrow_mut().with_node(id, |model, writes| {
            writes.command();
            model.scroll_to(id, offset);
        });
    }

    /// The default action of an unhandled wheel event: scroll the nearest
    /// scrollable container at or above `id` that can still move by
    /// `delta` (edges propagate further up). The engine never scrolls
    /// above a wheel-listening node on its own; kivact answers each
    /// `Wheel` no handler handled with this write.
    pub fn scroll_by(&self, id: NodeId, delta: Point) {
        self.app.borrow_mut().with_node(id, |model, writes| {
            writes.command();
            model.scroll_from(id, delta, None);
        });
    }

    /// Scroll every scrollable ancestor of `id` so that it is visible.
    pub fn scroll_into_view(&self, id: NodeId) {
        self.app.borrow_mut().with_node(id, |model, writes| {
            writes.command();
            model.reveal(id);
        });
    }

    /// Report `LayoutChanged` for `id` while `on`. Asking again for what
    /// is already so changes nothing and counts no write.
    pub fn observe_layout(&self, id: NodeId, on: bool) {
        self.app.borrow_mut().with_node(id, |model, writes| {
            if model.observe_layout(id, on) {
                writes.command();
            }
        });
    }

    /// Record what a drawable's export asks of the window
    /// (`HostRequest::DrawableExport`, `SetDrawableGeometry`). Nothing is
    /// reported back: the window's answers come as drawable events.
    pub fn drawable_request(&self, request: HostRequest) {
        let id = match &request {
            HostRequest::DrawableExport { id, .. }
            | HostRequest::SetDrawableGeometry { id, .. } => *id,
            _ => return,
        };
        let window = self.drawable_window(id);
        let mut app = self.app.borrow_mut();
        let AppModel {
            windows, writes, ..
        } = &mut *app;
        if let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) {
            writes.command();
            crate::model::drawables::apply_request(model, request);
        }
    }

    /// Register `font` (`HostRequest::RegisterFont`) at once, so the next
    /// layout measures with it.
    fn register_font(&self, app: &mut AppModel, request: RequestId, font: kivact_types::Asset) {
        let result = app
            .assets
            .borrow_mut()
            .register_font(&mut self.engine.borrow_mut(), &font);
        if matches!(result, PlatformResult::Fonts(_)) {
            let fonts = app.assets.borrow().fonts();
            for model in app.windows.values_mut().map(|w| &mut w.model) {
                model.fonts_added();
                model.update_env(|env, _| env.fonts = fonts);
            }
            app.font_generation += 1;
        }
        app.events.push(HostEvent::EffectResult { request, result });
    }

    /// Ask the platform for `effect`; the result is reported with
    /// `request` (`HostEvent::EffectResult`). A font is registered at
    /// once (the next layout measures with it); the other effects are
    /// carried out by the window after the turn.
    pub fn effect(&self, request: RequestId, effect: HostRequest) {
        let mut app = self.app.borrow_mut();
        match effect {
            HostRequest::Snapshot(node) => {
                let window = app.window_of(node).unwrap_or(WindowId::MAIN);
                let AppModel {
                    windows, writes, ..
                } = &mut *app;
                if let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) {
                    writes.command();
                    model.snapshots.push((request, node));
                }
            }
            HostRequest::RegisterFont(font) => self.register_font(&mut app, request, font),
            drawable @ (HostRequest::DrawableExport { .. }
            | HostRequest::SetDrawableGeometry { .. }) => apply_drawable_effect(&mut app, drawable),
            effect => queue_effect(&mut app, request, effect),
        }
    }

    #[doc(hidden)]
    #[cfg(feature = "devtools")]
    /// Set the DevTools overlay of `window`; the host applies it after the
    /// turn.
    pub fn set_devtools_overlay(&self, window: WindowId, overlay: DevtoolsOverlay) {
        self.app
            .borrow_mut()
            .push_devtools_change(crate::contract::DevtoolsChange::Overlay(window, overlay));
    }

    #[doc(hidden)]
    #[cfg(feature = "devtools")]
    /// Set the window in pick mode (`None` leaves it).
    pub fn set_pick(&self, pick: Option<WindowId>) {
        self.app
            .borrow_mut()
            .push_devtools_change(crate::contract::DevtoolsChange::Pick(pick));
    }

    #[doc(hidden)]
    #[cfg(feature = "devtools")]
    /// Set the keys that open the DevTools and those that open it in pick mode.
    pub fn set_devtools_keys(&self, keys: DevtoolsKeys) {
        self.app
            .borrow_mut()
            .push_devtools_change(crate::contract::DevtoolsChange::Keys(keys));
    }

    #[doc(hidden)]
    /// Turn the engine's sync and frame records on or off.
    pub fn set_engine_recording(&self, syncs: bool, frames: bool) {
        self.app.borrow().log.set(syncs, frames);
    }

    /// Set window `window`'s title, minimum size and resizable flag (after the turn).
    pub fn set_window(
        &self,
        window: WindowId,
        title: String,
        min_size: Option<Size>,
        resizable: bool,
    ) {
        let mut app = self.app.borrow_mut();
        let AppModel {
            windows, writes, ..
        } = &mut *app;
        if let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) {
            writes.command();
            model.window_request = Some((title, min_size, resizable));
        }
    }

    /// Import the textures an external render engine shares as the image
    /// `image` into window `window` (`Window::import_external_image`), after
    /// the turn; the answer is `HostEvent::ExternalImported`. The client names
    /// each image once.
    pub fn import_external(
        &self,
        image: kivact_types::ExternalImageId,
        window: WindowId,
        textures: kivact_types::SharedTextures,
    ) {
        let mut app = self.app.borrow_mut();
        let AppModel {
            windows,
            writes,
            external_windows,
            ..
        } = &mut *app;
        if let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) {
            external_windows.insert(image, window);
            writes.command();
            model
                .external_writes
                .push(crate::model::ExternalWrite::Import(image, textures));
        }
    }

    /// Drop the image `image` of its window (`Window::drop_external_image`),
    /// after the turn; nothing when it is not imported.
    pub fn drop_external(&self, image: kivact_types::ExternalImageId) {
        let mut app = self.app.borrow_mut();
        let Some(window) = app.external_windows.remove(&image) else {
            return;
        };
        let AppModel {
            windows, writes, ..
        } = &mut *app;
        if let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) {
            writes.command();
            model
                .external_writes
                .push(crate::model::ExternalWrite::Drop(image));
        }
    }

    /// Ask for a `FrameTick` at or after `at_ns` on the frame (motion) clock,
    /// which equals the host clock at time scale 1.
    pub fn request_frame(&self, at_ns: u64) {
        let mut app = self.app.borrow_mut();
        app.writes.command();
        app.client_frames.0.insert(at_ns);
    }

    /// The client dispatches `event` now (before its handlers run): a text
    /// edit stops holding back its node's input-value echoes.
    pub fn deliver(&self, event: &HostEvent) {
        self.app.borrow_mut().delivered(event);
    }

    /// Watch (or stop watching) the visibility facts of `id`: a commit that
    /// changes them queues `HostEvent::NodeFlagsChanged`.
    pub fn watch_flags(&self, id: NodeId, watch: bool) {
        let mut app = self.app.borrow_mut();
        if watch {
            let flags = app
                .window_of(id)
                .and_then(|window| app.model(window))
                .map(|model| model.node_flags(id))
                .unwrap_or_default();
            app.watched.insert(id, flags);
        } else {
            app.watched.remove(&id);
        }
    }

    /// End the writes of one revision and return it: every window prepares
    /// its commit, one revision is committed for the whole App, and every
    /// window syncs.
    pub fn commit(&self) -> Revision {
        let mut app = self.app.borrow_mut();
        for model in app.windows.values_mut().map(|w| &mut w.model) {
            model.prepare_commit();
        }
        let touched: usize = app
            .windows
            .values()
            .map(|w| w.model.changes.touched().count())
            .sum();
        let revision = Revision(app.events.applied().0 + 1);
        app.events.commit(revision);
        app.writes_mut().commit(revision, touched);
        for model in app.windows.values_mut().map(|w| &mut w.model) {
            model.sync();
        }
        app.report_flags();
        revision
    }

    /// The window whose model mounts drawable `id`, or the main window.
    fn drawable_window(&self, id: kivact_types::DrawableId) -> WindowId {
        drawable_window(&self.app.borrow(), id)
    }
}

/// Queue `effect` for the host after the turn; a window's request only
/// while the window is open. A request that answers (all but a GPU
/// demand and an unhandled key) is a command of the write.
fn queue_effect(app: &mut AppModel, request: RequestId, effect: HostRequest) {
    let window = match &effect {
        HostRequest::Platform { window, .. }
        | HostRequest::RequestWindowSize { window, .. }
        | HostRequest::Inject { window, .. }
        | HostRequest::DemandGpu { window }
        | HostRequest::UnhandledKey { window, .. } => Some(*window),
        _ => None,
    };
    if window.is_some_and(|w| !app.windows.contains_key(&w)) {
        return;
    }
    if !matches!(
        effect,
        HostRequest::DemandGpu { .. } | HostRequest::UnhandledKey { .. }
    ) {
        app.writes.command();
    }
    // The shell events of the window come from this injection until the shell reports it delivered.
    if let HostRequest::Inject { window, .. } = &effect {
        app.injecting.entry(*window).or_default().push_back(request);
    }
    app.effects.push((request, effect));
}

/// Apply a drawable export or geometry request for the drawable's window.
fn apply_drawable_effect(app: &mut AppModel, drawable: HostRequest) {
    let id = match &drawable {
        HostRequest::DrawableExport { id, .. } | HostRequest::SetDrawableGeometry { id, .. } => *id,
        _ => unreachable!("invariant: matching drawable request"),
    };
    let window = drawable_window(app, id);
    let AppModel {
        windows, writes, ..
    } = app;
    if let Some(model) = windows.get_mut(&window).map(|w| &mut w.model) {
        writes.command();
        crate::model::drawables::apply_request(model, drawable);
    }
}

/// The window whose model mounts drawable `id`, or the main window.
fn drawable_window(app: &AppModel, id: kivact_types::DrawableId) -> WindowId {
    app.windows
        .iter()
        .find(|(_, window)| window.model.drawables.get(id).is_some())
        .map(|(id, _)| *id)
        .unwrap_or(WindowId::MAIN)
}
