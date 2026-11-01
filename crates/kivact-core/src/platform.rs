//! Role: Platform services and window loop entry.
//!
//! Platform services (fullscreen, clipboard, file and folder dialogs,
//! opening and revealing paths) as free functions returning futures, and
//! the host environment (fullscreen, reduced motion, scale, window size,
//! safe area, surface activity, capabilities) as free functions reading
//! tracked values, owner-scoped lifecycle notifications, plus available font
//! families as a snapshot. The
//! test host of `kivact::test` scripts the services (an
//! in-memory clipboard, scripted dialogs, recorded paths), so they are
//! testable without touching the OS. Also the entry into the engine's window loop, the only place the core
//! starts it.

mod env;
pub(crate) mod lifecycle;
#[doc(inline)]
pub use lifecycle::{LifecycleEvent, on_lifecycle};
mod services;
mod window_loop;

pub(crate) use env::missing_capabilities;
#[doc(inline)]
pub use env::{
    capabilities, font_families, fullscreen, reduced_motion, safe_area, scale_factor,
    surface_active, window_size,
};
#[doc(inline)]
pub use services::{
    FileDialog, copy_text, open_path, pick_file, pick_files, pick_folder, read_text,
    request_window_size, reveal_path, save_file, set_fullscreen,
};
pub(crate) use window_loop::run_native;

#[doc(inline)]
pub use env::{Capabilities, Insets};
