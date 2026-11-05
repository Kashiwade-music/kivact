#![deny(clippy::too_many_lines, clippy::disallowed_types)]
//! Role: Public Kivact facade that re-exports core, interaction and the standard UI.
//!
//! Components are plain functions that take arguments, return a view and
//! run once. State lives in an App-owned arena; you hold Copy handles
//! (`Signal`, `Value`, `Binding`, ...) created by the setup functions
//! (`signal`, `memo`, `binding`, `effect`, ... like Solid's `createSignal`,
//! `createMemo`, `createEffect`). Callbacks are plain closures. A component
//! author who stores a caller's closure keeps it as a Copy
//! [`state::Callback`] (`callback`), and builds behavior from the
//! [`interaction`] parts. App authors may also use `callback` to share
//! a handler between controls. State belongs to the
//! nearest structural boundary (root, flow branch, list row, `provide`,
//! `error_boundary`). Dynamic properties are closures or handles; each
//! becomes a small render effect whose changed output is written into the
//! engine's tree as a diff.
//!
//! Feature `embed` experimentally embeds an App in a host window (off by default).
//!
//! ```no_run
//! use kivact::prelude::*;
//!
//! fn counter(initial: i32) -> impl IntoView {
//!     let count = signal(initial);
//!     column((
//!         text(move || format!("count: {}", count.get())),
//!         button("+1").on_press(move || count.update(|c| *c += 1)),
//!     ))
//! }
//!
//! fn main() -> kivact::Result<()> {
//!     App::new("Counter").run(|| counter(0))
//! }
//! ```

#![forbid(unsafe_code)]

#[doc(inline)]
pub use kivact_interaction as interaction;
#[doc(inline)]
pub use kivact_ui as ui;

#[cfg(feature = "embed")]
#[doc(inline)]
pub use kivact_core::embed;
#[cfg(feature = "gpu-interop")]
#[doc(inline)]
pub use kivact_core::gpu_interop;
#[doc(inline)]
pub use kivact_core::{App, ErrorKind, Result, UiError};
#[doc(inline)]
pub use kivact_core::{
    custom, diagnostics, draw, geometry, i18n, motion, platform, state, style, task, view,
};
#[cfg(feature = "test")]
#[doc(inline)]
pub use kivact_test as test;

/// Everyday names and standard UI functions used by the introductory guides.
pub mod prelude;
