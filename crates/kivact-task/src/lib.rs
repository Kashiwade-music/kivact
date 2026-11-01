//! Role: Async task execution and lifecycle.
//!
//! Async: local executor, blocking pool, resources, actions (with their
//! queue policies) and host-clock time. Futures run on the UI thread and
//! may write signals after `await` directly.
//!
//! There are four async entries: [`resource`], [`action`] (one constructor
//! for runs with and without an input, so a run without one is `run(())`),
//! [`spawn`] and [`blocking`]. Fetchers and
//! runners return `Result<T, UiError>`; `?` converts other errors. A
//! spawned future returns `()` or `Result<(), UiError>`, and an `Err`
//! reaches the `on_error` of the task's owner like a handler's `Err`. A
//! task is an object of its owner: disposing the owner cancels the future.
//! External async channels work with `spawn`, so an App's renderer can
//! deliver `Pixels` frames from a worker without any input.
//!
//! [`Resource`] and [`Action`] keep their state in one signal of
//! [`AsyncState<T>`](AsyncState): `Idle`, `Pending`, `Ready(T)`,
//! `Failed(UiError)`. They are read through `state()` (`is_pending()` tells
//! whether one runs) and `latest()` (the last success, kept while `Pending`
//! and `Failed`). A cancellation is `Failed` with an `ErrorKind::Cancelled`
//! error (`UiError::is_cancelled`), and `on_done` is not called for it. An
//! action's `input()` and `attempt()` read the run its state describes. A
//! resource publishes only its latest request (by generation); an action's
//! runs follow its [`ActionPolicy`] (`action`).
//!
//! The clock is the host's: the client (`HostLink`) owns it and lends it to
//! the runtime as its driver, and timers, frame loops and animations all
//! read `Runtime::now_ns()`. The `kivact::test` host runs a virtual clock.

#![allow(
    clippy::type_complexity,
    reason = "handler and effect closures are stored as boxed trait objects throughout"
)]

mod action;
mod async_state;
mod blocking;
mod completion;
mod detached;
mod drain;
mod executor;
mod resource;
mod spawn;
mod state;
mod time;

pub use action::{Action, ActionFn, ActionPolicy, action};
pub use async_state::AsyncState;
pub use blocking::{BlockingWork, CancelToken, blocking};
pub use completion::Completion;
pub use detached::detached_threads;
pub use drain::DEFAULT_DRAIN_NS;
pub use resource::{Resource, resource};
pub use spawn::{Task, spawn, spawn_in};
pub use state::{TaskState, TaskStats};
pub use time::{sleep, timeout};
