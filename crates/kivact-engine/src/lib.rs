//! Role: The engine under `kivact`: a retained tree run in an Arvane window.
//!
//! The client writes typed node properties directly into one retained tree.
//! The engine lays out that tree and runs it in an Arvane window.
//!
//! - `bridge`: owned [`bridge::Writer`] and [`bridge::Reader`] handles,
//!   concrete [`bridge::Client`] callbacks and wakers.
//! - `model`: the single model (nodes, pending changes, one state group
//!   per concern), tree mutations, focus and Tab order, modal confinement,
//!   pointer input, text editing, state styles, semantics and the event queue.
//! - `tree`: authoritative structural topology and passive typed properties.
//! - `layout`: the normal borrowed-tree trait adapter to kivact-layout.
//! - `measure`: resource measurement and retained editor payloads.
//! - `model::interact`: derived synchronization, geometry and visual processing.
//! - `paint`: the culled paint list and tree-aware frame assembly.
//! - `a11y`: full accessibility construction; publication compares the
//!   candidate with the last accepted tree and sends only changed nodes.
//! - `check`: text layout findings requested by the test driver or debug lint.
//! - `host`: the Arvane window loop, pacing, assets, the typed write API
//!   (one table of props), concrete client callbacks and the production
//!   test driver.
//!
//! Protocol values and pure motion live in `kivact-wire`; value lowering,
//! local scene adaptation and resources live in `kivact-lower`; retained
//! hard lines and lazy text queries live in `kivact-text`. Borrowed-tree
//! computation and derived layout column types live in `kivact-layout`;
//! Model owns those columns, editors, assets and the other derived state.
//!
//! Each state group keeps its state in a leaf `state` module under the
//! module that changes it, with fields visible to that group only.
//!
//! Arvane owns rendering, presentation, GPU fallback and surface recovery.
//! No `arvane-*` crate depends on a Kivact crate.
//!
//! Public are [`run_native`], [`native`] (its [`HostOptions`], also
//! re-exported here), [`bridge`] (the owned handles,
//! the client callbacks, `Offscreen`, `TextMeasurer` and
//! `Prop`), and `testing` with the
//! feature `testing`; everything else is `pub(crate)`. Geometry integration,
//! hit testing, scrolling and scenes share the model's positions. The model
//! decides what the tree
//! means; `paint`, `a11y` and `host` only map its state onto Arvane.
//!
//! # Invariants
//!
//! Feature `embed` exposes experimental host-window embedding through `embed`.
//!
//! - The client API exposes no Taffy, Unicode segmentation or native window
//!   implementation. Geometry and semantics share kivact-types value types.
//! - Mutations apply immediately. Comparisons record only changed fields;
//!   `Model::sync` folds derived work once at a commit.
//! - Every host event enters one FIFO. Private commit boundaries delimit
//!   prefixes delivered after layout, with model borrows released. Mounts
//!   run after their prefix; cached image failures stay after the boundary.
//! - Model and native callbacks stay on the UI thread. Only wakers cross
//!   threads. Each Writer or Reader call borrows shared state for that call.
#![forbid(unsafe_code)]

#[cfg(feature = "embed")]
pub mod embed;

mod a11y;
pub mod bridge;
#[cfg(any(debug_assertions, feature = "testing"))]
mod check;
mod frame_log;
mod host;
mod layout;
mod measure;
mod model;
mod mutation;
pub mod native;
mod offscreen;
mod paint;
mod text_measure;
mod tree;

pub use host::run_native;
#[cfg(feature = "testing")]
pub use host::testing;
pub use native::HostOptions;

pub(crate) use frame_log::{FrameLog, FrameTimes};
pub(crate) use model::keys::KeyResult;
pub(crate) use model::pointer::PointerSpace;
pub(crate) use model::semantics::{is_list_role, semantic_role};
