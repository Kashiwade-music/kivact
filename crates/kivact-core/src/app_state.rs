//! Role: The core's per-App state: one struct in the runtime's app slot.
//!
//! `Launch::start` builds it in the root's setup and gives it to
//! `Runtime::new` (`AppParts::state`); every subsystem reads its part
//! with one downcast (`AppState::of`).

use kivact_runtime::Runtime;

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
    pub(crate) close: crate::platform::close_request::CloseRequestState,
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
