//! Role: Playwright-like Testing Library over real windows (feature `test`).
//!
//! `render` mounts a root on the engine's in-process test driver: the
//! production host with its real layout, text shaping, rendering and
//! accessibility tree, shown in a real window (headless where none can
//! open, or with `KIVACT_TEST_MIRROR=off`), on a virtual clock. Queries by
//! role read the accessibility tree the host publishes; interactions are
//! real input (pointer, keys, IME, wheel, accessibility actions) checked
//! for actionability. A failing test leaves its last frame and trees in
//! [`failure_dir`].
//!
//! The test windows share one event loop per process: a test binary that
//! renders screens must not also call `App::run`.
//!
//! Tests are end to end only, as Playwright's are: the real layout,
//! shaping, drawing, pixels and accessibility, on a virtual clock. There is
//! no mock host, no host conformance suite and no unit test of internal
//! modules.
//!
//! - Settling: `render` mounts the root and settles. A settle runs turns
//!   while the host has something due, a worker woke it or the core has
//!   work (a requested turn, a ready task, an effect), then waits in real
//!   time, at most 5 s, for blocking jobs (a worker's wake unparks the test
//!   thread). [`Screen::advance`] sets the clock to each next deadline and
//!   settles; [`Screen::wait_for`] and the `find_by_*` queries step virtual
//!   time one display frame at a time (5 s by default).
//! - Drawing: every settle draws requested frames and then publishes the
//!   accessibility tree, following the production redraw path.
//! - Options: [`RenderOptions`] defaults to 800 × 600 at scale 1, the
//!   bundled font only, a real window (headless when none opens), the CPU
//!   and 60 Hz. The Cargo GPU feature only enables the capability; a test
//!   picks how it draws with `gpu` (no API says which one drew).
//!   The default reads `KIVACT_TEST_MIRROR` (`off`, `auto`, or `required`).
//!   [`RenderOptions::from_env`] also reads `KIVACT_TEST_GPU=1`.
//!   [`render_with_handle`] also returns values made by the root setup.
//! - Diagnostics: a failure puts the query, every candidate, the focus, the
//!   recent actions and turns, the errors, the published accessibility tree
//!   and the core's semantic tree in one message. Debug builds check at
//!   every settle, the first mount included, that the core's node records
//!   are the engine's live tree (every node the root reaches has one, and
//!   the core keeps no record the root cannot reach; topology and props live
//!   only in the engine's tree, so they are not compared).
//! - Panics: a panic in App code (a handler, effect, memo, setup, task,
//!   `blocking` job or a detected misuse) is not caught. It comes out of the
//!   [`Screen`] call that drove the turn with its original payload, and the
//!   failure artifacts are written. Test a detected misuse with
//!   `#[should_panic(expected = "..")]`.
//! - Layout check: [`Screen::check_layout`], `check_layout_with` and
//!   `Element::check_layout` turn the engine's findings into
//!   [`LayoutViolation`]s; `assert_layout` fails with all of them and keeps
//!   the frame in [`failure_dir`]. [`Screen::assert_layout_with`] preserves
//!   explicit check options and the same failure artifacts. A Japanese label needs Noto Sans JP given
//!   with `TestFonts::Only` (the bundled font has no CJK). The E2E is
//!   `dev/tests/kivact-public-api/tests/e2e/layout_check.rs`.

//!
//! Embedded editors can use `TestParent` to open a real host, inject native
//! input, pump the host message loop, and capture the child's presented pixels.
//!
//! ```no_run
//! # #[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
//! # fn example() -> kivact_core::Result<()> {
//! use kivact_core::{App, Result};
//! use kivact_core::view::{NodeBuilder, text};
//! use kivact_test::TestParent;
//! let parent = TestParent::open(320, 240)?;
//! let editor = App::new("Editor").window_size(320, 240)
//!     .embed(parent.parent(), || text("Editor"))?;
//! parent.pump(std::time::Duration::from_millis(50));
//! assert_eq!(parent.capture()?.width, 320);
//! drop(editor);
//! # Ok(())
//! # }
//! ```

// `render` drives an `AppClient` on the engine's in-process test driver
// (`kivact_engine::testing`), mirrored to a real window; queries by
// text and test id read the retained tree, input enters at the host's entry.

#![allow(
    clippy::type_complexity,
    reason = "handler and effect closures are stored as boxed trait objects throughout"
)]

mod options;
mod script;
pub use script::Script;
mod facts;
pub mod raw;
mod render;
mod screen;

pub use options::{MirrorMode, RenderOptions, TestFonts};
pub use render::{gpu_unavailable, native_unavailable, render, render_with, render_with_handle};

pub use screen::{
    Contact, Element, ImeText, LayoutCheck, LayoutLimit, LayoutRule, LayoutViolation, Screen,
    Screenshot, SemanticNode, Within, failure_dir,
};

// xtask-allow(os-branch): Native test environment availability is a test platform boundary (design 17.8).
#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
mod parent;
// xtask-allow(os-branch): Native test environment availability is a test platform boundary (design 17.8).
#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
pub use parent::TestParent;
