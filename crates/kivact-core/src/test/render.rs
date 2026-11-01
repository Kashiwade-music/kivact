//! Role: Start a test window: GPU availability and mounting a root view.

use super::options::RenderOptions;
use super::screen::Screen;
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
