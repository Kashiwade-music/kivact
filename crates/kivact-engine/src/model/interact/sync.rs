//! Role: Dispatches pending changes to derived work, once per commit or input.
//!
//! Model::sync is the only invalidation dispatcher. It retargets state
//! styles, rebuilds changed layout styles, derives touched paint and
//! assets, reports cached failures, and invalidates paint and accessibility.
//! Pending measurement and layout requests survive until the layout pass.
//! Font registration answers immediately; asset polling records changes
//! for sync. The private commit boundary delivers queued events after
//! layout and after releasing the model borrow.
//!
//! Writes, input, ticks, resizes and finished assets all record into the
//! one `Changes`, and all reach derived work through this one dispatch, in
//! this order: the state style transitions and the frame requested for
//! them, the leaving subtrees that stopped moving, the final layout styles
//! of the changed nodes and their dirty chains, registered fonts (every
//! text, text input and measured leaf is measured again and its input lines
//! split again), the paint metrics (`Look`) and image assets of the touched
//! nodes, the assets of the changed scene layers, the cached image
//! failures, then the redraw, restack and ink of the paint list, the
//! accessibility rebuild (`a11y::state`) and the editors' sessions to
//! report.
//!
//! Invariants:
//! - Facts are derived from the typed props where they are needed, never
//!   copied out of them.
//! - A change to one node walks no whole tree: only touched nodes are
//!   derived again, and only the changed layers of a scene are read.
//! - [`Model::settle`] syncs first, lays out only when something asks for
//!   it, with the usual skip of unchanged content (`Measure::measures_same`),
//!   records the geometry it changed in `Changes`, runs the debug layout
//!   lint, then syncs again to dispatch that geometry.

use crate::contract::{HostEvent, NodeType, SyncRecord};
use crate::lower::AssetEvent;
use crate::model::Model;
use crate::model::changes::{Dirty, Dispatched, Geometry};
use kivact_types::{NodeId, Size};

/// Whether settling performed pending layout and geometry work.
pub(crate) enum Settled {
    Kept,
    Updated,
}

impl Model {
    /// The window was resized.
    pub(crate) fn set_viewport(&mut self, window: Size) {
        if self.viewport != window {
            self.viewport = window;
            if let Some(root) = self.tree.root() {
                self.a11y.subtree(root);
            } else {
                self.a11y.invalidate();
            }
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::STYLE);
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
        }
    }

    /// The measured content of `id` changed outside its own props (an
    /// image finished loading, a registered font changed its text, a
    /// controlled value moved the caret): the next [`Model::layout`]
    /// measures it again.
    pub(crate) fn remeasure(&mut self, id: NodeId) {
        if self.tree.contains(id) {
            self.changes.mark(id, Dirty::MEASURE);
            self.changes
                .layout
                .insert(crate::model::changes::LayoutReasons::PASS);
        }
    }

    /// Whether the next [`Model::layout`] has work to do.
    pub(crate) fn needs_layout(&self) -> bool {
        self.changes
            .layout
            .contains(crate::model::changes::LayoutReasons::PASS)
    }

    pub(crate) fn invalidate_paint(&mut self) {
        self.changes.paint = true;
    }

    /// A font file was registered: every text, text input and measured
    /// leaf (which may measure text) is measured and drawn again (the
    /// engine discarded its shaping caches).
    pub(crate) fn fonts_added(&mut self) {
        self.changes.fonts = true;
    }

    /// Dispatch pending writes and input changes into derived work once.
    pub(crate) fn sync(&mut self) {
        let reasons = self.changes.layout.bits();
        self.retarget_frame();
        self.finish_entrances();
        self.finish_exits();
        self.tree.pack_child_orders();
        for id in self.changes.take(Dirty::LAYOUT_STYLE) {
            self.layout.rebuild_style(&self.tree, id);
        }
        for id in self.changes.take(Dirty::LAYOUT_CHAIN) {
            self.layout.invalidate_chain(&self.tree, id);
        }
        if self
            .changes
            .layout
            .take(crate::model::changes::LayoutReasons::VIEWPORT)
        {
            self.layout.invalidate_viewport();
        }
        if self
            .changes
            .layout
            .take(crate::model::changes::LayoutReasons::STYLE)
        {
            self.layout.topology_changed();
        }
        let tree = &self.tree;
        self.paint.forget_missing(|id| tree.contains(id));
        self.assets
            .borrow_mut()
            .prune_reports(|id| tree.contains(id));
        if self.changes.fonts {
            let texts: Vec<NodeId> = self
                .tree
                .iter()
                .filter(|(_, n)| {
                    (n.kind() == NodeType::Text || self.editors[n.index.0].is_some())
                        || n.measure().is_some()
                })
                .map(|(id, _)| id)
                .collect();
            for id in texts {
                // New fonts can change fallback: an input's lines are
                // shaped again.
                if let Some(slot) = self.tree.nodes.index_of(id)
                    && let Some(editor) = self.editors[slot.0].as_mut()
                {
                    editor.lines = Default::default();
                }
                self.a11y.node(id);
                self.remeasure(id);
            }
            self.changes.paint = true;
        }
        let mut touched: Vec<NodeId> = self.changes.touched().collect();
        for id in &touched {
            self.derive(*id);
        }
        touched.sort_unstable();
        self.report_images(&touched);
        let changes = self.changes.dispatch();
        self.record_sync(reasons, &changes);
        let moved = changes.geometry != Geometry::Kept;
        let redraw = changes.paint
            || changes.fonts
            || moved
            || changes.any(Dirty::WRITTEN.union(Dirty::REDRAW));
        let restyled: Vec<NodeId> = changes
            .marked(Dirty::REDRAW)
            .filter(|id| self.node(*id).is_some_and(|n| n.states().is_some()))
            .collect();
        for id in &restyled {
            self.stale_fact(*id);
            let children: Vec<NodeId> = self.tree.flat_children(*id).collect();
            for child in children {
                self.stale_fact(child);
            }
        }
        let restack = changes.restack || moved || !restyled.is_empty();
        self.paint
            .outdate_paint(&self.tree, &changes, redraw, changes.fonts, restack);
        self.report_sessions();
    }

    /// Keep one sync record for the DevTools when the sync dispatched work
    /// and the DevTools records syncs (nothing is collected otherwise).
    fn record_sync(&self, reasons: u8, changes: &Dispatched) {
        if !self.log.syncs_recording() {
            return;
        }
        let redrawn: Vec<NodeId> = changes.marked(Dirty::REDRAW).collect();
        if reasons == 0 && redrawn.is_empty() {
            return;
        }
        self.log.sync(|| SyncRecord {
            window: self.window,
            revision: self.events.applied(),
            reasons,
            measured: Vec::new(),
            laid_out: Vec::new(),
            redrawn,
        });
    }

    /// Keep the base paint metrics derived from `id`'s props.
    fn derive(&mut self, id: NodeId) {
        let Some(n) = self.tree.get(id) else {
            return;
        };
        let look = n.paint().map_or_else(
            || crate::paint::state::Look::of(&kivact_types::PaintProps::default()),
            crate::paint::state::Look::of,
        );
        // The list is built again when what its steps are made of changed
        // (a node not derived before has no list step yet).
        let kept = self.paint.look[n.index.0].replace(look);
        if kept.is_none_or(|kept| kept.stacking() != look.stacking()) {
            self.changes.restack = true;
        }
        if kept != Some(look) {
            self.stale_fact(id);
            let children: Vec<NodeId> = self.tree.flat_children(id).collect();
            for child in children {
                self.stale_fact(child);
            }
        }
    }

    /// Run pending layout work, then distribute its geometry changes.
    pub(crate) fn settle(&mut self, engine: &mut arvane::Engine) -> Settled {
        self.sync();
        if self.needs_layout() {
            #[cfg(debug_assertions)]
            let remeasured: Vec<NodeId> = self.changes.marked(Dirty::MEASURE).collect();
            self.layout(engine);
            let pending = self.assets.borrow_mut().take_pending();
            for e in pending {
                match e {
                    AssetEvent::AssetFailed { node, message } => {
                        self.push(HostEvent::AssetFailed { node, message });
                    }
                }
            }
            #[cfg(debug_assertions)]
            self.lint_layout(engine, self.changes.was_laid_out(), &remeasured);
            self.sync();
            // In-memory sources may finish decoding during layout without a
            // later store poll. Publish the same actual asset projection now.
            let images: Vec<NodeId> = self.tree.images.iter().copied().collect();
            self.report_images(&images);
            Settled::Updated
        } else {
            Settled::Kept
        }
    }

    /// Apply the asset store's finished work: measure again the images
    /// whose natural size changed and redraw (a raster at a new size may
    /// have landed), report failures. Laying out is left to the next
    /// [`Model::settle`].
    pub(crate) fn poll_assets(&mut self) {
        let (changed, failed) = self.assets.borrow_mut().poll();
        if !changed && !failed {
            return;
        }
        let images: Vec<NodeId> = self.tree.images.iter().copied().collect();
        if changed {
            for id in &images {
                let node = self.tree.get(*id);
                let used = node.and_then(|n| self.measure.natural[n.index.0].get());
                let image = node.and_then(|n| n.image());
                if self.assets.borrow_mut().natural_changed(image, used) {
                    self.remeasure(*id);
                    self.changes.mark(*id, Dirty::REDRAW);
                    self.a11y.node(*id);
                }
            }
            self.changes.paint = true;
        }
        self.sync();
        self.report_images(&images);
    }

    /// Report the loading status of the `Image` nodes among `nodes` when it
    /// changed, and their failed images once per node and asset, in the
    /// order of `nodes`.
    fn report_images(&mut self, nodes: &[NodeId]) {
        let mut assets = self.assets.borrow_mut();
        let mut reports = Vec::new();
        let mut statuses = Vec::new();
        for id in nodes {
            let Some(n) = self
                .tree
                .get_mut(*id)
                .filter(|n| n.kind() == NodeType::Image)
            else {
                continue;
            };
            let Some(asset) = n.image() else {
                continue;
            };
            let status = assets.image_status(asset);
            reports.extend(assets.check_image(*id, asset));
            if n.sent_image.update(status).is_some() {
                statuses.push((*id, status));
            }
        }
        drop(assets);
        for (node, status) in statuses {
            self.push(HostEvent::ImageStatusChanged { node, status });
        }
        for e in reports {
            match e {
                AssetEvent::AssetFailed { node, message } => {
                    self.push(HostEvent::AssetFailed { node, message });
                }
            }
        }
    }
}
