#![deny(clippy::too_many_lines, clippy::disallowed_types)]
//! Role: Core UI contracts, application setup and runtime implementation.
//!
//! The implementation behind the `kivact` facade owns App setup, views,
//! reactive resources, rendering contracts and platform requests.
//!
//! Components are plain functions that take arguments, return a view and
//! run once. State lives in an App-owned arena; you hold Copy handles
//! (`Signal`, `Value`, `Var`, `Callback`, ...) created by the setup
//! functions (`signal`, `memo`, `effect`, ... like Solid's `createSignal`,
//! `createMemo`, `createEffect`) or by `Type::new`. State belongs to the
//! nearest structural boundary (root, flow branch, list row, `provide`,
//! `error_boundary`). Dynamic properties are closures or handles; each
//! becomes a small render effect whose changed output is written into the
//! engine's tree as a diff.
//!
//! Standard components are available through `kivact::ui::button` in the facade.
//!
//! ```no_run
//! # use kivact_core as kivact;
//! use kivact::App;
//! use kivact::state::{signal, Signal};
//! use kivact::view::{IntoView, Role, column, node, text};
//!
//! fn counter(initial: i32) -> impl IntoView {
//!     let count = signal(initial);
//!     column((
//!         text(move || format!("count: {}", count.get())),
//!         node()
//!             .child(text("+1"))
//!             .role(Role::Button)
//!             .focusable(true)
//!             .on_press(move || count.update(|c| *c += 1)),
//!     ))
//! }
//!
//! fn main() -> kivact::Result<()> {
//!     App::new("Counter").run(|| counter(0))
//! }
//! ```

// No unsafe code: the two `allow`s are the declarations of the `unsafe fn`s
// `gpu_interop::import` and `ShareLink::provide` (contracts their callers
// keep; no unsafe block).
#![deny(unsafe_code)]

#[cfg(feature = "embed")]
pub mod embed;
#[cfg(not(feature = "embed"))]
#[allow(
    dead_code,
    unused_imports,
    unreachable_pub,
    reason = "the feature hides the public entry while keeping implementation paths compiled"
)]
mod embed;

mod app;
mod client;
pub mod custom;
pub mod diagnostics;
pub mod draw;
pub mod geometry;
#[cfg(feature = "gpu-interop")]
pub mod gpu_interop;
#[cfg(not(feature = "gpu-interop"))]
#[allow(
    dead_code,
    unused_imports,
    unreachable_pub,
    reason = "the feature hides the public entry while keeping implementation paths compiled"
)]
mod gpu_interop;
pub mod i18n;
pub mod launch;
pub mod motion;
pub mod platform;
pub mod state;
pub mod style;
pub mod task;
pub mod view;

pub use app::App;
#[doc(inline)]
pub use kivact_runtime::{ErrorKind, Result, UiError};
