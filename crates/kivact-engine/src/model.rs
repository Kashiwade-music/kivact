//! Role: The engine's one model: the node tree, its state groups and the event queue.
//!
//! One Model owns the retained node tree, the ordered event queue, the
//! pending Changes and one state group per concern. Each group's module
//! declares its state with private fields together with the methods that
//! change it; the semantic algorithms extend this same Model:
//!
//! - `crate::tree`: the node store, links and passive typed properties.
//!   Model owns derived layout/measurement/paint columns, editor payloads
//!   and resources; `interact` processes them without changing tree ownership.
//! - `changes`: what writes, input and layout outdated, and the typed
//!   [`crate::mutation::Mutation`] flags; `sync` alone dispatches it into layout,
//!   paint and accessibility work. `apply` applies the structural writes.
//! - `focus`: the focus target, Tab order and the reveals after layout;
//!   `modal` and `dismiss` confine and close overlays.
//! - `pointer`: the pointer state machine and the shell's pointer input;
//!   `keys` routes keys, `edit` keeps the text inputs' editing state and
//!   the IME's view of it.
//! - `states`, `state_motion`: state styles and their transitions;
//!   `transform`: element transforms; `scrollbar`: scrollbar geometry.
//! - `frame`: clocks, frame requests, smooth scrolls and drawability.
//! - `semantics`: the semantics policy; `events`: the event queue.
//!
//! Paint and accessibility keep their groups in their own modules
//! (`paint::state`, `a11y::state`).
//! Events always pass through the single ordered event queue.
//!
//! The model is the meaning of the tree, which knows no window, drawing or
//! shaping: focus, Tab order, modals and outside dismissal, the pointer,
//! the native editing state and the interpretation of keys, state styles,
//! the cursor and the semantics policy are decided here. `paint`, `a11y`
//! and `host` map that state onto Arvane and decide none of it. Besides
//! the groups, the model holds the window size (`viewport`, which the
//! layout fills) and, apart from it, the window title and minimum size to
//! set after the turn (`window_request`) and the imports and drops of
//! shared textures to carry out after it (`external_writes`, feature
//! `gpu-interop`).
//!
//! A group's state lives in a leaf `state` module under the module that
//! changes it: `focus::state::Focus` (the focus target, whether its
//! indicator shows, the reveals after layout), `pointer::state::Pointer`
//! (the pointer state machine's tracks and the shell's position, buttons,
//! touch and pen contacts, scrollbar drag, file drop target and shown
//! cursor), `edit::state::Edits` (the text inputs with an editing state and
//! the text last given to the IME), `state_motion::state::StateMotion` (the
//! moving state style transitions), `presence::state::Presences` (nodes
//! awaiting their entrance, leaving subtrees), `frame::state::Frame`
//! (clocks, frame requests, smooth scrolls, the surface) and
//! `check::lint::state::Lint` (debug builds) and `modal::state::Admission`
//! (the pointer drags admitted before a modal opened).
//!
//! Invariants:
//! - A group's fields are `pub(super)`: only its own module writes them,
//!   and everything else goes through its methods.
//! - A leaf `state` module never refers to the modules that process it
//!   (those with `impl Model`), so the Model holding every group makes no
//!   cycle of module references (xtask checks it).
//! - The Model is never cloned; work borrows the field groups it needs one
//!   by one.

pub(crate) mod apply;
pub(crate) mod changes;
pub(crate) mod dismiss;
pub(crate) mod edit;
pub(crate) mod events;
pub(crate) mod focus;
pub(crate) mod frame;
mod geometry;
pub(crate) mod interact;
pub(crate) mod keys;
mod lifecycle;
pub(crate) mod modal;
pub(crate) mod pointer;
pub(crate) mod presence;
pub(crate) mod scrollbar;
pub(crate) mod semantics;
pub(crate) mod state_motion;
pub(crate) mod states;
pub(crate) mod text;
pub(crate) mod transform;

use crate::a11y::state::A11yState;
use crate::mutation::Writes;
use crate::paint::state::PaintState;
use crate::tree::NodeTree;
use crate::tree::node::HostNode;
use arvane::asset::AssetStore;
use kivact_wire::{HostEvent, Modifiers, NodeId, PlatformRequest, RequestId, Revision, Size};

/// The engine's state: the node tree and one field per state group.
pub(crate) struct Model {
    pub(crate) tree: NodeTree,
    pub(crate) layout_columns: kivact_layout::NodeLayoutColumns,
    pub(crate) layout_state: kivact_layout::LayoutState,
    pub(crate) measure: crate::measure::state::MeasureColumns,
    pub(crate) editors: Vec<Option<Box<crate::measure::editor::EditorNode>>>,
    pub(crate) events: events::EventQueue,
    pub(crate) changes: changes::Changes,
    pub(crate) writes: Writes,
    pub(crate) viewport: Size,
    pub(crate) focus: focus::state::Focus,
    pub(crate) pointer: pointer::state::Pointer,
    /// The drags admitted before a modal opened (`modal`).
    pub(crate) admission: modal::state::Admission,
    /// The keyboard modifiers the shell reports held.
    pub(crate) modifiers: Modifiers,
    pub(crate) edits: edit::state::Edits,
    pub(crate) motion: state_motion::state::StateMotion,
    pub(crate) presence: presence::state::Presences,
    pub(crate) assets: std::cell::RefCell<kivact_lower::assets::ArvaneAssets>,
    pub(crate) paint: PaintState,
    pub(crate) a11y: A11yState,
    pub(crate) frame: frame::state::Frame,
    /// The window title and minimum size to set after the turn.
    pub(crate) window_request: Option<(String, Option<Size>)>,
    /// The platform effects to carry out after the turn.
    pub(crate) effects: Vec<(RequestId, PlatformRequest)>,
    /// A font was registered during the turn.
    pub(crate) fonts_pending: bool,
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
    Import(kivact_wire::ExternalImageId, kivact_wire::SharedTextures),
    Drop(kivact_wire::ExternalImageId),
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
            events: events::EventQueue::default(),
            changes: changes::Changes::initial(),
            writes: Writes::default(),
            viewport,
            focus: focus::state::Focus::default(),
            pointer: pointer::state::Pointer::default(),
            admission: modal::state::Admission::default(),
            modifiers: Modifiers::default(),
            edits: edit::state::Edits::default(),
            motion: state_motion::state::StateMotion::default(),
            presence: presence::state::Presences::default(),
            assets: std::cell::RefCell::new(kivact_lower::assets::ArvaneAssets::new(store)),
            paint: PaintState::new(),
            a11y: A11yState::default(),
            frame: frame::state::Frame::default(),
            window_request: None,
            effects: vec![],
            fonts_pending: false,
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

    /// Take every queued event, in order.
    pub(crate) fn take_events(&mut self) -> Vec<HostEvent> {
        self.events.take()
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
                changes::Dirty::REDRAW.union(changes::Dirty::A11Y_LOCAL_TEXT),
            );
        }
    }

    pub(crate) fn repaint(&mut self, id: NodeId) {
        if self.tree.contains(id) {
            self.changes.mark(
                id,
                changes::Dirty::REDRAW.union(changes::Dirty::A11Y_METADATA),
            );
        }
    }
}
