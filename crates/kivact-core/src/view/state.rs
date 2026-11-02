//! Role: View state and reactive composition.
//!
//! Per-App state of the view layer (what the host's start established: the
//! retained metadata and engine writer, the environment signals, the keyboard
//! focus source; and the last hovered node, the pointer drag the overlay
//! policy tracks, outstanding host requests), installed as a runtime
//! extension when the App is created.

use std::any::Any;
use std::cell::{Cell, OnceCell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::Selector;
use kivact_reactive::{ErrorKind, UiError};
use kivact_reactive::{Owner, Runtime};
use kivact_reactive::{Signal, Value};
use kivact_wire::{HostEnv, Modifiers, NodeId, PlatformResult};

use super::tree::Tree;
use kivact_style::draw::Rect;
use kivact_task::Completion;
use kivact_wire::Capabilities;

/// What a [`NodeRef`](super::NodeRef) is bound to.
#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Bound {
    /// A host node.
    Node(NodeId),
    /// A canvas region of a host node, with that region's pointer hover.
    Region {
        node: NodeId,
        region: kivact_style::draw::RegionId,
        hovered: Value<bool>,
    },
}

impl Bound {
    /// The host node, whole or by region.
    pub(crate) fn node(self) -> NodeId {
        match self {
            Bound::Node(node) | Bound::Region { node, .. } => node,
        }
    }

    /// The focus target the binding names.
    pub(crate) fn target(self) -> kivact_wire::FocusTarget {
        match self {
            Bound::Node(node) => kivact_wire::FocusTarget { node, region: None },
            Bound::Region { node, region, .. } => kivact_wire::FocusTarget {
                node,
                region: Some(region),
            },
        }
    }
}

/// What a [`NodeRef`](super::NodeRef) stores under its owner: the bound
/// target and its reactive readers.
pub(crate) struct NodeRefState {
    pub(crate) owner: Owner,
    /// The one source of the binding (`None` while unbound). Tracked readers `get` it;
    /// untracked paths use `try_get_untracked`, where a disposed signal reads as unbound.
    pub(crate) bound: Signal<Option<Bound>>,
    /// `NodeRef::rect` was called: every node the ref binds to has its layout observed.
    pub(crate) observes_layout: Cell<bool>,
    pub(crate) rect_value: Value<Option<Rect>>,
    pub(crate) scroll_value: Value<crate::draw::Point>,
}

/// The node the host last reported under the pointer (`view::hover`).
#[derive(Clone, Copy)]
pub(crate) enum Hover {
    /// No `NodeRef::hovered` asked yet: the last report as a plain value.
    Plain(Option<NodeId>),
    /// The graph source the first `NodeRef::hovered` created; it holds the last report.
    Reactive(Signal<Option<NodeId>>),
}

/// A focus request made in this turn and not sent to the host yet.
pub(crate) struct QueuedFocus {
    pub(crate) id: u64,
    pub(crate) target: kivact_wire::FocusTarget,
    /// Whether the host shows a focus indicator for it.
    pub(crate) visible: bool,
    pub(crate) waiter: FocusWaiter,
}

/// The App's host environment: one signal holding the whole `HostEnv` the
/// client writes, and one memo per field a reader asked for, so a change of
/// one field notifies only the readers of that field.
pub(crate) struct EnvState {
    /// The root owner: the memos live as long as the App.
    owner: Owner,
    env: Signal<HostEnv>,
    /// The memos made so far, by the name of the accessor that asked
    /// (each a `Value<T>` of its field's type).
    fields: RefCell<Vec<(&'static str, Box<dyn Any>)>>,
}

impl EnvState {
    /// The environment `env` at start, as a signal of `owner`.
    pub(crate) fn new(owner: Owner, env: HostEnv, at: &'static Location<'static>) -> Self {
        Self {
            owner,
            env: owner.signal(env, at),
            fields: RefCell::new(Vec::new()),
        }
    }

    /// The host reports `env`; readers of the fields that differ are
    /// notified.
    pub(crate) fn set(&self, env: HostEnv) {
        self.env.set(env);
    }

    /// A memo of the field `f` reads, shared by every call with the same
    /// `name`: it re-runs its readers only when that field changed. Made
    /// on first use, whatever context asks (a memo, an effect, a handler).
    #[track_caller]
    pub(crate) fn field<T: Clone + PartialEq + 'static>(
        &self,
        name: &'static str,
        f: fn(&HostEnv) -> T,
    ) -> Value<T> {
        if let Some((_, made)) = self.fields.borrow().iter().find(|(n, _)| *n == name) {
            return *made
                .downcast_ref::<Value<T>>()
                .expect("invariant: one field type per environment accessor");
        }
        let env = self.env;
        let made = self
            .owner
            .with_setup(|| kivact_reactive::memo(move || env.with(f)));
        self.fields.borrow_mut().push((name, Box::new(made)));
        made
    }
}

/// The reactive focus nodes of an App (created at root mount).
#[derive(Clone, Copy)]
pub(crate) struct FocusKeys {
    /// The focused node as last reported by the host.
    pub(crate) node: Signal<Option<kivact_wire::FocusTarget>>,
    /// "Is this node focused", notifying only the previous and next node.
    pub(crate) selector: Selector<kivact_wire::FocusTarget>,
    pub(crate) visible: Signal<bool>,
}

/// Who waits for the answer of a focus request.
pub(crate) enum FocusWaiter {
    /// Report a failure to the owner's error chain, located at the call.
    Report(Owner, &'static Location<'static>),
    /// A focus restore: the core's own when a popup or modal overlay
    /// closes (`report: None`), or `NodeRef::restore_focus` (the ref's
    /// owner and the call). It is dropped without a request or an answer
    /// when, at the flush, its target may not take the focus back
    /// (`Reader::can_take_focus`), the reporting owner is gone, or an
    /// explicit request of the same flush wins; a request the host then
    /// refuses is reported like `Report` when `report` names where.
    Restore {
        report: Option<(Owner, &'static Location<'static>)>,
    },
}

/// Who waits for the answer of a platform effect.
pub(crate) enum EffectWaiter {
    /// Report a failure to the owner's error chain (the default sink
    /// without an owner), located at the call; a `Failed` answer is an
    /// error of kind `failed`.
    Report {
        owner: Option<Owner>,
        at: &'static Location<'static>,
        failed: ErrorKind,
    },
    /// Complete a future.
    Future(Completion<Result<PlatformResult, UiError>>),
}

/// Outstanding host requests of an App.
#[derive(Default)]
pub(crate) struct Requests {
    pub(crate) next_request: u64,
    /// Focus requests of this turn, in request order (`requests::flush_focus` sends or cancels each).
    pub(crate) queued_focus: Vec<QueuedFocus>,
    /// Focus requests sent to the host, by request id, until it answers.
    pub(crate) sent_focus: HashMap<u64, FocusWaiter>,
    pub(crate) effects: HashMap<u64, EffectWaiter>,
    /// Activations `NodeRef::activate` asked for, run by the client after
    /// the handler that asked.
    pub(crate) pending_activations: Vec<kivact_wire::FocusTarget>,
}

impl Requests {
    pub(crate) fn request_id(&mut self) -> u64 {
        self.next_request += 1;
        self.next_request
    }
}

/// What the host's start establishes for the App's whole life
/// ([`ViewState::start`]): set once, before any App code or host event runs.
pub(crate) struct Started {
    /// The retained metadata and direct engine access of the whole App.
    pub(super) tree: RefCell<Tree>,
    /// The host environment.
    pub(super) env: Rc<EnvState>,
    /// What the host's platform can do (fixed for the App's life).
    pub(super) capabilities: Capabilities,
    /// The keyboard focus source.
    pub(super) focus: FocusKeys,
    /// The window's minimum size (`None`: no minimum).
    pub(super) min_window_size: Option<kivact_wire::Size>,
    /// The adapter of the window's GPU device that can import shared
    /// textures (`None` until the host reports one).
    #[cfg(feature = "gpu-interop")]
    pub(super) adapter: Signal<Option<kivact_wire::GpuAdapter>>,
}

/// Everything the view layer keeps per App. Fields drop in declaration
/// order.
pub(crate) struct ViewState {
    /// What the host's start established (`Started`).
    started: OnceCell<Started>,
    /// Missing capabilities the App relied on and that were noted.
    noted_missing: RefCell<std::collections::BTreeSet<&'static str>>,
    /// The node the host reported under the pointer last.
    hover: Cell<Hover>,
    /// Whether the last key or pointer press the App got was a pointer press
    /// (`NodeRef::focus` then asks for a focus without a visible indicator).
    pointer_modality: Cell<bool>,
    /// The modifier keys the host reported last (`view::current_modifiers`).
    modifiers: Cell<Modifiers>,
    /// Focus / platform requests waiting for the host.
    pub(super) requests: RefCell<Requests>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewState {
    /// A view state the host has not started yet.
    pub(crate) fn new() -> Self {
        ViewState {
            started: OnceCell::new(),
            noted_missing: RefCell::new(std::collections::BTreeSet::new()),
            hover: Cell::new(Hover::Plain(None)),
            pointer_modality: Cell::new(false),
            modifiers: Cell::new(Modifiers::default()),
            requests: RefCell::new(Requests::default()),
        }
    }

    /// Install what the host's start established (`view::root::start`).
    pub(crate) fn start(&self, started: Started) {
        assert!(
            self.started.set(started).is_ok(),
            "invariant: the host starts an App once"
        );
    }

    fn started(&self) -> &Started {
        self.started
            .get()
            .expect("invariant: App code and host events run only after the host started the App")
    }

    /// Release spare request/tree capacity. Environment memos and live
    /// reactive/user resources are authority, not rebuildable caches.
    pub(crate) fn release_capacity(&self) {
        self.started().tree.borrow_mut().release_capacity();
        let mut requests = self.requests.borrow_mut();
        requests.queued_focus.shrink_to_fit();
        requests.sent_focus.shrink_to_fit();
        requests.effects.shrink_to_fit();
        requests.pending_activations.shrink_to_fit();
    }

    /// The view state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &ViewState {
        rt.ext::<ViewState>()
    }

    /// Read the retained tree of `rt`'s App.
    #[inline]
    pub(crate) fn tree(rt: &Runtime) -> Ref<'_, Tree> {
        Self::of(rt).started().tree.borrow()
    }

    /// Change the retained tree of `rt`'s App. A prop written through the
    /// borrow names its writer, `rt.running_effect()` (`Tree::update`).
    #[inline]
    pub(crate) fn tree_mut(rt: &Runtime) -> RefMut<'_, Tree> {
        Self::of(rt).started().tree.borrow_mut()
    }

    /// The host environment.
    #[inline]
    pub(crate) fn env(&self) -> Rc<EnvState> {
        self.started().env.clone()
    }

    /// What the host's platform can do.
    pub(crate) fn capabilities(&self) -> Capabilities {
        self.started().capabilities
    }

    /// Whether the missing capability `name` has not been noted yet (it
    /// is noted from now on).
    pub(crate) fn note_once_missing(&self, name: &'static str) -> bool {
        self.noted_missing.borrow_mut().insert(name)
    }

    /// The keyboard focus source.
    #[inline]
    pub(crate) fn focus_keys(&self) -> FocusKeys {
        self.started().focus
    }

    /// The window's minimum size (`None`: no minimum).
    pub(crate) fn min_window_size(&self) -> Option<kivact_wire::Size> {
        self.started().min_window_size
    }

    /// The adapter of the window's GPU device that can import shared
    /// textures.
    #[cfg(feature = "gpu-interop")]
    pub(crate) fn adapter(&self) -> Signal<Option<kivact_wire::GpuAdapter>> {
        self.started().adapter
    }

    /// Whether the last key or pointer press was a pointer press.
    pub(crate) fn pointer_modality(&self) -> bool {
        self.pointer_modality.get()
    }

    /// Record the modality of a key press (`false`) or pointer press (`true`).
    pub(crate) fn set_pointer_modality(&self, pointer: bool) {
        self.pointer_modality.set(pointer);
    }

    /// The modifier keys the host reported last.
    pub(crate) fn modifiers(&self) -> Modifiers {
        self.modifiers.get()
    }

    /// Record the modifier keys the host reported.
    pub(crate) fn set_modifiers(&self, modifiers: Modifiers) {
        self.modifiers.set(modifiers);
    }

    /// The node the host reported under the pointer last.
    pub(crate) fn hover(&self) -> Hover {
        self.hover.get()
    }

    /// Replace the hover record (a report, or the first `NodeRef::hovered`).
    pub(crate) fn set_hover(&self, hover: Hover) {
        self.hover.set(hover);
    }
}
