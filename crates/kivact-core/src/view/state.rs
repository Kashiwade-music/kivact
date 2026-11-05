//! Role: View state and reactive composition.
//!
//! Per-App state of the view layer (what the host's start established: the
//! retained metadata and engine writer, the environment signals, the keyboard
//! focus source; and the last hovered node, the pointer drag the overlay
//! policy tracks, outstanding host requests), installed as a runtime
//! extension when the App is created.

use std::any::Any;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::Selector;
use kivact_runtime::{ErrorKind, UiError};
use kivact_runtime::{Owner, Runtime};
use kivact_runtime::{Signal, Value};
use kivact_types::{Environment, FocusTarget, Modifiers, NodeId, PlatformResult};

use super::tree::ViewNodes;
use kivact_runtime::Completion;
use kivact_types::Capabilities;

pub(crate) use super::ref_data::{Bound, LayoutObservation, NodeRefState, RefState};

/// The node the host last reported under the pointer (`view::hover`).
#[derive(Clone, Copy)]
pub(crate) enum PointerObservation<T: kivact_runtime::PropValue> {
    /// No `NodeRef::hovered` asked yet: the last report as a plain value.
    Plain(Option<T>),
    /// The graph source the first `NodeRef::hovered` created; it holds the last report.
    Reactive(Signal<Option<T>>),
}

/// Focus requests, with ordering independent from their stage.
#[derive(Default)]
pub(crate) struct FocusRequests {
    pub(crate) requests: HashMap<kivact_types::RequestId, FocusRequest>,
    pub(crate) order: Vec<kivact_types::RequestId>,
}

pub(crate) struct FocusRequest {
    pub(crate) target: kivact_types::FocusTarget,
    pub(crate) reply: FocusReply,
    pub(crate) stage: RequestStage,
    /// The requested focus indicator, independent of request progress.
    pub(crate) visible: bool,
}

pub(crate) enum RequestStage {
    Queued,
    Sent,
}

pub(crate) struct FocusCompletion {
    pub(crate) owner: Owner,
    pub(crate) at: &'static Location<'static>,
}

/// The App's host environment: one signal holding the whole `Environment` the
/// client writes, and one memo per field a reader asked for, so a change of
/// one field notifies only the readers of that field.
pub(crate) struct EnvState {
    /// The root owner: the memos live as long as the App.
    owner: Owner,
    env: Signal<Environment>,
    pub(crate) surface_active: Signal<bool>,
    /// The memos made so far, by the name of the accessor that asked
    /// (each a `Value<T>` of its field's type).
    fields: RefCell<Vec<(&'static str, Box<dyn Any>)>>,
}

impl EnvState {
    /// The environment `env` at start, as a signal of `owner`.
    pub(crate) fn new(
        owner: Owner,
        env: Environment,
        surface_active: bool,
        at: &'static Location<'static>,
    ) -> Self {
        let state = Self {
            owner,
            env: owner.signal(env, at),
            surface_active: owner.signal(surface_active, at),
            fields: RefCell::new(Vec::new()),
        };
        state.field("is_fullscreen", |e| e.fullscreen);
        state.field("is_motion_reduced", |e| e.reduced_motion);
        state.field("scale_factor", |e| e.scale_factor);
        state.field("safe_area", |e| e.safe_area);
        state.field("window_size", |e| e.window_size);
        state.field("pointer_settings", |e| e.pointer);
        state.field("keyboard_inset", |e| e.keyboard_inset);
        state.field("theme_hint", |e| e.theme_hint);
        state.field("locale", |e| e.locale.clone());
        state.field("frame_period_ns", |e| e.frame_period_ns);
        state
    }

    /// The host reports `env`; readers of the fields that differ are
    /// notified.
    pub(crate) fn set(&self, env: Environment) {
        self.env.set(env);
    }

    /// A memo of the field `f` reads, shared by every call with the same
    /// `name`: it re-runs its readers only when that field changed. Made
    /// on first use, whatever context asks (a memo, an effect, a handler).
    #[track_caller]
    pub(crate) fn field<T: kivact_runtime::PropValue>(
        &self,
        name: &'static str,
        f: fn(&Environment) -> T,
    ) -> Value<T> {
        if let Some((_, made)) = self.fields.borrow().iter().find(|(n, _)| *n == name) {
            return *made
                .downcast_ref::<Value<T>>()
                .expect("invariant: one field type per environment accessor");
        }
        let env = self.env;
        let made = self
            .owner
            .with_setup(|| kivact_runtime::memo(move || env.with(f)));
        self.fields.borrow_mut().push((name, Box::new(made)));
        made
    }
}

// xtask-allow(state-fields): Focus identity and visible indicator are independent host observations; selector only projects identity.
/// The reactive focus nodes of an App (created at root mount).
#[derive(Clone, Copy)]
pub(crate) struct FocusKeys {
    /// The focused node as last reported by the host.
    pub(crate) node: Signal<Option<kivact_types::FocusTarget>>,
    /// "Is this node focused", notifying only the previous and next node.
    pub(crate) selector: Selector<kivact_types::FocusTarget>,
    pub(crate) visible: Signal<bool>,
}

/// How a focus request completes.
pub(crate) enum FocusReply {
    Report(FocusCompletion),
    Restore,
    RestoreAndReport(FocusCompletion),
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
    pub(crate) focus: FocusRequests,
    pub(crate) effects: HashMap<u64, EffectWaiter>,
    /// Pending CPU captures, answered at the settled host boundary.
    pub(crate) snapshots: HashMap<u64, SnapshotWaiter>,
    /// Activations `NodeRef::activate` asked for, run by the client after
    /// the handler that asked.
    pub(crate) pending_activations: Vec<kivact_types::FocusTarget>,
    /// Actual capture releases awaiting the current pointer dispatch's cleanup.
    pub(crate) pointer_releases: Vec<(NodeId, kivact_types::PointerId)>,
}

pub(crate) struct SnapshotWaiter {
    pub(crate) owner: Owner,
    pub(crate) at: &'static Location<'static>,
    pub(crate) reply: Completion<Result<kivact_types::Pixels, UiError>>,
}

impl Requests {
    pub(crate) fn request_id(&mut self) -> u64 {
        self.next_request += 1;
        self.next_request
    }
}

// xtask-allow(state-fields): Initial render is an independent host fact; optional minimum size is construction configuration and requests own their lifecycles.
/// Everything the view layer keeps per App. Fields drop in declaration
/// order.
pub(crate) struct ViewState {
    /// The retained metadata and direct engine access of the whole App.
    pub(super) tree: RefCell<ViewNodes>,
    /// Removed node ids awaiting host revision cleanup.
    pub(crate) removed: RefCell<Vec<NodeId>>,
    /// The host environment.
    pub(super) env: Rc<EnvState>,
    /// What the host's platform can do (fixed for the App's life).
    pub(super) capabilities: Capabilities,
    pub(crate) capabilities_value: Value<Capabilities>,
    pub(crate) fonts_value: Signal<Vec<String>>,
    pub(crate) initial_render: Signal<bool>,
    /// The keyboard focus source.
    pub(super) focus: FocusKeys,
    /// The window's minimum size (`None`: no minimum).
    pub(super) min_window_size: Option<kivact_types::Size>,
    /// Missing capabilities the App relied on and that were noted.
    noted_missing: kivact_types::cell::ReportOnceSet<&'static str>,
    /// The node the host reported under the pointer last.
    hover: Cell<PointerObservation<kivact_types::FocusTarget>>,
    pub(crate) mouse: Cell<PointerObservation<crate::geometry::Point>>,
    /// Whether the last key or pointer press the App got was a pointer press
    /// (`NodeRef::focus` then asks for a focus without a visible indicator).
    pointer_modality: Cell<Modality>,
    /// The modifier keys the host reported last (`view::modifiers`).
    modifiers: Signal<Modifiers>,
    /// Focus / platform requests waiting for the host.
    pub(super) requests: RefCell<Requests>,
}

impl ViewState {
    /// Construct the complete state before mounting any application code.
    pub(crate) fn new(
        owner: Owner,
        info: &kivact_engine::contract::HostInfo,
        engine: kivact_engine::api::Writer,
        env: Rc<EnvState>,
        min_window_size: Option<kivact_types::Size>,
        at: &'static Location<'static>,
    ) -> Self {
        Self {
            tree: RefCell::new(ViewNodes::new(engine, owner)),
            removed: RefCell::new(Vec::new()),
            env,
            capabilities: info.capabilities,
            capabilities_value: owner.constant(info.capabilities, at),
            fonts_value: owner.signal(Vec::new(), at),
            initial_render: owner.signal(true, at),
            focus: FocusKeys::new(owner, at),
            min_window_size,
            noted_missing: Default::default(),
            hover: Cell::new(PointerObservation::Plain(None)),
            mouse: Cell::new(PointerObservation::Reactive(owner.signal(None, at))),
            pointer_modality: Cell::new(Modality::Keyboard),
            modifiers: owner.signal(Modifiers::default(), at),
            requests: RefCell::new(Requests::default()),
        }
    }

    /// Release spare request/tree capacity. Environment memos and live
    /// reactive/user resources are authority, not rebuildable caches.
    pub(crate) fn release_capacity(&self) {
        self.tree.borrow_mut().release_capacity();
        let mut requests = self.requests.borrow_mut();
        requests.focus.order.shrink_to_fit();
        requests.focus.requests.shrink_to_fit();
        requests.effects.shrink_to_fit();
        requests.snapshots.shrink_to_fit();
        requests.pending_activations.shrink_to_fit();
    }

    /// The view state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &ViewState {
        rt.ext::<ViewState>()
    }

    /// Read the retained tree of `rt`'s App.
    #[inline]
    pub(crate) fn tree(rt: &Runtime) -> Ref<'_, ViewNodes> {
        Self::of(rt).tree.borrow()
    }

    /// Change the retained tree of `rt`'s App. A prop written through the
    /// borrow names its writer, `rt.running_effect()` (`ViewNodes::update`).
    #[inline]
    pub(crate) fn tree_mut(rt: &Runtime) -> RefMut<'_, ViewNodes> {
        Self::of(rt).tree.borrow_mut()
    }

    /// The host environment.
    #[inline]
    pub(crate) fn env(&self) -> Rc<EnvState> {
        self.env.clone()
    }

    /// What the host's platform can do.
    pub(crate) fn capabilities(&self) -> Capabilities {
        self.capabilities
    }

    /// Whether the missing capability `name` has not been noted yet (it
    /// is noted from now on).
    pub(crate) fn note_once_missing(&self, name: &'static str) -> bool {
        self.noted_missing.note(name)
    }

    /// The keyboard focus source.
    #[inline]
    pub(crate) fn focus_keys(&self) -> FocusKeys {
        self.focus
    }

    /// The window's minimum size (`None`: no minimum).
    pub(crate) fn min_window_size(&self) -> Option<kivact_types::Size> {
        self.min_window_size
    }

    /// Whether the last key or pointer press was a pointer press.
    pub(crate) fn pointer_modality(&self) -> bool {
        matches!(self.pointer_modality.get(), Modality::Pointer)
    }

    /// Record the modality of a key press (`false`) or pointer press (`true`).
    pub(crate) fn set_pointer_modality(&self, pointer: bool) {
        self.pointer_modality.set(if pointer {
            Modality::Pointer
        } else {
            Modality::Keyboard
        });
    }

    /// The modifier keys the host reported last.
    pub(crate) fn modifiers(&self) -> Value<Modifiers> {
        self.modifiers.into()
    }

    /// Record the modifier keys the host reported.
    pub(crate) fn set_modifiers(&self, modifiers: Modifiers) {
        self.modifiers.set(modifiers);
    }

    /// The node the host reported under the pointer last.
    pub(crate) fn hover(&self) -> PointerObservation<kivact_types::FocusTarget> {
        self.hover.get()
    }

    /// Replace the hover record (a report, or the first `NodeRef::hovered`).
    pub(crate) fn set_hover(&self, hover: PointerObservation<kivact_types::FocusTarget>) {
        self.hover.set(hover);
    }
}

#[derive(Clone, Copy)]
enum Modality {
    Pointer,
    Keyboard,
}

impl FocusKeys {
    /// Create the focus source and its selector under the root `owner`.
    fn new(owner: Owner, at: &'static Location<'static>) -> FocusKeys {
        let node: Signal<Option<FocusTarget>> = owner.signal(None::<FocusTarget>, at);
        // `selector` registers under the setup owner.
        let selector = owner.with_setup(|| kivact_runtime::selector(node));
        let visible = owner.signal(false, at);
        FocusKeys {
            node,
            selector,
            visible,
        }
    }
}

impl super::tree::Removed {
    pub(crate) fn commit(self, rt: &kivact_runtime::Runtime) {
        ViewState::of(rt).removed.borrow_mut().extend(self.ids);
    }
}
