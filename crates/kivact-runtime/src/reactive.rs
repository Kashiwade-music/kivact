//! Role: Reactive runtime and public handle types for Kivact apps.
//!
//! Kivact internal crate: the single reactive runtime of an App (arena,
//! graph, owners, turn), the public handle types, the setup functions
//! (`signal`, `memo`, `effect`, `effect_on`, `context`, ...), errors, the trace
//! ring, `IntoValue` and per-App extensions. The runtime knows no host:
//! the host advances its clock and runs the turns.
//!
//! The API has three audiences:
//! - applications and component authors, through the `kivact` facade: the
//!   handles ([`Signal`], [`Value`], [`Var`], [`Callback`], [`Binding`],
//!   [`Selector`]), the setup functions, [`IntoValue`] and [`UiError`];
//! - the layers above (view, task, motion, style): [`Owner`] and [`Owned`]
//!   for owners and the objects stored under them, [`Notifier`], [`Prop`],
//!   [`keyed`], [`pure`], [`require_setup`] and [`require_effectful`];
//! - the host that runs an App's turns: [`Runtime`] and [`Host`].
//!
//! [`Runtime`] is defined in `data`; each module adds the methods of one
//! concern: `registry` (the thread-local App registry and handle lookup),
//! `frame` (the frame stack: tracking, purity, setup, owners), `graph`
//! (nodes and edges, push-pull), `owner` (owners and disposal), `object`
//! (owner-bound objects), `report` (error routing), `turn` (effect
//! scheduling), `mounts` (`on_mount`), `clock` (clock and turn requests),
//! `ext` (per-App extensions), `diag` (the trace ring), `host` (the host's
//! entry points).

mod arena;
mod clock;
mod data;
mod diag;
mod error;
mod execution;
mod ext;
mod frame;
mod graph;
pub mod handler;
mod handles;
mod host;
pub(crate) mod host_types;
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

pub use clock::clock_now;
pub use data::{
    Binding, Callback, EffectKind, Extensions, Kind, LiveCounts, NodeInfo, Runtime, Signal,
    TraceRun, TraceWrite, Turn, TurnGuard, TurnRecord, UiThreadOnly, Value, Var,
};
pub use error::{ErrorKind, Result, UiError, raise};
pub use frame::{pure, require_effectful, require_setup, untrack};
pub(crate) use graph::watch;
pub use handler::{Handled, Handler, HandlerResult};
pub use handles::IntoBinding;
pub use host_types::{ExitState, Host, Pending, Stage, ThreadWaker, TurnReport};
pub use identity::{AppId, NodeId, OwnerId};
pub use owner_handle::{Owned, Owner};
pub use props::{IntoReactive, IntoText, IntoValue, ItemKey, Prop, PropValue, Reactive};
pub use selector::{Selector, selector};
pub use setup::{
    binding, callback, constant, context, effect, effect_on, memo, notifier, on_cleanup, on_mount,
    signal, try_context, var,
};
pub use trigger::Notifier;
