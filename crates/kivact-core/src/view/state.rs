//! Role: View state and reactive composition.
//!
//! Per-App state of the view layer (the retained metadata and engine writer,
//! the environment signals, the keyboard focus source, the last hovered node,
//! the pointer drag the overlay policy tracks, outstanding host requests),
//! installed as a runtime extension when the App is created.

use std::any::Any;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_reactive::Selector;
use kivact_reactive::{ErrorKind, UiError};
use kivact_reactive::{Owned, Owner, Runtime};
use kivact_reactive::{Signal, Value};
use kivact_wire::{HostEnv, NodeId, PlatformResult};

use super::tree::Tree;
use kivact_style::draw::Rect;
use kivact_task::Completion;
use kivact_wire::Capabilities;

/// What a [`NodeRef`](super::NodeRef) stores under its owner: the bound
/// target and its reactive readers.
pub(crate) struct NodeRefState {
    pub(crate) owner: Owner,
    pub(crate) node: Cell<Option<kivact_wire::FocusTarget>>,
    /// The binding for reactive readers and region bindings.
    pub(crate) node_sig: Signal<Option<kivact_wire::FocusTarget>>,
    pub(crate) rect_value: Value<Option<Rect>>,
    pub(crate) scroll_value: Value<crate::draw::Point>,
    pub(crate) regional_hover: Cell<Option<Value<bool>>>,
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
    pub(crate) focus_waiters: HashMap<u64, FocusWaiter>,
    /// Focus requests not sent yet: (node, request id).
    pub(crate) pending_focus: Vec<(kivact_wire::FocusTarget, u64, bool)>,
    pub(crate) effects: HashMap<u64, EffectWaiter>,
    /// `NodeRef`s whose node is observed as soon as they are bound.
    pub(crate) observe_on_bind: Vec<Owned<NodeRefState>>,
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

/// Everything the view layer keeps per App. Fields drop in declaration
/// order.
pub(crate) struct ViewState {
    /// The retained metadata and direct engine access of the whole App.
    tree: RefCell<Tree>,
    min_window_size: Cell<Option<kivact_wire::Size>>,
    /// The host environment, once the root is mounted.
    env: RefCell<Option<Rc<EnvState>>>,
    /// What the host's platform can do, once the root is mounted.
    capabilities: Cell<Capabilities>,
    /// Missing capabilities the App relied on and that were noted.
    noted_missing: RefCell<std::collections::BTreeSet<&'static str>>,
    /// The keyboard focus source, once the root is mounted.
    focus: Cell<Option<FocusKeys>>,
    /// The node the host reported under the pointer last.
    hovered: Cell<Option<NodeId>>,
    /// The hover source, once the first `NodeRef::hovered` created it.
    hover: Cell<Option<Signal<Option<NodeId>>>>,
    /// Whether the last key or pointer press the App got was a pointer press
    /// (`NodeRef::focus` then asks for a focus without a visible indicator).
    pointer_modality: Cell<bool>,
    /// Focus / platform requests waiting for the host.
    pub(super) requests: RefCell<Requests>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self::new()
    }
}

impl ViewState {
    /// An empty tree.
    pub(crate) fn new() -> Self {
        ViewState {
            tree: RefCell::new(Tree::new()),
            min_window_size: Cell::new(None),
            env: RefCell::new(None),
            capabilities: Cell::new(Capabilities::default()),
            noted_missing: RefCell::new(std::collections::BTreeSet::new()),
            focus: Cell::new(None),
            hovered: Cell::new(None),
            hover: Cell::new(None),
            pointer_modality: Cell::new(false),
            requests: RefCell::new(Requests::default()),
        }
    }

    pub(crate) fn min_window_size(&self) -> Option<kivact_wire::Size> {
        self.min_window_size.get()
    }
    pub(crate) fn set_min_window_size(&self, size: Option<kivact_wire::Size>) {
        self.min_window_size.set(size);
    }

    /// Release spare request/tree capacity. Environment memos and live
    /// reactive/user resources are authority, not rebuildable caches.
    pub(crate) fn release_capacity(&self) {
        self.tree.borrow_mut().release_capacity();
        let mut requests = self.requests.borrow_mut();
        requests.focus_waiters.shrink_to_fit();
        requests.pending_focus.shrink_to_fit();
        requests.effects.shrink_to_fit();
        requests.observe_on_bind.shrink_to_fit();
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
        Self::of(rt).tree.borrow()
    }

    /// Change the retained tree of `rt`'s App. A prop written through the
    /// borrow names its writer, `rt.running_effect()` (`Tree::update`).
    #[inline]
    pub(crate) fn tree_mut(rt: &Runtime) -> RefMut<'_, Tree> {
        Self::of(rt).tree.borrow_mut()
    }

    /// The host environment (`None` before the root is mounted).
    #[inline]
    pub(crate) fn env(&self) -> Option<Rc<EnvState>> {
        self.env.borrow().clone()
    }

    /// Install the host environment (root mount).
    pub(crate) fn set_env(&self, env: Rc<EnvState>) {
        *self.env.borrow_mut() = Some(env);
    }

    /// What the host's platform can do (none before the root is mounted).
    pub(crate) fn capabilities(&self) -> Capabilities {
        self.capabilities.get()
    }

    /// Whether the missing capability `name` has not been noted yet (it
    /// is noted from now on).
    pub(crate) fn note_once_missing(&self, name: &'static str) -> bool {
        self.noted_missing.borrow_mut().insert(name)
    }

    /// Record what the host's platform can do (root mount).
    pub(crate) fn set_capabilities(&self, capabilities: Capabilities) {
        self.capabilities.set(capabilities);
    }

    /// The keyboard focus source (`None` before the root is mounted).
    #[inline]
    pub(crate) fn focus_keys(&self) -> Option<FocusKeys> {
        self.focus.get()
    }

    /// Install the keyboard focus source (root mount).
    pub(crate) fn set_focus_keys(&self, keys: FocusKeys) {
        self.focus.set(Some(keys));
    }

    /// The node the host reported under the pointer last.
    pub(crate) fn hovered(&self) -> Option<NodeId> {
        self.hovered.get()
    }

    /// Record the node the host reported under the pointer.
    pub(crate) fn set_hovered(&self, node: Option<NodeId>) {
        self.hovered.set(node);
    }

    /// Whether the last key or pointer press was a pointer press.
    pub(crate) fn pointer_modality(&self) -> bool {
        self.pointer_modality.get()
    }

    /// Record the modality of a key press (`false`) or pointer press (`true`).
    pub(crate) fn set_pointer_modality(&self, pointer: bool) {
        self.pointer_modality.set(pointer);
    }

    /// The hover source (`None` until the first `NodeRef::hovered`).
    pub(crate) fn hover(&self) -> Option<Signal<Option<NodeId>>> {
        self.hover.get()
    }

    /// Install the hover source (its first use).
    pub(crate) fn set_hover(&self, node: Signal<Option<NodeId>>) {
        self.hover.set(Some(node));
    }
}
