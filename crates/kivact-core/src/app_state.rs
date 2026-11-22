//! Role: The core's per-App state: one struct in the runtime's app slot.
//!
//! `Launch::start` builds it in the root's setup and gives it to
//! `Runtime::new` (`AppParts::state`); every subsystem reads its part
//! with one downcast (`AppState::of`). Each part reads its own `of`, an
//! inherent impl below, so the parts never name this struct's module.

use kivact_runtime::driver::Runtime;

use crate::client::state::HostState;
use crate::view::state::ViewState;

pub(crate) struct AppState {
    pub(crate) view: ViewState,
    pub(crate) gpu: crate::gpu_interop::StateSlot,
    pub(crate) handle: crate::platform::AppHandleExt,
    #[cfg(feature = "devtools")]
    pub(crate) devtools: crate::devtools::state::DevtoolsState,
    #[cfg(debug_assertions)]
    pub(crate) trace: crate::client::trace::Trace,
    pub(crate) lifecycle: crate::platform::lifecycle::LifecycleState,
    pub(crate) host: HostState,
    /// Runs an activation `NodeRef::activate` deferred (the client's
    /// `run_activation`; the view cannot name the client module).
    pub(crate) activate: fn(&Runtime, kivact_types::FocusTarget),
}

impl AppState {
    /// The core's state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &AppState {
        rt.app_state()
            .downcast_ref::<AppState>()
            .expect("invariant: Launch::start gives Runtime::new an AppState")
    }
}

impl ViewState {
    /// The view state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &ViewState {
        &AppState::of(rt).view
    }
}

impl HostState {
    /// The host-link state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &HostState {
        &AppState::of(rt).host
    }
}

impl crate::gpu_interop::StateSlot {
    /// The gpu-interop slot of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &Self {
        &AppState::of(rt).gpu
    }
}

impl crate::platform::AppHandleExt {
    /// The handle of `rt`'s App.
    pub(crate) fn of(rt: &Runtime) -> &Self {
        &AppState::of(rt).handle
    }
}

impl crate::platform::lifecycle::LifecycleState {
    /// The lifecycle subscriptions of `rt`'s App.
    pub(crate) fn of(rt: &Runtime) -> &Self {
        &AppState::of(rt).lifecycle
    }
}

#[cfg(debug_assertions)]
impl crate::client::trace::Trace {
    /// The comparison trace of `rt`'s App.
    pub(crate) fn of(rt: &Runtime) -> &Self {
        &AppState::of(rt).trace
    }
}

#[cfg(feature = "devtools")]
impl crate::devtools::state::DevtoolsState {
    /// The DevTools state of `rt`'s App.
    #[inline]
    pub(crate) fn of(rt: &Runtime) -> &Self {
        &AppState::of(rt).devtools
    }
}
