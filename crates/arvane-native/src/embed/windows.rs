//! Role: Win32 embedded children, owned by the host GUI thread.
//!
//! The Send handle owns only atomic/locked command state. Rc state never
//! crosses threads. Every wndproc entry retains it independently; destruction
//! restores the system window procedure and detaches USERDATA before returning.
//! A system-owned STATIC class avoids DLL class registration surviving a
//! parent-driven destruction that drops the last handle from its own cleanup.
#![allow(
    unsafe_code,
    reason = "Win32 child-window FFI is confined to the native backend"
)]

mod a11y;
mod bridge;
mod cpu;
mod create;
mod dpi;
mod ime;
mod input;
mod native;
mod proc;
mod services;
mod state;
mod target;
#[cfg(feature = "test")]
mod test_parent;
mod window;
#[cfg(feature = "test")]
pub use test_parent::TestParent;

pub(super) use create::open;
