//! Role: Public async names of the facade: tasks, resources and actions.
//!
//! The implementation is the `kivact-runtime` crate; this module re-exports the
//! names an app or a component author calls (`spawn`, `sleep`, `resource`,
//! `action`, ...). The driver API of the host that runs the App's turns
//! (`TaskState`, `Completion`, ...) stays in `kivact_runtime` and is not part of
//! the public facade.

#[doc(inline)]
pub use kivact_runtime::{
    Action, ActionFn, ActionMode, ActionOutcome, AsyncState, BlockingWork, CancelToken, Resource,
    Task, action, blocking, resource, sleep, spawn, timeout,
};

#[cfg(feature = "tokio")]
pub use kivact_runtime::on_tokio;
pub use kivact_runtime::{AsyncSource, Retry};
// xtask-allow(internal-contract): Standard async views consume sealed source publication identities.
#[doc(hidden)]
pub use kivact_runtime::AsyncSnapshot;
