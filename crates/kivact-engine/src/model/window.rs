//! Role: The host's state of each platform window (surface, presentation, IME projections), kept with the window's model.
//!
//! The App-wide `Host` keeps the client, the models and the loop facts; the
//! facts that belong to one native window live here.

use kivact_types::SurfaceSize;
use kivact_types::WindowId;
use kivact_types::WindowMode;
use kivact_types::cell::Sent;

/// What a window holds for its content.
pub(crate) enum Content {
    /// Kivact draws the window.
    Drawn,
    /// A native child fills the content area; `reported` is the last surface
    /// size sent to the client.
    Foreign { reported: Sent<SurfaceSize> },
}

/// How long an idle window keeps the CPU frame state of its last shown
/// frame (a later frame redraws only what changed); after it the state is
/// released and the next frame redraws everything.
pub(crate) const FRAME_STATE_KEEP_NS: u64 = 1_000_000_000;

/// What became of the last frame presented.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LastPresent {
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

/// The host's state for one window.
pub(crate) struct HostWindow {
    /// What became of the last frame presented (`Presented`).
    pub(crate) last_present: LastPresent,
    /// The window's environment as last read: a change of the system's
    /// theme is found by comparing with it (the theme reported at start is
    /// the host's own, not the snapshot's).
    pub(crate) environment: Sent<kivact_types::Environment>,
    /// Whether the window can show a frame, as the window last said.
    pub(crate) drawable: arvane_shell::Drawable,
    /// The IME request last sent to the window (`None`: IME disabled).
    pub(crate) ime: Sent<Option<arvane_shell::ImeRequest>>,
    /// The editor's surrounding text as last published by the model.
    pub(crate) ime_text: Option<arvane_shell::ShellImeText>,
    /// The window's adapter the client was told of last
    /// (`HostEvent::GpuAdapterChanged`).
    pub(crate) reported_adapter: Sent<kivact_types::GpuAdapter>,
    /// The work counts of the last frame presented (`Presented::Shown`),
    /// overwritten by every one; `None` before the first.
    pub(crate) last_stats: Option<kivact_types::export::FrameStats>,
    /// The kind of window this is.
    pub(crate) mode: WindowMode,
    /// The window the request was declared in.
    pub(crate) parent: Option<WindowId>,
    /// What the window holds for its content.
    pub(crate) content: Content,
}

impl HostWindow {
    pub(crate) fn new(mode: WindowMode, parent: Option<WindowId>) -> Self {
        Self {
            last_present: LastPresent::Settled,
            environment: Sent::new(),
            drawable: arvane_shell::Drawable::NoSurface,
            ime: Sent::new(),
            ime_text: None,
            reported_adapter: Sent::new(),
            last_stats: None,
            mode,
            parent,
            content: Content::Drawn,
        }
    }
}
