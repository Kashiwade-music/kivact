//! Role: Public embedded App configuration, lifetime and host notices.
//!
//! Build a [`Parent`] from CLAP `clap_window.win32` or `clap_window.x11`,
//! VST3 `"HWND"` or `"X11EmbedWindowID"`, or nih-plug
//! `ParentWindowHandle::Win32Hwnd` or `ParentWindowHandle::X11Window`.
//! The App's initial `window_size` uses logical pixels; the host's `set_size`
//! and `size` use physical pixels. Scale is physical pixels per logical pixel.
//! Dropping [`Embedded`] requests closure without draining queued actions.
//! Keep plugin state outside the App so it survives closing and reopening.
//!
//! A stop is final: the App never runs again, and [`Embedded::status`] keeps
//! reporting it. A panic in the App is reported as [`EmbedStatus::Panicked`];
//! Kivact never unwinds it into the caller, so an embedder behind a C ABI
//! does not abort the host.
//!
//! [`Parent::hidden`] opens without a surface; [`Embedded::show`] and
//! [`Embedded::hide`] follow the host's `gui.show`/`gui.hide`.
//! [`Parent::forward_unhandled_keys`] gives key presses nobody handled, and
//! their releases, back to the parent window. [`Parent::on_resize_request`]
//! answers [`ResizeAnswer::Pending`] when the host confirms the size later with
//! [`Embedded::set_size`].
//!
//! ```no_run
//! # #[cfg(feature = "embed")]
//! # {
//! use kivact_core::{App, view::text, embed::Parent};
//! # fn open(hwnd: *mut std::ffi::c_void) -> kivact_core::Result<()> {
//! let editor = App::new("Editor").window_size(640, 480)
//!     .embed(Parent::win32(hwnd).scale(1.0), || text("Editor"))?;
//! # Ok(())
//! # }
//! # }
//! ```

mod embedded;
mod notice;
mod open;
mod parent;
mod status;

pub use embedded::Embedded;
pub use kivact_types::embed::{ResizeAnswer, UnhandledKeys, Visibility};
pub use notice::EmbedNotice;
pub use parent::Parent;
pub use status::EmbedStatus;
