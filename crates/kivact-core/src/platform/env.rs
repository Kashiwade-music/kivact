//! Role: Host environment reads.
//!
//! Host environment as reactive values read with free functions of
//! `kivact::platform`: fullscreen, reduced motion, scale, window size, safe
//! area, surface activity, capabilities and available font families.

use std::panic::Location;

use std::rc::Rc;

use crate::view::state::{EnvState, ViewState};
use kivact_runtime::Value;
use kivact_style::draw::Size;
use kivact_types::Environment;
#[doc(inline)]
pub use kivact_types::{
    Capabilities, CommandKey, ContextMenuTrigger, Insets, PointerClass, PointerSettings, ThemeHint,
};

/// The App's environment as seen from the nearest owner at the call (`api`
/// names the caller in misuse diagnostics).
fn state(api: &'static str, at: &'static Location<'static>) -> Rc<EnvState> {
    let rt = kivact_runtime::Runtime::current()
        .unwrap_or_else(|| panic!("`{api}` needs a running App at {at}"));
    ViewState::of(&rt).env()
}

/// What one field of the host environment is now, tracked: the reader is
/// notified only when that field changes. The accessor named `api` shares
/// one memo of its field.
#[track_caller]
fn env_field<T: kivact_runtime::PropValue>(
    api: &'static str,
    f: fn(&Environment) -> T,
) -> Value<T> {
    state(api, Location::caller()).field(api, f)
}

/// Whether the App is still building its first host revision.
///
/// Read this during mount setup to distinguish content present on the first
/// screen from content mounted later. The value changes after the first host revision.
#[track_caller]
pub fn is_initial_render() -> Value<bool> {
    let rt = kivact_runtime::Runtime::current()
        .expect("invariant: is_initial_render requires a running App");
    ViewState::of(&rt).initial_render.into()
}

/// The window's available font families, sorted by name without duplicates.
///
/// Names retain their original case and include fonts registered with
/// [`crate::App::font`]. This value follows changes to the font database.
/// Like other environment reads, call it from setup, a handler, task or effect;
/// a call outside every owner reports a located `SetupOnly` misuse.
#[track_caller]
pub fn font_families() -> Value<Vec<String>> {
    let rt = kivact_runtime::Runtime::current()
        .expect("invariant: font_families requires a running App");
    ViewState::of(&rt).fonts_value.into()
}

/// Whether the window is borderless fullscreen, tracked. Like every read
/// of this module it resolves the nearest owner (setup, handler, task or
/// effect) at the call; a located `SetupOnly` misuse outside every owner.
#[track_caller]
pub fn is_fullscreen() -> Value<bool> {
    env_field("is_fullscreen", |e| e.fullscreen)
}

/// Whether reduced motion is active, tracked. Uses the system preference
/// unless the app sets an override with [`super::set_reduced_motion`].
#[track_caller]
pub fn is_motion_reduced() -> Value<bool> {
    env_field("is_motion_reduced", |e| e.reduced_motion)
}

/// The display scale factor, tracked.
#[track_caller]
pub fn scale_factor() -> Value<f64> {
    env_field("scale_factor", |e| e.scale_factor)
}

/// Whether the window surface is visible and active, tracked.
#[track_caller]
pub fn is_surface_active() -> Value<bool> {
    state("surface_active", Location::caller())
        .surface_active
        .into()
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
///     let inset = platform::safe_area().get();
///     style().margin_x(inset.left).top(inset.top)
/// })
/// ```
#[track_caller]
pub fn safe_area() -> Value<Insets> {
    let at = Location::caller();
    let rt =
        kivact_runtime::Runtime::current().expect("invariant: safe_area requires a running App");
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
    env.field("safe_area", |e| e.safe_area)
}

/// The window's size in logical pixels (its content area: what the
/// root fills), tracked: a style or view that reads it follows every
/// resize and scale change. The responsive breakpoints of CSS
/// (`@media (max-width: 860px)`) and its viewport units (`vw`, `vh`)
/// are reads of it.
///
/// ```
/// # use kivact_core as kivact;
/// use kivact::state::{memo};
/// use kivact::style::{Direction, style};
/// use kivact::view::{IntoView, NodeBuilder, column, row};
/// fn shell(sidebar: impl IntoView, main: impl IntoView) -> impl IntoView {
///     // @media (max-width: 860px) { .shell { flex-direction: column } }
///     let narrow = memo(move || kivact::platform::window_size().get().width < 860.0);
///     row((sidebar, main)).style(move || {
///         let s = style().gap(16);
///         if narrow.get() { s.direction(kivact::style::Direction::Column) } else { s }
///     })
/// }
/// ```
#[track_caller]
pub fn window_size() -> Value<Size> {
    env_field("window_size", |e| e.window_size)
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
/// if !platform::capabilities().get().touch {
///     // Hover hints only make sense where a pointer hovers.
/// }
/// ```
#[track_caller]
pub fn capabilities() -> Value<Capabilities> {
    let rt =
        kivact_runtime::Runtime::current().expect("invariant: capabilities requires a running App");
    ViewState::of(&rt).capabilities_value
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

/// The system pointer preferences.
#[track_caller]
pub fn pointer_settings() -> Value<kivact_types::PointerSettings> {
    env_field("pointer_settings", |e| e.pointer)
}
/// Insets occupied by the software keyboard.
#[track_caller]
pub fn keyboard_inset() -> Value<Insets> {
    env_field("keyboard_inset", |e| e.keyboard_inset)
}
