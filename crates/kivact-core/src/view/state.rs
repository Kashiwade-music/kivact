//! Role: View state and reactive composition.
//!
//! Per-App state of the view layer (what the host's start established: the
//! retained metadata and engine writer, the environment signals, the keyboard
//! focus source; and the last hovered node, the pointer drag the overlay
//! policy tracks, outstanding host requests), installed as a runtime
//! extension when the App is created.

use std::any::Any;
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::Selector;
use kivact_runtime::{Binding, ErrorKind, Handler, UiError};
use kivact_runtime::{Owner, Runtime};
use kivact_runtime::{Signal, Value};
use kivact_types::{
    DismissReason, Environment, FocusTarget, Modifiers, NodeId, PlatformResult, WindowId,
    WindowMode,
};

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

    /// Whether the environment last reported asks for reduced motion.
    pub(crate) fn reduced_motion(&self) -> bool {
        self.env.with_untracked(|env| env.reduced_motion)
    }

    /// The environment last reported.
    pub(crate) fn environment(&self) -> Environment {
        self.env.get_untracked()
    }

    /// Whether the last reported surface was active.
    pub(crate) fn surface_active(&self) -> bool {
        self.surface_active.get_untracked()
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

/// Everything the view layer keeps per App. Fields drop in declaration
/// order.
pub(crate) struct ViewState {
    /// The retained metadata and direct engine access of the whole App.
    pub(super) tree: RefCell<ViewNodes>,
    /// Removed node ids awaiting host revision cleanup.
    pub(crate) removed: RefCell<Vec<NodeId>>,
    /// The open windows' records.
    pub(super) windows: RefCell<BTreeMap<WindowId, WindowRecord>>,
    // xtask-allow(state-fields): S-8 ID - the next window id
    /// The last window id handed out (`WindowId::MAIN` is 1).
    next_window: Cell<u64>,
    /// What the host's platform can do (host events alone write it).
    #[allow(
        dead_code,
        reason = "the capabilities memo is the only reader until a host event writes it"
    )]
    pub(super) host_capabilities: Signal<Capabilities>,
    pub(crate) capabilities: Value<Capabilities>,
    pub(crate) fonts_value: Signal<Vec<String>>,
    // xtask-allow(state-fields): independent fact - the App has not drawn yet
    pub(crate) initial_render: Signal<bool>,
    /// The App-wide motion clock input.
    pub(crate) app_motion: Signal<AppMotion>,
    /// Missing capabilities the App relied on and that were noted.
    noted_missing: kivact_types::cell::ReportOnceSet<&'static str>,
    /// Whether the last key or pointer press the App got was a pointer press
    /// (`NodeRef::focus` then asks for a focus without a visible indicator).
    pointer_modality: Cell<Modality>,
    /// The modifier keys the host reported last (`view::modifiers`).
    modifiers: Signal<Modifiers>,
    /// Focus / platform requests waiting for the host.
    pub(super) requests: RefCell<Requests>,
}

/// One window's view record: the owner that declared it, its environment,
/// focus source, pointer observations and open state.
#[derive(Clone)]
pub(crate) struct WindowRecord {
    /// The owner that declared the window (the root owner for the main one).
    pub(crate) declared_in: Owner,
    pub(crate) env: Rc<EnvState>,
    pub(crate) focus: FocusKeys,
    pub(crate) hover: Cell<PointerObservation<FocusTarget>>,
    pub(crate) mouse: Cell<PointerObservation<crate::geometry::Point>>,
    #[allow(
        dead_code,
        reason = "the record keeps the window's title for DevTools and diagnostics"
    )]
    pub(crate) title: Value<String>,
    // xtask-allow(state-fields): construction option - Window::min_size
    pub(crate) min_size: Option<kivact_types::Size>,
    pub(crate) dismiss: Option<Handler<DismissReason>>,
    pub(crate) open: Binding<bool>,
    #[allow(
        dead_code,
        reason = "the record keeps the window's mode for later input routing diagnostics"
    )]
    pub(crate) mode: WindowMode,
}

/// The window a subtree belongs to.
#[derive(Clone, Copy)]
pub(crate) struct WindowScope(pub(crate) WindowId);

/// The window of `owner`'s subtree, or `WindowId::MAIN` outside one.
pub(crate) fn current_window(owner: Owner) -> WindowId {
    owner
        .context::<WindowScope>()
        .map_or(WindowId::MAIN, |scope| scope.0)
}

/// The App's motion clock input: whether any window can draw and the tick
/// source's frame period.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct AppMotion {
    pub(crate) drawable: bool,
    pub(crate) frame_period_ns: u64,
}

impl ViewState {
    /// Construct the complete state before mounting any application code.
    pub(crate) fn new(
        owner: Owner,
        info: &kivact_engine::contract::HostInfo,
        engine: kivact_engine::api::Writer,
        at: &'static Location<'static>,
    ) -> Self {
        let host_capabilities = owner.signal(info.capabilities, at);
        let capabilities = crate::view::devtools_hooks::wrap_capabilities(owner, host_capabilities);
        Self {
            tree: RefCell::new(ViewNodes::new(engine, owner)),
            removed: RefCell::new(Vec::new()),
            windows: RefCell::new(BTreeMap::new()),
            next_window: Cell::new(1),
            host_capabilities,
            capabilities,
            fonts_value: owner.signal(Vec::new(), at),
            initial_render: owner.signal(true, at),
            app_motion: owner.signal(
                AppMotion {
                    drawable: info.surface_active,
                    frame_period_ns: info.env.frame_period_ns,
                },
                at,
            ),
            noted_missing: Default::default(),
            pointer_modality: Cell::new(Modality::Keyboard),
            modifiers: owner.signal(Modifiers::default(), at),
            requests: RefCell::new(Requests::default()),
        }
    }

    /// Insert the record of `window`.
    pub(crate) fn install_window(&self, window: WindowId, record: WindowRecord) {
        self.windows.borrow_mut().insert(window, record);
    }

    /// Remove and return the record of `window`.
    pub(crate) fn remove_window(&self, window: WindowId) -> Option<WindowRecord> {
        self.windows.borrow_mut().remove(&window)
    }

    /// The next window id (`WindowId::MAIN` is 1 and never reused).
    pub(crate) fn next_window(&self) -> WindowId {
        let raw = self.next_window.get() + 1;
        self.next_window.set(raw);
        WindowId::new(raw).expect("invariant: window ids start at one")
    }

    /// A copy of `window`'s record.
    pub(crate) fn window_record(&self, window: WindowId) -> Option<WindowRecord> {
        self.windows.borrow().get(&window).cloned()
    }

    /// Run `f` with `window`'s record.
    pub(crate) fn with_window<R>(
        &self,
        window: WindowId,
        f: impl FnOnce(&WindowRecord) -> R,
    ) -> Option<R> {
        self.windows.borrow().get(&window).map(f)
    }

    /// Run `f` with `window`'s record for change.
    pub(crate) fn with_window_mut<R>(
        &self,
        window: WindowId,
        f: impl FnOnce(&mut WindowRecord) -> R,
    ) -> Option<R> {
        self.windows.borrow_mut().get_mut(&window).map(f)
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

    /// The environment of `window`.
    #[inline]
    pub(crate) fn env(&self, window: WindowId) -> Option<Rc<EnvState>> {
        self.with_window(window, |record| record.env.clone())
    }

    /// The window's minimum size (`None`: no minimum).
    pub(crate) fn min_window_size(&self, window: WindowId) -> Option<kivact_types::Size> {
        self.with_window(window, |record| record.min_size).flatten()
    }

    /// What the host's platform can do.
    pub(crate) fn capabilities(&self) -> Capabilities {
        self.capabilities.get_untracked()
    }

    /// Whether the missing capability `name` has not been noted yet (it
    /// is noted from now on).
    pub(crate) fn note_once_missing(&self, name: &'static str) -> bool {
        self.noted_missing.note(name)
    }

    /// The keyboard focus source of `window`.
    #[inline]
    pub(crate) fn focus_keys(&self, window: WindowId) -> Option<FocusKeys> {
        self.with_window(window, |record| record.focus)
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

    /// The node the host reported under the pointer last in `window`.
    pub(crate) fn hover(&self, window: WindowId) -> PointerObservation<FocusTarget> {
        self.with_window(window, |record| record.hover.get())
            .unwrap_or(PointerObservation::Plain(None))
    }

    /// Replace `window`'s hover record (a report, or the first `NodeRef::hovered`).
    pub(crate) fn set_hover(&self, window: WindowId, hover: PointerObservation<FocusTarget>) {
        self.with_window_mut(window, |record| record.hover.set(hover));
    }

    /// The mouse position of `window`.
    pub(crate) fn mouse(&self, window: WindowId) -> PointerObservation<crate::geometry::Point> {
        self.with_window(window, |record| record.mouse.get())
            .unwrap_or(PointerObservation::Plain(None))
    }

    /// Replace `window`'s mouse record.
    pub(crate) fn set_mouse(
        &self,
        window: WindowId,
        mouse: PointerObservation<crate::geometry::Point>,
    ) {
        self.with_window_mut(window, |record| record.mouse.set(mouse));
    }
}

#[derive(Clone, Copy)]
enum Modality {
    Pointer,
    Keyboard,
}

impl FocusKeys {
    /// Create the focus source and its selector under the window's `owner`.
    pub(crate) fn new(owner: Owner, at: &'static Location<'static>) -> FocusKeys {
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
