//! Role: Public async names of the facade: tasks, resources and actions.
//!
//! The implementation is the `kivact-runtime` crate; this module re-exports the
//! names an app or a component author calls (`spawn`, `sleep`, `resource`,
//! `action`, ...). The driver API of the host that runs the App's turns
//! (`TaskState`, `Completion`, ...) stays in `kivact_runtime` and is not part of
//! the public facade.

#[doc(inline)]
pub use kivact_runtime::{
    Action, ActionFn, ActionPolicy, AsyncState, BlockingWork, CancelToken, Resource, Task, action,
    blocking, resource, sleep, spawn, timeout,
};
