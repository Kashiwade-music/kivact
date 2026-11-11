//! Role: The Model aggregate: every passive state group, its creation and reads.

use crate::a11y::state::A11yState;
use crate::contract::{EngineEmulation, HostEvent};
use crate::model::events::EngineLog;
use crate::paint::state::PaintState;
use crate::tree::NodeTree;
use crate::tree::node::HostNode;
use kivact_types::{Modifiers, NodeId, PlatformRequest, RequestId, Size, WindowId};
use std::cell::RefCell;
use std::rc::Rc;

/// Requests that must wait until the current client turn has finished.
pub(crate) enum DeferredRequest {
    Platform(PlatformRequest),
    ReducedMotion(Option<bool>),
    Inject(Vec<arvane::InjectedInput>),
    Emulate(EngineEmulation),
    TimeScale(f64),
    /// Start the model's window GPU device (nothing is answered).
    DemandGpu,
}

/// The engine's state: the node tree and one field per state group.
pub(crate) struct Model {
    /// This model's window.
    pub(crate) window: WindowId,
    pub(crate) tree: NodeTree,
    pub(crate) layout: kivact_layout::Layout,
    pub(crate) layout_motion: super::layout_motion::state::LayoutMotion,
    pub(crate) playback: super::playback::state::Playback,
    pub(crate) sources: super::frame::sources::MotionSources,
    pub(crate) measure: crate::measure::state::MeasureColumns,
    /// The recent values of the shared node properties (`Model::set_shared_prop`).
    pub(crate) recent: super::recent::RecentProps,
    pub(crate) editors: Vec<Option<Box<crate::measure::editor::EditorNode>>>,
    pub(crate) events: super::events::EventSink,
    pub(crate) changes: super::changes::Changes,
    pub(crate) viewport: Size,
    pub(crate) focus: super::focus::state::Focus,
    pub(crate) pointer: super::pointer::state::Pointer,
    /// The drags admitted before a modal opened (`modal`).
    pub(crate) admission: super::modal::state::Admission,
    // xtask-allow(state-fields): projection: the modifiers last reported to the client, written only by Model::report_modifiers
    /// The keyboard modifiers the shell reports held.
    pub(crate) modifiers: kivact_types::cell::Sent<Modifiers>,
    pub(crate) key_down_targets: super::focus::state::KeyDownTargets,
    pub(crate) edits: super::edit::state::Edits,
    pub(crate) motion: super::state_motion::state::StateMotion,
    pub(crate) presence: super::presence::state::Presences,
    pub(crate) assets: Rc<RefCell<crate::lower::assets::ArvaneAssets>>,
    /// What the DevTools records, shared by every model and the host.
    pub(crate) log: EngineLog,
    pub(crate) paint: PaintState,
    pub(crate) a11y: A11yState,
    pub(crate) frame: super::frame::state::Frame,
    // xtask-allow(state-fields): independent: the title and minimum size Writer::set_window queues and finish_window takes after the turn; the modifiers projection follows shell key input, so either is set without the other
    /// The window title and minimum size to set after the turn.
    pub(crate) window_request: Option<(String, Option<Size>)>,
    /// The platform effects to carry out after the turn.
    pub(crate) effects: Vec<(RequestId, DeferredRequest)>,
    /// CPU captures awaiting the next settled host boundary.
    pub(crate) snapshots: Vec<(RequestId, NodeId)>,
    /// The imports and drops of shared textures to carry out after the
    /// turn, in the order written.
    pub(crate) external_writes: Vec<ExternalWrite>,
    /// The drawable roots and what the app asked of each export.
    pub(crate) drawables: super::drawables::state::Drawables,
    #[cfg(debug_assertions)]
    pub(crate) lint: crate::check::lint::state::Lint,
}

/// A write about textures an external render engine shares, carried out
/// by the window after the turn (`Writer::import_external`,
/// `Writer::drop_external`). The textures hold OS handles, so the write
/// owns them (it is not `Clone`).
pub(crate) enum ExternalWrite {
    Import(kivact_types::ExternalImageId, kivact_types::SharedTextures),
    Drop(kivact_types::ExternalImageId),
}

impl Model {
    /// A model for `window` loading images with the App's shared resources.
    pub(crate) fn new(window: WindowId, viewport: Size, shared: &super::events::Shared) -> Self {
        Self {
            window,
            tree: NodeTree::default(),
            layout: Default::default(),
            layout_motion: Default::default(),
            playback: Default::default(),
            sources: Default::default(),
            measure: Default::default(),
            recent: Default::default(),
            editors: Vec::new(),
            events: shared.events.clone(),
            changes: super::changes::Changes::initial(),
            viewport,
            focus: super::focus::state::Focus::default(),
            pointer: super::pointer::state::Pointer::default(),
            admission: super::modal::state::Admission::default(),
            modifiers: {
                let mut sent = kivact_types::cell::Sent::new();
                sent.update(Modifiers::default());
                sent
            },
            key_down_targets: super::focus::state::KeyDownTargets::default(),
            edits: super::edit::state::Edits::default(),
            motion: super::state_motion::state::StateMotion::default(),
            presence: super::presence::state::Presences::default(),
            assets: Rc::clone(&shared.assets),
            log: shared.log.clone(),
            paint: PaintState::new(),
            a11y: A11yState::default(),
            frame: super::frame::state::Frame::default(),
            window_request: None,
            effects: vec![],
            snapshots: Vec::new(),
            external_writes: Vec::new(),
            drawables: Default::default(),
            #[cfg(debug_assertions)]
            lint: Default::default(),
        }
    }
}

impl Model {
    /// The node tree.
    pub(crate) fn tree(&self) -> &NodeTree {
        &self.tree
    }

    /// The node `id`.
    pub(crate) fn node(&self, id: NodeId) -> Option<&HostNode> {
        self.tree.get(id)
    }

    /// Mark `id`'s subtree facts stale for the next paint list. A transparent
    /// node marks the flat parent that holds the children it contributes, and
    /// those children (their own layer reads that parent's look).
    pub(crate) fn stale_fact(&mut self, id: NodeId) {
        let Some(node) = self.tree.get(id) else {
            return;
        };
        if !node.is_transparent() {
            self.paint.stale_fact(id);
            return;
        }
        if let Some(at) = self.tree.flat_parent(node).or_else(|| self.tree.root()) {
            self.paint.stale_fact(at);
        }
        let children: Vec<NodeId> = self.tree.flat_children(id).collect();
        for child in children {
            self.paint.stale_fact(child);
        }
    }

    /// Computed geometry from the retained node.
    pub(crate) fn computed_layout(&self, id: NodeId) -> Option<kivact_layout::NodeLayout> {
        self.layout.computed_layout(&self.tree, id)
    }

    /// The window size in logical pixels.
    pub(crate) fn window(&self) -> Size {
        self.viewport
    }

    /// Queue a host event after every event queued so far.
    pub(crate) fn push(&mut self, event: HostEvent) {
        self.events.push(event);
    }

    /// Existing native document/session changed; geometry reports can still
    /// promote this to a subtree/metadata refresh during the same settle.
    pub(crate) fn repaint_text(&mut self, id: NodeId) {
        if self.tree.contains(id) {
            self.a11y.payload(id);
            self.changes.mark(id, super::changes::Dirty::REDRAW);
        }
    }

    /// `id` is drawn again (its paint or state styles): only its own payload
    /// (its scene regions' padding box) can change; a state style that moves
    /// or reorders it moves its subtree.
    pub(crate) fn repaint(&mut self, id: NodeId) {
        if self.tree.contains(id) {
            self.a11y.payload(id);
            if self
                .node(id)
                .is_some_and(|n| n.states().is_some_and(crate::tree::states_place))
            {
                self.a11y.moved(id);
            }
            self.changes.mark(id, super::changes::Dirty::REDRAW);
        }
    }

    /// The scroll offset of `id` changed: its own payload and the clipping
    /// lists at or above it. Every other payload under it keeps its
    /// content-space bounds, except the layers under it (an overlay, a fixed
    /// box), drawn at their window origins: they move in content space.
    /// Overlays anchored under it follow (`a11y::build::payload`).
    pub(crate) fn a11y_scrolled(&mut self, id: NodeId) {
        self.a11y.scrolled(id);
        let layers: Vec<NodeId> = self
            .tree
            .overlays
            .iter()
            .chain(self.tree.fixed.iter())
            .copied()
            .filter(|x| *x != id && self.tree.is_in_subtree(*x, id))
            .collect();
        for layer in layers {
            self.a11y.moved(layer);
        }
    }
}
