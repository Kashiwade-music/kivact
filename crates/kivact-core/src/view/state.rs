//! Role: View state and reactive composition.
//!
//! Per-App state of the view layer (what the host's start established: the
//! retained metadata and engine writer, the environment signals, the keyboard
//! focus source; and the last hovered node, the pointer drag the overlay
//! policy tracks, outstanding host requests), installed as a runtime
//! extension when the App is created.

use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::{BTreeMap, HashMap};
use std::panic::Location;
use std::rc::Rc;

use kivact_runtime::Selector;
use kivact_runtime::{Binding, ErrorKind, Handler, UiError};
use kivact_runtime::{Owner, Runtime};
use kivact_runtime::{Signal, Value};
use kivact_types::cell::Deferred;
use kivact_types::{
    DismissReason, Environment, FocusTarget, Insets, Modifiers, NodeId, PlatformResult,
    PointerSettings, Size, ThemeHint, WindowId, WindowMode, WindowPlacement,
};

use super::tree::ViewNodes;
use kivact_runtime::Completion;
use kivact_types::Capabilities;

pub(crate) use super::ref_data::{Bound, NodeRefState, RefObservation, RefState};

/// Focus requests: those waiting for the next commit, in order, and those
/// the host has not answered yet.
#[derive(Default)]
pub(crate) struct FocusRequests {
    pub(crate) queued: Vec<(kivact_types::RequestId, FocusRequest)>,
    pub(crate) sent: HashMap<kivact_types::RequestId, FocusReply>,
}

/// One focus request waiting for the commit.
pub(crate) struct FocusRequest {
    pub(crate) target: kivact_types::FocusTarget,
    pub(crate) reply: FocusReply,
    /// The requested focus indicator.
    pub(crate) visible: bool,
}

pub(crate) struct FocusCompletion {
    pub(crate) owner: Owner,
    pub(crate) at: &'static Location<'static>,
}

// xtask-allow(state): env is the host's one report (written by `set` only) and surface_active is the surface's own report; neither follows from the other
/// The App's host environment: one signal holding the whole `Environment` the
/// client writes, and one memo per field a reader asked for, so a change of
/// one field notifies only the readers of that field.
pub(crate) struct EnvState {
    /// The root owner: the memos live as long as the App.
    owner: Owner,
    env: Signal<Environment>,
    pub(crate) surface_active: Signal<bool>,
    fields: EnvFields,
}

/// One memo per environment field, made when a reader first asks.
#[derive(Default)]
struct EnvFields {
    fullscreen: Deferred<Value<bool>>,
    reduced_motion: Deferred<Value<bool>>,
    scale_factor: Deferred<Value<f64>>,
    safe_area: Deferred<Value<Insets>>,
    window_size: Deferred<Value<Size>>,
    pointer: Deferred<Value<PointerSettings>>,
    keyboard_inset: Deferred<Value<Insets>>,
    theme_hint: Deferred<Value<ThemeHint>>,
    locale: Deferred<Value<Option<String>>>,
    restored_state: Deferred<Value<BTreeMap<String, Vec<u8>>>>,
}

impl EnvState {
    /// The environment `env` at start, as a signal of `owner`.
    pub(crate) fn new(
        owner: Owner,
        env: Environment,
        surface_active: bool,
        at: &'static Location<'static>,
    ) -> Self {
        Self {
            owner,
            env: owner.signal(env, at),
            surface_active: owner.signal(surface_active, at),
            fields: EnvFields::default(),
        }
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

    /// The memo of one field, made on its first use by any context (a memo,
    /// an effect, a handler): it re-runs its readers only when that field
    /// changed.
    fn memo_of<T: kivact_runtime::PropValue>(
        &self,
        slot: &Deferred<Value<T>>,
        f: fn(&Environment) -> T,
    ) -> Value<T> {
        let env = self.env;
        *slot.get_or_init(|| {
            self.owner
                .with_setup(|| kivact_runtime::memo(move || env.with(f)))
        })
    }

    /// The memo of the window's full-screen state.
    pub(crate) fn fullscreen(&self) -> Value<bool> {
        self.memo_of(&self.fields.fullscreen, |e| e.fullscreen)
    }

    /// The memo of the reduced-motion request.
    pub(crate) fn reduced_motion_value(&self) -> Value<bool> {
        self.memo_of(&self.fields.reduced_motion, |e| e.reduced_motion)
    }

    /// The memo of the scale factor.
    pub(crate) fn scale_factor(&self) -> Value<f64> {
        self.memo_of(&self.fields.scale_factor, |e| e.scale_factor)
    }

    /// The memo of the safe area.
    pub(crate) fn safe_area(&self) -> Value<Insets> {
        self.memo_of(&self.fields.safe_area, |e| e.safe_area)
    }

    /// The memo of the window size.
    pub(crate) fn window_size(&self) -> Value<Size> {
        self.memo_of(&self.fields.window_size, |e| e.window_size)
    }

    /// The memo of the pointer settings.
    pub(crate) fn pointer(&self) -> Value<PointerSettings> {
        self.memo_of(&self.fields.pointer, |e| e.pointer)
    }

    /// The memo of the software keyboard's inset.
    pub(crate) fn keyboard_inset(&self) -> Value<Insets> {
        self.memo_of(&self.fields.keyboard_inset, |e| e.keyboard_inset)
    }

    /// The memo of the system's color mode.
    pub(crate) fn theme_hint(&self) -> Value<ThemeHint> {
        self.memo_of(&self.fields.theme_hint, |e| e.theme_hint)
    }

    /// The memo of the system's locale tag.
    pub(crate) fn locale(&self) -> Value<Option<String>> {
        self.memo_of(&self.fields.locale, |e| e.locale.clone())
    }

    /// The memo of the state the platform restored at launch.
    pub(crate) fn restored_state(&self) -> Value<BTreeMap<String, Vec<u8>>> {
        self.memo_of(&self.fields.restored_state, |e| e.restored_state.clone())
    }
}

/// The keyboard focus of a window: the target and whether its indicator is
/// visible.
#[derive(Clone, Copy, PartialEq)]
pub(crate) struct KeyFocus {
    pub(crate) target: kivact_types::FocusTarget,
    pub(crate) visible: bool,
}

/// The reactive focus nodes of an App (created at root mount).
#[derive(Clone, Copy)]
pub(crate) struct FocusKeys {
    /// The focused node as last reported by the host, with the indicator it
    /// asked for; `None` while nothing is focused.
    pub(crate) focus: Signal<Option<KeyFocus>>,
    /// "Is this node focused", notifying only the previous and next node.
    pub(crate) selector: Selector<kivact_types::FocusTarget>,
}

/// How a focus request completes.
pub(crate) enum FocusReply {
    Report(FocusCompletion),
    Restore,
    RestoreAndReport(FocusCompletion),
}

/// Who waits for a host answer, by request id.
pub(crate) enum Waiter {
    /// Report a failure to the owner's error chain (the default sink
    /// without an owner), located at the call; a `Failed` answer is an
    /// error of kind `failed`.
    Report {
        owner: Option<Owner>,
        at: &'static Location<'static>,
        failed: ErrorKind,
    },
    /// Complete a future with the platform answer.
    Future(Completion<Result<PlatformResult, UiError>>),
    /// A size request of a foreign window's native child.
    ForeignSize {
        window: WindowId,
        at: &'static Location<'static>,
        slot: Completion<Result<(), UiError>>,
    },
    /// A CPU capture answered at the settled host boundary.
    Snapshot {
        owner: Owner,
        at: &'static Location<'static>,
        reply: Completion<Result<kivact_types::Pixels, UiError>>,
    },
}

/// Outstanding host requests of an App.
#[derive(Default)]
pub(crate) struct Requests {
    pub(crate) next_request: u64,
    pub(crate) focus: FocusRequests,
    /// The answers the host owes, by request id.
    pub(crate) waiters: HashMap<u64, Waiter>,
    /// Activations `NodeRef::activate` asked for, run by the client after
    /// the handler that asked.
    pub(crate) pending_activations: Vec<kivact_types::FocusTarget>,
    /// Actual capture releases awaiting the current pointer dispatch's cleanup.
    pub(crate) pointer_releases: Vec<(NodeId, kivact_types::PointerId)>,
}

impl Requests {
    pub(crate) fn request_id(&mut self) -> u64 {
        self.next_request += 1;
        self.next_request
    }
}

// xtask-allow(state): legacy: tree, removed, windows, next_window, fonts_value, initial_render, app_motion, pointer_modality, modifiers, requests; allocates
/// Everything the view layer keeps per App. Fields drop in declaration
/// order.
pub(crate) struct ViewState {
    /// The retained metadata and direct engine access of the whole App.
    pub(super) tree: RefCell<ViewNodes>,
    /// The DevTools hooks; None without DevTools. Fixed at construction.
    pub(crate) devtools: Option<Rc<dyn crate::view::devtools_hook_traits::DevtoolsHooks>>,
    /// Removed node ids awaiting host revision cleanup.
    pub(crate) removed: RefCell<Vec<NodeId>>,
    /// The open windows' records.
    pub(super) windows: RefCell<BTreeMap<WindowId, WindowRecord>>,
    /// The last window id handed out (`WindowId::MAIN` is 1).
    next_window: Cell<u64>,
    /// What the host's platform can do.
    pub(crate) capabilities: Value<Capabilities>,
    pub(crate) fonts_value: Signal<Vec<String>>,
    /// True until the first turn after the App's setup (written once, by start_mount).
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
    pub(crate) requests: RefCell<Requests>,
}

/// One window's view record: the owner that declared it, its declaration,
/// environment, focus source and pointer sources.
#[derive(Clone)]
pub(crate) struct WindowRecord {
    /// The owner that declared the window (the root owner for the main one).
    pub(crate) declared_in: Owner,
    /// The declaration: title, open binding, sizes, mode, placement,
    /// dismiss handler and the foreign hooks. Shared with the window slot.
    pub(crate) decl: Rc<WindowDecl>,
    pub(crate) env: Rc<EnvState>,
    pub(crate) focus: FocusKeys,
    /// The node under the pointer, as the host reports it.
    pub(crate) hover: Signal<Option<FocusTarget>>,
    /// The mouse position in the window.
    pub(crate) mouse: Signal<Option<crate::geometry::Point>>,
}

/// What declares a window. The main window's is built by the launch from
/// the App's settings.
pub(crate) struct WindowDecl {
    pub(crate) title: Value<String>,
    pub(crate) open: Binding<bool>,
    pub(crate) size: Size,
    pub(crate) min_size: Option<Size>,
    pub(crate) mode: WindowMode,
    pub(crate) placement: WindowPlacement,
    pub(crate) dismiss: Option<Handler<DismissReason>>,
    /// The native child machinery of a `foreign_window`.
    pub(crate) foreign: Option<Rc<super::foreign::ForeignHooks>>,
}

/// The window a subtree belongs to.
#[derive(Clone, Copy)]
pub(crate) struct WindowScope(pub(crate) WindowId);

/// The window of `owner`'s subtree. The App's root provides the main window's scope.
pub(crate) fn current_window(owner: Owner) -> WindowId {
    owner
        .context::<WindowScope>()
        .expect("invariant: every owner is under a WindowScope")
        .0
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
        devtools: Option<Rc<dyn crate::view::devtools_hook_traits::DevtoolsHooks>>,
        at: &'static Location<'static>,
    ) -> Self {
        let host_capabilities = owner.signal(info.capabilities, at);
        let capabilities = match &devtools {
            Some(h) => h.wrap_capabilities(owner, host_capabilities),
            None => host_capabilities.into(),
        };
        Self {
            tree: RefCell::new(ViewNodes::new(engine, owner, devtools.clone())),
            devtools,
            removed: RefCell::new(Vec::new()),
            windows: RefCell::new(BTreeMap::new()),
            next_window: Cell::new(1),
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

    /// Resolve the size requests of `window` with `ErrorKind::Closed`; its
    /// native window is going away.
    pub(crate) fn close_foreign_sizes(&self, window: WindowId) {
        let drained: Vec<Waiter> = {
            let mut requests = self.requests.borrow_mut();
            let ids: Vec<u64> = requests
                .waiters
                .iter()
                .filter(|(_, waiter)| matches!(waiter, Waiter::ForeignSize { window: w, .. } if *w == window))
                .map(|(id, _)| *id)
                .collect();
            ids.into_iter()
                .filter_map(|id| requests.waiters.remove(&id))
                .collect()
        };
        for waiter in drained {
            if let Waiter::ForeignSize { at, slot, .. } = waiter {
                slot.complete(Err(UiError::new(
                    ErrorKind::Closed,
                    "the foreign window closed before the size request was answered",
                    Some(at),
                )));
            }
        }
    }

    /// Release spare request/tree capacity. Environment memos and live
    /// reactive/user resources are authority, not rebuildable caches.
    pub(crate) fn release_capacity(&self) {
        self.tree.borrow_mut().release_capacity();
        let mut requests = self.requests.borrow_mut();
        requests.focus.queued.shrink_to_fit();
        requests.focus.sent.shrink_to_fit();
        requests.waiters.shrink_to_fit();
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
        self.with_window(window, |record| record.decl.min_size)
            .flatten()
    }

    /// Write `false` to the `open` binding of every foreign window that is
    /// still open, as the App's exit turn does. Owners are disposed by the
    /// next stabilization, so `on_destroy` runs while the child is alive.
    pub(crate) fn close_foreign_for_exit(&self) {
        let open: Vec<Rc<WindowDecl>> = self
            .windows
            .borrow()
            .values()
            .filter(|record| record.decl.foreign.is_some())
            .map(|record| record.decl.clone())
            .collect();
        for decl in open {
            let released = decl
                .foreign
                .as_ref()
                .is_some_and(|hooks| hooks.phase.get() == super::foreign::ForeignPhase::Released);
            if !released && decl.open.get_untracked() {
                decl.open.set(false);
            }
        }
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
}

#[derive(Clone, Copy)]
enum Modality {
    Pointer,
    Keyboard,
}

impl FocusKeys {
    /// Create the focus source and its selector under the window's `owner`.
    pub(crate) fn new(owner: Owner, at: &'static Location<'static>) -> FocusKeys {
        let focus: Signal<Option<KeyFocus>> = owner.signal(None, at);
        // `selector` registers under the setup owner, and follows the target
        // only: a new indicator for the same target changes nothing.
        let targets =
            owner.with_setup(|| kivact_runtime::memo(move || focus.get().map(|key| key.target)));
        let selector = owner.with_setup(|| kivact_runtime::selector(targets));
        FocusKeys { focus, selector }
    }
}

impl super::tree::Removed {
    pub(crate) fn commit(self, rt: &kivact_runtime::Runtime) {
        ViewState::of(rt).removed.borrow_mut().extend(self.ids);
    }
}
