//! Role: The in-process test driver of the engine (feature `testing`).
//!
//! [`ArvaneTestHost`] runs the production host (`Host::begin`/`handle`, the
//! code the native host contract drives from winit) on the calling test thread,
//! with a window of its own:
//! - frames are drawn on the CPU, bit for bit the same on every machine,
//!   unless a test asks for the GPU ([`TestHostOptions::gpu`]: frames go to
//!   the real window through a GPU surface and are read back);
//! - the clock is virtual ([`ArvaneTestHost::set_now`]): frame ticks,
//!   timers and animations happen at exact times, and the host's next wake
//!   deadline is reported, never slept on;
//! - size and scale are the test's, not the monitor's;
//! - text sees only bundled fonts unless system fonts are asked for
//!   ([`TestFonts`]), so frames are the same on every machine;
//! - the clipboard is in memory, file dialogs answer from a script and
//!   platform requests can be made to fail;
//! - the last frame is kept (screenshots), and frames and the
//!   accessibility tree are mirrored to a real window on the process-wide
//!   test display (`arvane::test_display`) unless the window mode is
//!   headless.
//!
//! Input is injected as `ShellEvent`s at the host's entry
//! ([`ArvaneTestHost::send`]), exactly where the shell's mapped winit
//! events enter: the test window counts clicks by the native window's
//! rules, pointer events go through the host's real hit testing, keys
//! through its key mapping and text editing, IME and accessibility actions
//! through theirs. There is no OS input.
//!
//! The window mode ([`WindowMode`]): `Real` fails when no window opens,
//! `Headless` draws and keeps frames only, `Auto` (the default) is headless
//! when no window opens or `ARVANE_TEST_HEADLESS=1`; frames are the same
//! pixels either way.
//!
//! Fonts: by default only Arvane's bundled test font ([`default_font`]:
//! Tuffy, public domain), never the system's (`Engine::with_fonts_only`);
//! [`TestFonts::Only`] takes another set, [`TestFonts::System`] the
//! system's. The bundled font has no CJK glyphs (they show as boxes), but
//! the layout stays deterministic.
//!
//! The platform is the test's: frame ticks are paced at
//! [`TestHostOptions::frame_period_ns`] (60 Hz by default); the theme,
//! reduced motion, refresh period, safe area and locale change through
//! `ShellEvent::EnvironmentChanged`, the production path (an `EnvChanged`
//! only when something changed); the application can be suspended and
//! resumed (destroying its surface, as Android does, or not), the surface
//! lost and made again, memory warned about, and touch and pen contacts
//! made ([`ArvaneTestHost::contact`]); the software keyboard requests, the
//! cursor, the IME area and text, and the paths asked to open or reveal
//! are recorded.
//!
//! What a test reads: the last frame (savable as PNG), the accessibility
//! tree handed to the window (always built: a test window is always read),
//! the rects of nodes and scene regions, the accessibility ids of nodes
//! and regions, the focus, hover and modal, the host's tree and props,
//! layout findings, and the turns and frame ticks run.
//!
//! Invariants:
//! - The client and the host share the test thread, as they share the UI
//!   thread of a running App; only presentation crosses threads.
//! - A callback error the shell would have stopped on is kept
//!   ([`ArvaneTestHost::failure`]); later events are ignored.

mod display_probe;
#[cfg(debug_assertions)]
mod self_check;
mod test_frame;
mod test_host;
mod test_options;
#[cfg(debug_assertions)]
mod text_reference;

pub use arvane::test_display::{TestDisplayStats, WindowMode};
pub use arvane::{Renderer, ShellEvent, ShellImeText};
pub use kivact_types::render::{SemanticsNode, SemanticsTree};
pub use kivact_types::{
    A11yAction, A11yRequest, ButtonPress, CheckedState, Cursor, DismissReason, FilePhase, Insets,
    Key, KeyCode, LiveRegion, Modifiers, MouseButton, Orientation, Point, PointerId, PointerKind,
    PointerPhase, Role, SortDirection,
};
pub use kivact_types::{PointerSample, Semantics, Transform};

pub use display_probe::{display_stats, gpu_unavailable};
pub use test_frame::TestFrame;
pub use test_host::ArvaneTestHost;
pub use test_options::{TestFonts, TestHostOptions, default_font};

#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
pub use arvane::test_display::TestParent;
