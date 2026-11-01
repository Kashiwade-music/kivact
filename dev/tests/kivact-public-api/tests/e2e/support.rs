//! Role: Shared support and utilities for end-to-end tests.
//! root's setup handed out), the probe fixture of the probe tables and the
//! deterministic random driver of the random probe tests.

pub(crate) mod probe;
pub(crate) mod random;

use kivact::prelude::*;
use kivact::test::{RenderOptions, Screen, render_with_handle};

/// Render the view `f` returns and hand back the screen and what `f`
/// handed out (handles of its setup), located at the call.
#[track_caller]
pub(crate) fn mount<H: 'static, V: IntoView>(f: impl FnOnce() -> (H, V) + 'static) -> (Screen, H) {
    mount_with(RenderOptions::from_env(), f)
}

/// [`mount`] with explicit [`RenderOptions`].
#[track_caller]
pub(crate) fn mount_with<H: 'static, V: IntoView>(
    opts: RenderOptions,
    f: impl FnOnce() -> (H, V) + 'static,
) -> (Screen, H) {
    render_with_handle(opts, f)
}
