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

// xtask-allow(internal-contract): the host driver contract of kivact-core and kivact-test; applications use the facade
#[doc(hidden)]
pub mod driver;

pub use machine::{Machine, MachineCx, MachineHandle, Step, machine};
pub use motion::{
    Animatable, Animated, At, Easing, FrameInfo, FrameLoop, FrameRate, Keyframe, Repeat, Retarget,
    Segment, Spring, Stagger, StaggerFrom, StepPosition, Timeline, TimelineTrack, Transition,
    animated, frame_loop, timeline,
};
pub use reactive::{
    AppId, Callback, Cause, CauseTag, ErrorKind, Handled, Handler, HandlerResult, IntoHandler,
    IntoText, IntoValue, ItemKey, NodeId, Owned, Owner, OwnerId, Prop, PropValue, ReportKind,
    Reported, Result, Selector, Signal, Stage, StageSpan, TraceEdge, TraceRun, TraceWrite, Trigger,
    Turn, TurnRecord, UiError, UiThreadOnly, Value, Var, binding, callback, clock_now, effect,
    effect_on, memo, on_cleanup, on_mount, pure, raise, require_effectful, require_setup, selector,
    signal, trigger, try_use_context, untrack, use_context, var,
};
pub use reactive::{handler, keyed};
pub use task::{
    Action, ActionFn, AsyncState, BlockingWork, CancelToken, Completion, Remote, RemoteTrigger,
    Resource, Task, action, blocking, clock, queued_action, remote, remote_trigger, resource,
    sleep, spawn, timeout,
};

pub use task::AsyncSource;
#[cfg(feature = "tokio")]
pub use task::on_tokio;
