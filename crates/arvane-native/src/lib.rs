//! Role: Arvane native backend for top-level and embedded child windows.
//!
//! Arvane's native backend: top-level windows opened with winit and child
//! windows embedded in another program's window (`embed`, feature `embed`).
//! The top-level backend supplies the event loop, IME, presentation over
//! softbuffer and native accessibility over AccessKit. Portable event vocabulary,
//! configuration and contact/click normalization come from `arvane-shell`.
//!
//! Windows and surfaces are UI-thread confined. External types never escape
//! the `arvane` facade, which keeps this crate's windows behind its own
//! window type: what a GPU surface needs from a window ([`arvane_shell::GpuTarget`],
//! [`arvane_shell::ShellWindow::with_gpu_target`]) is handed to it alone. The only `unsafe`
//! code includes native window FFI and the OS loader operations that keep
//! this code mapped while detached threads or providers may still run.
//!
//! Besides windows: the text clipboard (arboard), file and folder dialogs
//! (rfd), opening and revealing paths (`open.rs`) and the system locale
//! (`sys-locale`, `native_input/locale.rs`). What the platform does beyond
//! winit 0.30 (the IME's queries, pens and canceled contacts, settings
//! changes) comes from a Windows subclass of the window (`native_input`);
//! [`arvane_shell::ShellWindow::capabilities`] says what each platform delivers.
//!
//! When upgrading winit, review its Windows input changes before retaining
//! the native subclass workarounds:
//! - Check IME reconversion and surrounding-text requests (`WM_IME_REQUEST`).
//! - Check pen identity, pressure and tilt from pointer events.
//! - Check touch cancellation and suppression of synthesized mouse events.
//! - Remove a workaround only after the replacement preserves its contract;
//!   run the Arvane input E2E tests and the permitted native IME/input checks.
//! - Upgrade windows-sys alongside winit and recheck the subclass signatures.
//!
//! Linux (experimental; the end-to-end tests run on WSLg's Wayland and
//! X11): winit picks the display server (Wayland when `WAYLAND_DISPLAY` or
//! `WAYLAND_SOCKET` is set, else X11 through `DISPLAY`; the Wayland
//! libraries are loaded at run time). Mouse settings and reduced motion are
//! not read from the desktop: the defaults (500 ms, 4 px, 3 lines, no
//! reduced motion). The locale comes from the environment variables, read
//! again when a window regains the focus. The refresh period is the
//! monitor's rate winit reports (Wayland's `wl_output` reports millihertz).
//! The IME is winit's (text-input v3 on Wayland, XIM on X11), with no
//! reconversion or surrounding-text queries. Accessibility is AccessKit's
//! AT-SPI adapter, which reaches the session and AT-SPI buses from a
//! background thread (without a bus nothing is published and the window
//! keeps working). The clipboard covers X11 and Wayland's data-control
//! protocol; dialogs go through the XDG desktop portal. Only WSLg
//! (Ubuntu 24.04: Wayland over Weston, and XWayland) has run it: real
//! desktops (GNOME, KDE), fcitx and ibus, AT-SPI screen readers (Orca),
//! the clipboard and the portal's dialogs are unchecked. On WSLg's X11,
//! Weston's window manager may fail an assertion and drop the XWayland
//! connection, ending the process.
//!
//! A panic in a callback propagates out of the loop: winit carries it across
//! its window procedure (Win32, macOS) and resumes it after the loop
//! ([`arvane_shell::ShellEventLoop::run`]).
#![deny(unsafe_code)]

#[cfg(feature = "embed")]
pub mod embed;

mod a11y;
mod contacts;
mod cpu;
mod detached;
mod display;
mod key_code;
mod module;
mod native_input;
mod open;
mod runtime;
mod services;
#[cfg(feature = "testing")]
pub mod test_display;
mod wait;
mod wheel;

pub use detached::detached_threads;
pub use module::retain_module;
pub use runtime::{Shell, WinitWindow};
