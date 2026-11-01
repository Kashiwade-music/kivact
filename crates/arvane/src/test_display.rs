//! Role: Windows for tests (feature `testing`).
//!
//! [`VirtualWindow`](crate::test_display::VirtualWindow) is the window a test's host draws into: what a native
//! window does, in memory, on the test's thread:
//! - the clock is virtual ([`VirtualWindow::set_now`](crate::test_display::VirtualWindow::set_now));
//! - size, scale and the display's refresh period are the test's, not the
//!   monitor's;
//! - the clipboard is in memory, file and folder dialogs answer from a
//!   script, paths to open or reveal are recorded instead of starting a
//!   program, and platform requests can be made to fail;
//! - frames are drawn on the CPU, bit for bit the same on every machine,
//!   unless the window draws on the GPU (frames then go to the real window
//!   through a GPU surface and are read back);
//! - the last frame is kept (screenshots), and frames and the
//!   accessibility tree are mirrored to a real window on the process-wide
//!   test display (one UI thread and winit event loop per process) unless
//!   the window is headless.
//!
//! Its platform half is a headless implementation of the shell SPI
//! (`arvane_shell::ShellWindow`, `test_display/shell.rs`), the second one
//! beside the native backend: the presenter reaches both through the trait,
//! while scripting, capture and renderer observations stay here.
//!
//! A headless window
//! ([`WindowMode`](crate::test_display::WindowMode)) draws the same
//! frames, pixel for pixel, as one mirrored to a real window. Text is
//! deterministic when the engine sees only
//! [`test_font`](crate::test_display::test_font) (Tuffy, public domain, in
//! `crates/arvane/assets/fonts/`; it has no CJK glyphs, so CJK text draws
//! missing glyphs while its layout stays the same everywhere) through
//! [`crate::Engine::with_fonts_only`].
//!
//! The platform's own doings can be made to happen, by the native
//! window's rules: contacts
//! ([`VirtualWindow::contact`](crate::test_display::VirtualWindow::contact),
//! counted by the same `arvane_shell::Contacts`), focus changes
//! ([`VirtualWindow::focus`](crate::test_display::VirtualWindow::focus);
//! losing it cancels the contacts in progress), assistive technology's
//! action requests (checked as natively), an occluded window (the
//! presenter skips), lost surfaces, environment changes, the surface's
//! destruction and creation, and the renderer forced
//! ([`VirtualWindowOptions::renderer`](crate::test_display::VirtualWindowOptions::renderer)).
//! A GPU test window's device is made on the test's thread as the window
//! opens.

mod environment;
mod font;
mod shell;
mod virtual_window;

pub use arvane_native::test_display::TestDisplayStats;
pub use environment::{display_stats, gpu_unavailable};
pub use font::test_font;
pub use virtual_window::{VirtualWindow, VirtualWindowOptions, WindowMode};

/// A real native host window for embedded application tests.
#[cfg(all(feature = "embed", any(windows, target_os = "linux")))]
pub use arvane_native::embed::TestParent;
