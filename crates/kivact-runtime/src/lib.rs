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
    MotionStats, Repeat, Spring, Transition, animated, frame_loop,
};
pub use reactive::keyed;
pub use reactive::{
    AppId, Binding, Callback, EffectKind, ErrorKind, ExitState, Extensions, Handled, Handler,
    HandlerResult, Host, IntoReactive, IntoText, IntoValue, Kind, LiveCounts, NodeId, NodeInfo,
    Owned, Owner, OwnerId, Pending, Prop, Reactive, Result, Runtime, Selector, Signal, Stage,
    ThreadWaker, TraceRun, TraceWrite, Trigger, Turn, TurnGuard, TurnRecord, TurnReport, UiError,
    UiThreadOnly, Value, Var, binding, callback, clock_now, constant, context, effect, effect_on,
    memo, on_cleanup, on_mount, pure, raise, require_effectful, require_setup, selector, signal,
    try_context, untrack, var,
};
pub use task::{
    Action, ActionFn, ActionPolicy, AsyncState, BlockingWork, CancelToken, Completion,
    DEFAULT_DRAIN_NS, Resource, Task, TaskState, TaskStats, action, blocking, resource, sleep,
    spawn, spawn_in, timeout,
};
