//! Role: Reactive runtime and public handle types for Kivact apps.
//!
//! Kivact internal crate: the single reactive runtime of an App (arena,
//! graph, owners, turn), the public handle types, the setup functions
//! (`signal`, `memo`, `effect`, `effect_on`, `context`, ...), errors, the trace
//! ring, `IntoValue` and per-App extensions. The runtime knows no host:
//! its driver (the client) supplies the clock and runs the turns.
//!
//! The API has three audiences:
//! - applications and component authors, through the `kivact` facade: the
//!   handles ([`Signal`], [`Value`], [`Var`], [`Callback`], [`Binding`],
//!   [`Selector`]), the setup functions, [`IntoValue`] and [`UiError`];
//! - the layers above (view, task, motion, style): [`Owner`] and [`Owned`]
//!   for owners and the objects stored under them, [`Trigger`], [`Prop`],
//!   [`keyed`], [`pure`], [`require_setup`] and [`require_effectful`];
//! - the host that runs an App's turns: [`Runtime`] and [`Driver`].
//!
//! [`Runtime`] is defined in `data`; each module adds the methods of one
//! concern: `registry` (the thread-local App registry and handle lookup),
//! `frame` (the frame stack: tracking, purity, setup, owners), `graph`
//! (nodes and edges, push-pull), `owner` (owners and disposal), `object`
//! (owner-bound objects), `report` (error routing), `turn` (effect
//! scheduling), `mounts` (`on_mount`), `driver` (clock and turn requests),
//! `ext` (per-App extensions), `diag` (the trace ring), `host` (the host's
//! entry points).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::type_complexity,
    reason = "handler and effect closures are stored as boxed trait objects throughout"
)]

mod arena;
mod data;
mod diag;
mod driver;
mod error;
mod execution;
mod ext;
mod frame;
mod graph;
mod handles;
mod host;
mod identity;
pub mod keyed;
mod mounts;
mod object;
mod owner;
mod owner_handle;
mod props;
mod registry;
mod report;
mod runtime;
mod scope;
mod sealed;
mod selector;
mod setup;
mod trigger;
mod turn;

pub use data::{
    Binding, Callback, Driver, EffectKind, Extensions, Kind, LiveCounts, NodeInfo, Runtime, Signal,
    TraceRun, TraceWrite, TurnRecord, UiThreadOnly, Value, Var,
};
pub use driver::clock_now;
pub use error::{ErrorKind, HandlerResult, Result, UiError, raise};
pub use frame::{pure, require_effectful, require_setup, untrack};
pub use host::{Mounts, Pending, TurnRecorder};
pub use identity::{AppId, NodeId, OwnerId};
pub use owner_handle::{Owned, Owner};
pub use props::{IntoReactive, IntoText, IntoValue, Prop, Reactive};
pub use selector::{Selector, selector};
pub use setup::{
    binding, callback, constant, context, effect, effect_on, memo, on_cleanup, on_mount, signal,
    try_context, var,
};
pub use trigger::Trigger;
