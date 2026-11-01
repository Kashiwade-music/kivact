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
//! - [`Model::relayout`] syncs first, lays out only when something asks for
//!   it, with the usual skip of unchanged content (`Measure::measures_same`),
//!   records the geometry it changed in `Changes`, runs the debug layout
//!   lint, then syncs again to dispatch that geometry.

use crate::model::Model;
use crate::model::changes::Dirty;
use arvane::asset::AssetStore;
use kivact_wire::{NodeId, NodeType, Size};

impl Model {
    /// The window was resized.
    pub(crate) fn set_viewport(&mut self, window: Size) {
        if self.viewport != window {
            self.viewport = window;
            if let Some(root) = self.tree.root() {
                self.changes.a11y_subtrees.insert(root);
            } else {
                self.a11y.invalidate();
            }
            self.changes.layout_changed = true;
            self.changes.needs_layout = true;
        }
    }

    /// The measured content of `id` changed outside its own props (an
    /// image finished loading, a registered font changed its text, a
    /// controlled value moved the caret): the next [`Model::layout`]
    /// measures it again.
    pub(crate) fn remeasure(&mut self, id: NodeId) {
        if self.tree.contains(id) {
            self.changes.mark(id, Dirty::MEASURE);
            self.changes.needs_layout = true;
        }
    }

    /// Whether the next [`Model::layout`] has work to do.
    pub(crate) fn needs_layout(&self) -> bool {
        self.changes.needs_layout
    }

    /// Load images with `store` from now on (the shell's waker exists only
    /// once the window does; nothing is loaded before).
    pub(crate) fn set_store(&mut self, store: AssetStore) {
        self.assets.get_mut().replace_store(store);
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
        self.tree.pack_child_orders();
        self.retarget_frame();
        self.finish_exits();
        self.tree.pack_child_orders();
        for id in self.changes.take(Dirty::LAYOUT_STYLE) {
            self.layout_columns
                .rebuild_style(&self.tree, &mut self.layout_state, id);
        }
        for id in self.changes.take(Dirty::LAYOUT_CHAIN) {
            self.layout_columns
                .invalidate_chain(&self.tree, &mut self.layout_state, id);
        }
        if std::mem::take(&mut self.changes.layout_changed) {
            self.layout_state.topology_changed();
        }
        let tree = &self.tree;
        self.paint.forget_missing(|id| tree.contains(id));
        self.assets.get_mut().prune_reports(|id| tree.contains(id));
        if self.changes.fonts {
            let texts: Vec<NodeId> = self
                .tree
                .iter()
                .filter(|(_, n)| {
                    (n.kind() == NodeType::Text
                        || self.editors[n.index.0]
                            .as_deref()
                            .map(|editor| editor.editor.state())
                            .is_some())
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
        let mut changes = std::mem::take(&mut self.changes);
        let redraw = changes.paint
            || changes.fonts
            || changes.moved
            || changes.any(Dirty::WRITTEN.union(Dirty::REDRAW));
        let restyled = changes
            .marked(Dirty::REDRAW)
            .any(|id| self.node(id).is_some_and(|n| n.states().is_some()));
        let restack = changes.restack || changes.moved || restyled;
        self.paint
            .outdate_paint(&self.tree, &changes, redraw, changes.fonts, restack);
        for id in changes.marked(Dirty::WRITTEN.union(Dirty::REDRAW)) {
            let flags = changes.dirty.get(&id).copied().unwrap_or_default();
            if flags.contains(Dirty::A11Y_LOCAL_TEXT) && !flags.contains(Dirty::A11Y_METADATA) {
                self.a11y.local_text(id);
            } else {
                self.a11y.node(id);
            }
            if self
                .node(id)
                .is_some_and(|n| n.states().is_some_and(crate::tree::states_place))
            {
                self.a11y.subtree(id);
            }
        }
        for id in changes
            .marked(Dirty::SCROLLED.union(Dirty::STUCK))
            .chain(changes.a11y_subtrees.iter().copied())
        {
            self.a11y.subtree(id);
        }
        if changes.a11y_structure || changes.a11y_focus || !changes.a11y_removed.is_empty() {
            self.a11y
                .request_publication(changes.a11y_removed.iter().copied());
        }
        for id in changes.take(Dirty::MEASURE) {
            self.changes.mark(id, Dirty::MEASURE);
        }
        self.changes.needs_layout = changes.needs_layout;
        self.report_sessions();
    }

    /// Keep the base paint metrics and backdrop reach derived from `id`'s props.
    fn derive(&mut self, id: NodeId) {
        let Some(n) = self.tree.get(id) else {
            return;
        };
        if let Some(p) = n.paint() {
            self.paint
                .note_backdrop(kivact_lower::effects::backdrop_reach(p));
        }
        let look = n.paint().map_or_else(
            || crate::paint::state::Look::of(&kivact_wire::PaintProps::default()),
            crate::paint::state::Look::of,
        );
        // The list is built again when what its steps are made of changed
        // (a node not derived before has no list step yet).
        let kept = self.paint.look[n.index.0].replace(look);
        if kept.is_none_or(|kept| kept.stacking() != look.stacking()) {
            self.changes.restack = true;
        }
    }

    /// Run pending layout work, then distribute its geometry changes.
    pub(crate) fn relayout(&mut self, engine: &mut arvane::Engine) {
        self.sync();
        if self.needs_layout() {
            #[cfg(debug_assertions)]
            let remeasured: Vec<NodeId> = self.changes.marked(Dirty::MEASURE).collect();
            self.layout(engine);
            let pending = self.assets.get_mut().take_pending();
            for e in pending {
                self.push(e);
            }
            #[cfg(debug_assertions)]
            self.lint_layout(engine, self.changes.laid_out, &remeasured);
            self.sync();
        }
    }

    /// Apply the asset store's finished work: measure again the images
    /// whose natural size changed and redraw (a raster at a new size may
    /// have landed), report failures. Laying out is left to the next
    /// [`Model::relayout`].
    pub(crate) fn poll_assets(&mut self) {
        let (changed, failed) = self.assets.get_mut().poll();
        if !changed && !failed {
            return;
        }
        let images: Vec<NodeId> = self.tree.images.iter().copied().collect();
        if changed {
            for id in &images {
                let node = self.tree.get(*id);
                let used = node.and_then(|n| self.measure.natural[n.index.0].get());
                let image = node.and_then(|n| n.image());
                if self.assets.get_mut().natural_changed(image, used) {
                    self.remeasure(*id);
                    self.changes
                        .mark(*id, Dirty::REDRAW.union(Dirty::A11Y_METADATA));
                }
            }
            self.changes.paint = true;
        }
        self.sync();
        if failed {
            self.report_images(&images);
        }
    }

    /// Report the failed images of the `Image` nodes among `nodes`, once
    /// per node and asset, in the order of `nodes`.
    fn report_images(&mut self, nodes: &[NodeId]) {
        let assets = self.assets.get_mut();
        let mut reports = Vec::new();
        for id in nodes {
            let Some(n) = self.tree.get(*id).filter(|n| n.kind() == NodeType::Image) else {
                continue;
            };
            let Some(asset) = n.image() else {
                continue;
            };
            reports.extend(assets.check_image(*id, asset));
        }
        for e in reports {
            self.push(e);
        }
    }
}
