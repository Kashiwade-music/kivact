//! Role: Reactive runtime, async tasks, and frame-driven motion for Kivact apps.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::type_complexity,
    reason = "handler and effect closures are stored as boxed trait objects throughout"
)]

mod host;
mod machine;
mod motion;
mod reactive;
mod task;

pub use host::RuntimeConfig;
pub use machine::{Machine, MachineCx, MachineHandle, Step, machine};
pub use motion::{
    Animatable, Animated, Easing, FrameInfo, FrameLoop, FrameRate, MotionEnv, MotionState,
    MotionStats, Repeat, Retarget, Spring, Transition, animated, frame_loop,
};
pub use reactive::{
    AppId, Binding, Callback, EffectKind, ErrorKind, ExitState, Extensions, Handled, Handler,
    HandlerResult, Host, IntoBinding, IntoReactive, IntoText, IntoValue, ItemKey, Kind, LiveCounts,
    NodeId, NodeInfo, Notifier, Owned, Owner, OwnerId, Pending, Prop, PropValue, Reactive, Result,
    Runtime, Selector, Signal, Stage, ThreadWaker, TraceRun, TraceWrite, Turn, TurnGuard,
    TurnRecord, TurnReport, UiError, UiThreadOnly, Value, Var, binding, callback, clock_now,
    effect, effect_on, memo, notifier, on_cleanup, on_mount, pure, raise, require_effectful,
    require_setup, selector, signal, try_use_context, untrack, use_context, var,
};
pub use reactive::{handler, keyed};
pub use task::{
    Action, ActionFn, ActionMode, ActionOutcome, AsyncState, BlockingWork, CancelToken, Completion,
    DEFAULT_DRAIN_NS, Resource, Task, TaskState, TaskStats, action, blocking, resource, sleep,
    spawn, spawn_in, timeout,
};

#[cfg(feature = "tokio")]
pub use task::on_tokio;
pub use task::{AsyncSource, Retry};
// xtask-allow(internal-contract): Standard async views consume sealed source publication identities.
#[doc(hidden)]
pub use task::AsyncSnapshot;
