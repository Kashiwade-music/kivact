//! Role: The core side of the engine boundary.
//! [`AppClient`] assembles the
//! runtime of one App (its per-App state extensions), drives it (it owns
//! the host clock and waker), supplies concrete engine callbacks (the turn loop) and
//! delivers host events. Effects write directly through the retained tree
//! (`view::tree`) using the engine's owned `Writer`.
//!
//! `App::run` hands an `AppClient` to the engine, and the Testing
//! Library drives one over the engine's test driver. This module never
//! names the engine. Exit events use the task drain deadline directly, so
//! event delivery does not depend on this parent reexport module.
//!
//! The engine's `bridge::Client` has two callbacks: `start` (the host info,
//! the waker and the owned `Writer`) and `turn` (every queued event, the
//! commit boundaries among them and the host clock). Only the sharing of
//! the `AppClient` between them uses `Rc<RefCell<_>>`; the `Writer` and the
//! `Reader` are not wrapped again.
//!
//! Which turn settles which event:
//! - The host always lays out before a turn and passes every waiting event
//!   in one turn, with the commit boundaries among them (`Boundary`: up to
//!   which event the writes of which revision produced the events). A
//!   boundary only decides when the `on_mount` callbacks of its revision
//!   may run; it does not change the rules of the turn.
//! - Events that a client write produced inside the engine before its
//!   commit (`FocusChanged` and `FocusResult` of a focus request, ...)
//!   arrive in the turn after that revision's layout, before its boundary,
//!   and the revision's `on_mount` callbacks run after them. Events a
//!   layout produces (`LayoutChanged`, `ScrollChanged`, `LayoutWarning`)
//!   arrive in the turn after that layout, after the boundary. Input and
//!   wakes arrive in the same turn or the next.
//! - In every turn, the writes of the handlers, the effects and tasks they
//!   cause and their commit settle in that same turn (never waiting for the
//!   next turn or a wake). Boundaries delivered after the same layout make
//!   one turn, so a focus move that went on in the previous turn (the focus
//!   of a removed node moving to the next tab stop, then back by an
//!   effect's request) is seen by the effects in its final state.
//!   Boundaries that carry no event and release no `on_mount` make no turn.
//! - When 64 turns in a row of one host pump still leave events, the host
//!   leaves the rest to its next wake and sends `HostEvent::TurnsCut`,
//!   which the client reports to the default sink as a `Cycle` (`dispatch`).
//! - A turn requested during a turn (a handler's write, ...) wakes nobody:
//!   the turn settles it, and asks again at its end only for work left
//!   over (user effects, ready tasks, due `on_mount` callbacks; `link`).

#![allow(
    clippy::type_complexity,
    reason = "the root builder is stored as a boxed trait object"
)]

pub(crate) mod app_client;
mod dispatch;
mod link;
mod notice;
mod phases;
mod settle;
pub(crate) mod state;

pub(crate) use app_client::AppClient;
#[cfg(feature = "test")]
pub(crate) use state::HostState;
