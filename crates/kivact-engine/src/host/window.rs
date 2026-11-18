//! Role: The host's state for one window.
//!
//! The App-wide `Host` keeps the client, the models and the loop facts; the
//! facts that belong to one native window live here.

use kivact_types::WindowId;
use kivact_types::WindowMode;
use kivact_types::cell::Sent;
use kivact_types::{NativeWindow, SurfaceSize};

/// What a window holds for its content.
pub(super) enum Content {
    /// Kivact draws the window.
    Drawn,
    /// A native child fills the content area; `reported` is the last surface
    /// size sent to the client.
    Foreign {
        #[allow(
            dead_code,
            reason = "the engine keeps the native child handle for the window's lifetime; the client received it in the ready event"
        )]
        child: NativeWindow,
        reported: Sent<SurfaceSize>,
    },
}

/// How long an idle window keeps the CPU frame state of its last shown
/// frame (a later frame redraws only what changed); after it the state is
/// released and the next frame redraws everything.
pub(super) const FRAME_STATE_KEEP_NS: u64 = 1_000_000_000;

/// What became of the last frame presented.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LastPresent {
    /// Shown at `at_ns`: the window keeps its CPU frame state for the next
    /// partial frame until `at_ns + FRAME_STATE_KEEP_NS`.
    Shown { at_ns: u64 },
    /// Nothing to keep or retry: dropped, lost, retried, or released after
    /// idling (a change or a surface loss redraws).
    Settled,
    /// The window had no buffer (occluded, timed out): the frame is drawn again at
    /// `retry_at_ns`, or at once when the window regains the focus.
    Skipped { retry_at_ns: u64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Surface {
    Drawable { generation: u64 },
    Unavailable(Unavailable),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Unavailable {
    NoSurface,
    ZeroSize,
}

#[derive(PartialEq)]
pub(super) enum SoftKeyboard {
    Hidden,
    Shown,
}

#[derive(PartialEq)]
pub(super) enum ImeAllowed {
    No,
    Yes,
}

/// The host's state for one window.
pub(super) struct HostWindow {
    /// What became of the last frame presented (`Presented`).
    pub(super) last_present: LastPresent,
    /// The window's environment as last read: a change of the system's
    /// theme is found by comparing with it (the theme reported at start is
    /// the host's own, not the snapshot's).
    pub(super) environment: Sent<kivact_types::Environment>,
    pub(super) surface: Surface,
    /// The software keyboard was asked for (an editable text input has
    /// the focus).
    pub(super) soft_keyboard: Sent<SoftKeyboard>,
    pub(super) ime_allowed: Sent<ImeAllowed>,
    /// The window's adapter the client was told of last
    /// (`HostEvent::GpuAdapterChanged`).
    pub(super) reported_adapter: Sent<kivact_types::GpuAdapter>,
    /// The work counts of the last frame presented (`Presented::Shown`),
    /// overwritten by every one; `None` before the first.
    pub(super) last_stats: Option<kivact_types::render::FrameStats>,
    /// The kind of window this is.
    pub(super) mode: WindowMode,
    /// The window the request was declared in.
    pub(super) parent: Option<WindowId>,
    /// What the window holds for its content.
    pub(super) content: Content,
}

impl HostWindow {
    pub(super) fn new(mode: WindowMode, parent: Option<WindowId>) -> Self {
        Self {
            last_present: LastPresent::Settled,
            environment: Sent::new(),
            surface: Surface::Unavailable(Unavailable::NoSurface),
            soft_keyboard: Sent::new(),
            ime_allowed: Sent::new(),
            reported_adapter: Sent::new(),
            last_stats: None,
            mode,
            parent,
            content: Content::Drawn,
        }
    }
}

/// The surface state `window` reports now.
pub(super) fn surface_of(window: &arvane::Window<'_>) -> Surface {
    let size = window.size();
    match window.surface_generation() {
        None => Surface::Unavailable(Unavailable::NoSurface),
        Some(_) if size.width <= 0.0 || size.height <= 0.0 => {
            Surface::Unavailable(Unavailable::ZeroSize)
        }
        Some(generation) => Surface::Drawable { generation },
    }
}
