//! Role: Host event delivery.
//! `dispatch` only routes each [`HostEvent`] to the module of its kind
//! (`pointer`, `key`, `text`, `a11y`, `env`, `platform`, `diagnostics`,
//! `gpu`); `common` holds what they share.
//!
//! Events for nodes that no longer exist are stale and
//! dropped (recorded in diagnostics). An asset failure is reported once per
//! node and message as `ErrorKind::Asset`, located where its node was
//! created.
//!
//! Modifier changes are only recorded (`view::modifiers`).
//!
//! While overlays are shown, the engine decides which events are admitted
//! (`Reader::admit_input`, asked once per event right before it is
//! dispatched), where key, pointer and wheel handler chains stop and which
//! overlay an unhandled Escape dismisses (`crate::view::overlays`); these
//! modules apply its answers. Events they do not admit are dropped and
//! counted like stale events.

mod a11y;
mod common;
mod delivery;
mod diagnostics;
mod env;
mod gpu;
mod ime;
mod key;
mod platform;
mod pointer;
mod text;

pub(crate) use delivery::dispatch;
pub(crate) use pointer::run_requested_activations;
