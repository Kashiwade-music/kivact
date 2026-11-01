//! Role: Playwright-like Testing Library over real windows (feature `test`).
//!
//! `render` mounts a root on the engine's in-process test driver: the
//! production host with its real layout, text shaping, rendering and
//! accessibility tree, shown in a real window (headless where none can
//! open, or with `ARVANE_TEST_HEADLESS=1`), on a virtual clock. Queries by
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
//! - Drawing: the accessibility tree and the layout are updated every
//!   turn, so queries and actions need no frame. A frame is drawn when it is
//!   looked at (a screenshot, a failure, `KIVACT_TEST_HOLD_MS`);
//!   `KIVACT_TEST_LIVE=1` draws one at every settle, to watch the window.
//! - Options: [`RenderOptions`] defaults to 800 × 600 at scale 1, the
//!   bundled font only, a real window (headless when none opens), the CPU
//!   and 60 Hz. The Cargo GPU feature only enables the capability; a test
//!   picks how it draws with `gpu` (no API says which one drew).
//!   [`RenderOptions::from_env`] keeps those defaults except for an explicit
//!   developer GPU/headless preference and otherwise requires a real window.
//!   [`render_with_handle`] also returns values made by the root setup.
//! - Diagnostics: a failure puts the query, every candidate, the focus, the
//!   recent actions and turns, the errors, the published accessibility tree
//!   and the core's semantic tree in one message. Debug builds check at
//!   every settle, the first mount included, that the core's host nodes
//!   (fragments flattened) are the engine's tree with the same nodes, kinds
//!   and child order, and that the core keeps no record the root cannot
//!   reach (`structure`; props live only in the engine's tree, so they are
//!   not compared).
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
//! use kivact_core::{App, prelude::*, test::TestParent};
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
mod screen;

pub use options::{RenderOptions, TestFonts, WindowMode};

pub use screen::{
    Contact, Element, ImeText, LayoutCheck, LayoutRule, LayoutViolation, Screen, Screenshot,
    SemanticNode, Within, failure_dir,
};

use crate::view::IntoView;

/// Why a GPU test cannot start, or `None` when a hardware adapter is available.
/// This probes availability without mounting a view or changing OS settings.
pub fn gpu_unavailable() -> Option<String> {
    kivact_engine::testing::gpu_unavailable()
}

/// Mount `root()` on a real window and settle. `root` is the body of the
/// root component (usually a plain function such as `app` or
/// `|| counter(1)`); it runs once, in the setup of the App's root owner.
///
/// ```ignore
/// let screen = render(app);
/// screen.get_by_role(Role::Button, "+1").click();
/// ```
#[track_caller]
pub fn render<V: IntoView>(root: impl FnOnce() -> V + 'static) -> Screen {
    render_with(RenderOptions::default(), root)
}

/// Mount `root()` with explicit [`RenderOptions`] (theme, locale, messages, size, scale,
/// fonts, window mode, reduced motion, frame period) and settle.
#[track_caller]
pub fn render_with<V: IntoView>(
    options: RenderOptions,
    root: impl FnOnce() -> V + 'static,
) -> Screen {
    let gpu = options.gpu;
    Screen::start(
        options,
        gpu,
        Vec::new(),
        Box::new(move || root().into_view()),
        std::panic::Location::caller(),
    )
}

/// Mount a view with explicit options and return its setup-created handle.
///
/// Use this when assertions need signals or other handles owned by the root
/// setup. The setup runs once, just as in [`render_with`]; mounting and
/// failures retain the caller's location.
///
/// ```no_run
/// # use kivact_core as kivact;
/// use kivact::prelude::*;
/// use kivact::test::{RenderOptions, render_with_handle};
/// let (screen, count) = render_with_handle(RenderOptions::from_env(), || {
///     let count = signal(0);
///     (count, text(move || format!("Count: {}", count.get())))
/// });
/// count.set(1);
/// screen.get_by_text("Count: 1");
/// ```
#[track_caller]
pub fn render_with_handle<H: 'static, V: IntoView>(
    options: RenderOptions,
    setup: impl FnOnce() -> (H, V) + 'static,
) -> (Screen, H) {
    let handle = std::rc::Rc::new(std::cell::RefCell::new(None));
    let captured = handle.clone();
    let screen = render_with(options, move || {
        let (handle, view) = setup();
        *captured.borrow_mut() = Some(handle);
        view
    });
    let handle = handle
        .borrow_mut()
        .take()
        .expect("invariant: render executes root setup");
    (screen, handle)
}

#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
mod parent;
#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
pub use parent::TestParent;
