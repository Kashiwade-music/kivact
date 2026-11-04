//! Role: Host environment reads.
//!
//! Host environment as reactive values read with free functions of
//! `kivact::platform`: fullscreen, reduced motion, scale, window size, safe
//! area, surface activity, capabilities and available font families.

use std::panic::Location;

use std::rc::Rc;

use crate::view::state::{EnvState, ViewState};
use kivact_style::draw::Size;
use kivact_wire::HostEnv;
#[doc(inline)]
pub use kivact_wire::{Capabilities, Insets};

/// The App's environment as seen from the nearest owner at the call (`api`
/// names the caller in misuse diagnostics).
fn state(api: &'static str, at: &'static Location<'static>) -> Rc<EnvState> {
    let rt = kivact_reactive::Owner::ambient(api, at).runtime();
    ViewState::of(&rt).env()
}

/// What one field of the host environment is now, tracked: the reader is
/// notified only when that field changes. The accessor named `api` shares
/// one memo of its field.
#[track_caller]
fn env_field<T: Clone + PartialEq + 'static>(api: &'static str, f: fn(&HostEnv) -> T) -> T {
    state(api, Location::caller()).field(api, f).get()
}

/// Whether the App is still building its first host revision.
///
/// Read this during mount setup to distinguish content present on the first
/// screen from content mounted later. This is an untracked snapshot.
#[track_caller]
pub fn is_initial_render() -> bool {
    let rt = kivact_reactive::Owner::ambient("platform::is_initial_render", Location::caller())
        .runtime();
    let revision = ViewState::tree(&rt).engine().stats().revision;
    revision == 0
}

/// The window's available font families, sorted by name without duplicates.
///
/// Names retain their original case and include fonts registered with
/// [`crate::App::font`]. This is an untracked snapshot of the font database.
/// Like other environment reads, call it from setup, a handler, task or effect;
/// a call outside every owner reports a located `SetupOnly` misuse.
#[track_caller]
pub fn font_families() -> Vec<String> {
    let rt = kivact_reactive::Owner::ambient("font_families", Location::caller()).runtime();
    ViewState::tree(&rt).reader().font_families()
}

/// Whether the window is borderless fullscreen, tracked. Like every read
/// of this module it resolves the nearest owner (setup, handler, task or
/// effect) at the call; a located `SetupOnly` misuse outside every owner.
#[track_caller]
pub fn fullscreen() -> bool {
    env_field("fullscreen", |e| e.fullscreen)
}

/// Whether reduced motion is active, tracked. Uses the system preference
/// unless the app sets an override with [`super::set_reduced_motion`].
#[track_caller]
pub fn reduced_motion() -> bool {
    env_field("reduced_motion", |e| e.reduced_motion)
}

/// The display scale factor, tracked.
#[track_caller]
pub fn scale_factor() -> f32 {
    env_field("scale_factor", |e| e.scale)
}

/// Whether the window surface is visible and active, tracked.
#[track_caller]
pub fn surface_active() -> bool {
    env_field("surface_active", |e| e.surface_active)
}

/// The part of each window edge that system UI may cover (a notch, a
/// status bar, rounded corners, a home indicator), tracked: pad
/// content that must stay visible and touchable by it. Zero on
/// desktops, and wherever the platform does not report one
/// ([`Capabilities::safe_area`]).
///
/// ```ignore
/// // Keep the toolbar below a phone's status bar or notch.
/// toolbar().style(move || {
///     let inset = platform::safe_area();
///     style().margin_x(inset.left).top(inset.top)
/// })
/// ```
#[track_caller]
pub fn safe_area() -> Insets {
    let at = Location::caller();
    let rt = kivact_reactive::Owner::ambient("safe_area", at).runtime();
    let state = ViewState::of(&rt);
    // Relying on a safe area the platform does not report is noted
    // (once per App) instead of silently reading zero.
    if !state.capabilities().safe_area && state.note_once_missing("safe area") {
        rt.note(format!(
            "safe_area at {at}: the platform reports no safe area; it stays zero"
        ));
    }
    let env = state.env();
    drop(rt);
    env.field("safe_area", |e| e.safe_area).get()
}

/// The window's size in logical pixels (its content area: what the
/// root fills), tracked: a style or view that reads it follows every
/// resize and scale change. The responsive breakpoints of CSS
/// (`@media (max-width: 860px)`) and its viewport units (`vw`, `vh`)
/// are reads of it.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::prelude::*;
/// fn shell(sidebar: impl IntoView, main: impl IntoView) -> impl IntoView {
///     // @media (max-width: 860px) { .shell { flex-direction: column } }
///     let narrow = memo(move || kivact::platform::window_size().width < 860.0);
///     row((sidebar, main)).style(move || {
///         let s = style().gap(16);
///         if narrow.get() { s.direction(kivact::style::Direction::Column) } else { s }
///     })
/// }
/// ```
#[track_caller]
pub fn window_size() -> Size {
    env_field("window_size", |e| e.window)
}

/// What the platform the App runs on can do (fixed for the App's
/// life; not tracked): whether touch and pens arrive as such, a
/// software keyboard comes up, the safe area and memory warnings are
/// reported, the IME can reconvert, a screen reader reads the App and
/// frames can be drawn on the GPU. What is missing is logged
/// (`tracing`) at start; relying on it ([`safe_area`]) is noted in
/// the turn trace.
///
/// ```ignore
/// if !platform::capabilities().touch {
///     // Hover hints only make sense where a pointer hovers.
/// }
/// ```
#[track_caller]
pub fn capabilities() -> Capabilities {
    let rt = kivact_reactive::Owner::ambient("capabilities", Location::caller()).runtime();
    ViewState::of(&rt).capabilities()
}

/// The names of the capabilities `c` lacks, for diagnostics.
pub(crate) fn missing_capabilities(c: &Capabilities) -> Vec<&'static str> {
    [
        (c.touch, "touch"),
        (c.pen, "pen"),
        (c.soft_keyboard, "soft keyboard"),
        (c.safe_area, "safe area"),
        (c.memory_warnings, "memory warnings"),
        (c.ime_reconversion, "IME reconversion"),
        (c.native_accessibility, "native accessibility"),
        (c.gpu, "GPU"),
    ]
    .into_iter()
    .filter(|(has, _)| !has)
    .map(|(_, name)| name)
    .collect()
}
