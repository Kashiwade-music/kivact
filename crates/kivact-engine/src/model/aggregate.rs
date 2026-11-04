//! Role: The Model aggregate: every passive state group, its creation and reads.

use crate::a11y::state::A11yState;
use crate::contract::HostEvent;
use crate::mutation::Writes;
use crate::paint::state::PaintState;
use crate::tree::NodeTree;
use crate::tree::node::HostNode;
use arvane::asset::AssetStore;
use kivact_types::{Modifiers, NodeId, PlatformRequest, RequestId, Revision, Size};

/// Requests that must wait until the current client turn has finished.
pub(crate) enum DeferredRequest {
    Platform(PlatformRequest),
    ReducedMotion(Option<bool>),
}

/// The engine's state: the node tree and one field per state group.
pub(crate) struct Model {
    pub(crate) tree: NodeTree,
    pub(crate) layout_columns: kivact_layout::NodeLayoutColumns,
    pub(crate) layout_state: kivact_layout::LayoutState,
    pub(crate) measure: crate::measure::state::MeasureColumns,
    pub(crate) editors: Vec<Option<Box<crate::measure::editor::EditorNode>>>,
    pub(crate) events: super::events::EventQueue,
    pub(crate) changes: super::changes::Changes,
    pub(crate) writes: Writes,
    pub(crate) viewport: Size,
    pub(crate) focus: super::focus::state::Focus,
    pub(crate) pointer: super::pointer::state::Pointer,
    /// The drags admitted before a modal opened (`modal`).
    pub(crate) admission: super::modal::state::Admission,
    /// The keyboard modifiers the shell reports held.
    pub(crate) modifiers: Modifiers,
    pub(crate) edits: super::edit::state::Edits,
    pub(crate) motion: super::state_motion::state::StateMotion,
    pub(crate) presence: super::presence::state::Presences,
    pub(crate) assets: std::cell::RefCell<crate::lower::assets::ArvaneAssets>,
    pub(crate) paint: PaintState,
    pub(crate) a11y: A11yState,
    pub(crate) frame: super::frame::state::Frame,
    /// The window title and minimum size to set after the turn.
    pub(crate) window_request: Option<(String, Option<Size>)>,
    /// The platform effects to carry out after the turn.
    pub(crate) effects: Vec<(RequestId, DeferredRequest)>,
    /// The imports and drops of shared textures to carry out after the
    /// turn, in the order written.
    #[cfg(feature = "gpu-interop")]
    pub(crate) external_writes: Vec<ExternalWrite>,
    #[cfg(debug_assertions)]
    pub(crate) lint: crate::check::lint::state::Lint,
}

/// A write about textures an external render engine shares, carried out
/// by the window after the turn (`Writer::import_external`,
/// `Writer::drop_external`). The textures hold OS handles, so the write
/// owns them (it is not `Clone`).
#[cfg(feature = "gpu-interop")]
pub(crate) enum ExternalWrite {
    Import(kivact_types::ExternalImageId, kivact_types::SharedTextures),
    Drop(kivact_types::ExternalImageId),
}

impl Model {
    /// A model loading images with `store` (the host's worker-thread store;
    /// tests may pass an inline one).
    pub(crate) fn new(viewport: Size, store: AssetStore) -> Self {
        Self {
            tree: NodeTree::default(),
            layout_columns: Default::default(),
            layout_state: Default::default(),
            measure: Default::default(),
            editors: Vec::new(),
            events: super::events::EventQueue::default(),
            changes: super::changes::Changes::initial(),
            writes: Writes::default(),
            viewport,
            focus: super::focus::state::Focus::default(),
            pointer: super::pointer::state::Pointer::default(),
            admission: super::modal::state::Admission::default(),
            modifiers: Modifiers::default(),
            edits: super::edit::state::Edits::default(),
            motion: super::state_motion::state::StateMotion::default(),
            presence: super::presence::state::Presences::default(),
            assets: std::cell::RefCell::new(crate::lower::assets::ArvaneAssets::new(store)),
            paint: PaintState::new(),
            a11y: A11yState::default(),
            frame: super::frame::state::Frame::default(),
            window_request: None,
            effects: vec![],
            #[cfg(feature = "gpu-interop")]
            external_writes: Vec::new(),
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

    /// Computed geometry from the retained node.
    pub(crate) fn computed_layout(&self, id: NodeId) -> Option<kivact_layout::NodeLayout> {
        self.layout_columns.computed_layout(&self.tree, id)
    }

    /// The window size in logical pixels.
    pub(crate) fn window(&self) -> Size {
        self.viewport
    }

    /// Queue a host event after every event queued so far.
    pub(crate) fn push(&mut self, event: HostEvent) {
        self.events.push(event);
    }

    /// Whether events are waiting.
    pub(crate) fn has_events(&self) -> bool {
        self.events.has_events()
    }

    /// The last applied revision.
    pub(crate) fn applied(&self) -> Revision {
        self.events.applied()
    }

    /// Existing native document/session changed; geometry reports can still
    /// promote this to a subtree/metadata refresh during the same settle.
    pub(crate) fn repaint_text(&mut self, id: NodeId) {
        if self.tree.contains(id) {
            self.changes.mark(
                id,
                super::changes::Dirty::REDRAW.union(super::changes::Dirty::A11Y_LOCAL_TEXT),
            );
        }
    }

    pub(crate) fn repaint(&mut self, id: NodeId) {
        if self.tree.contains(id) {
            self.changes.mark(
                id,
                super::changes::Dirty::REDRAW.union(super::changes::Dirty::A11Y_METADATA),
            );
        }
    }
}
